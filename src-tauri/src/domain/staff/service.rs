use rusqlite::{params, Connection};

use crate::db::repository::now_utc_rfc3339;
use crate::domain::identity;
use crate::domain::ids::{new_entity_id, parse_entity_id};
use crate::domain::staff::pin::{hash_pin, verify_pin};
use crate::domain::staff::repository::{self, StaffInsert};
use crate::domain::staff::types::{
    Session, Staff, StaffInput, StaffListQuery, StaffListResult, StaffNameInput, StaffPinInput,
    StaffRole,
};
use crate::domain::staff::validation::{validate_pin, validate_staff_name};
use crate::domain::sync::{self, begin_write};
use crate::error::AppError;

pub fn require_session(conn: &Connection) -> Result<Session, AppError> {
    get_current_session(conn)?.ok_or_else(AppError::unauthorized)
}

pub fn require_admin(conn: &Connection) -> Result<Session, AppError> {
    let session = require_session(conn)?;
    if session.staff.role != StaffRole::Admin {
        return Err(AppError::forbidden("Only an admin can do this."));
    }
    Ok(session)
}

pub fn list_staff(conn: &Connection, query: StaffListQuery) -> Result<StaffListResult, AppError> {
    let include_deactivated = query.include_deactivated.unwrap_or(false);
    let (items, total) = repository::list_staff(conn, query.query.as_deref(), include_deactivated)?;
    Ok(StaffListResult { items, total })
}

pub fn get_staff(conn: &Connection, id: &str) -> Result<Staff, AppError> {
    let id = parse_entity_id(id)?;
    repository::get_staff_by_id(conn, &id)?.ok_or(AppError::NotFound)
}

/// Create a staff row with an internal unguessable PIN and sign them in.
/// Caller must invoke this before `local_identity.team_id` is set (or with a session).
pub fn create_signed_in_member(
    conn: &Connection,
    name: &str,
    role: StaffRole,
    team_id: Option<&str>,
) -> Result<Session, AppError> {
    let name = validate_staff_name(name)?;
    let mut bytes = [0u8; 16];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut bytes);
    let pin = hex::encode(bytes);
    let ctx = begin_write(conn)?;
    let (salt, hash) = hash_pin(&pin)?;
    let id = new_entity_id();
    let now = now_utc_rfc3339()?;
    let staff = repository::insert_staff(
        conn,
        &StaffInsert {
            id: id.clone(),
            team_id: team_id.map(str::to_string),
            name,
            role,
            pin_salt: salt,
            pin_hash: hash,
        },
        &now,
        &ctx,
    )?;
    sync::record_upsert(
        conn,
        "staff",
        &staff.id,
        staff_sync_payload(conn, &staff)?,
        &ctx,
    )?;
    conn.execute(
        "UPDATE local_identity SET current_staff_id = ?1, session_started_at = ?2 WHERE id = 1",
        params![id, now],
    )?;
    get_current_session(conn)?.ok_or(AppError::Internal {
        message: "session missing after member create".into(),
    })
}

pub fn create_staff(conn: &Connection, input: StaffInput) -> Result<Staff, AppError> {
    let name = validate_staff_name(&input.name)?;
    let pin = validate_pin(&input.pin)?;
    let identity = identity::require_local_identity(conn)?;
    let count = repository::count_staff(conn)?;

    let role = if identity.team_id.is_some() {
        require_admin(conn)?;
        match input.role {
            Some(StaffRole::Admin) => StaffRole::Admin,
            _ => StaffRole::Staff,
        }
    } else {
        StaffRole::Staff
    };

    let ctx = begin_write(conn)?;
    let (salt, hash) = hash_pin(&pin)?;
    let id = new_entity_id();
    let now = now_utc_rfc3339()?;
    let staff = repository::insert_staff(
        conn,
        &StaffInsert {
            id: id.clone(),
            team_id: identity.team_id,
            name,
            role,
            pin_salt: salt,
            pin_hash: hash,
        },
        &now,
        &ctx,
    )?;
    sync::record_upsert(
        conn,
        "staff",
        &staff.id,
        staff_sync_payload(conn, &staff)?,
        &ctx,
    )?;
    let _ = count;
    Ok(staff)
}

pub fn update_staff(conn: &Connection, id: &str, input: StaffNameInput) -> Result<Staff, AppError> {
    let id = parse_entity_id(id)?;
    let name = validate_staff_name(&input.name)?;
    let existing = get_staff(conn, &id)?;
    let identity = identity::require_local_identity(conn)?;
    if identity.team_id.is_some() {
        let session = require_session(conn)?;
        let is_self = session.staff.id == id;
        if !is_self && session.staff.role != StaffRole::Admin {
            return Err(AppError::forbidden("Only an admin can rename other staff."));
        }
    }
    let _ = existing;
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let staff = repository::update_staff_name(conn, &id, &name, &now, &ctx)?;
    sync::record_upsert(
        conn,
        "staff",
        &staff.id,
        staff_sync_payload(conn, &staff)?,
        &ctx,
    )?;
    Ok(staff)
}

pub fn deactivate_staff(conn: &Connection, id: &str) -> Result<Staff, AppError> {
    let id = parse_entity_id(id)?;
    require_admin(conn)?;
    let existing = get_staff(conn, &id)?;
    if existing.deactivated_at.is_some() {
        return Ok(existing);
    }
    ensure_not_last_admin(conn, &existing, true)?;
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let staff = repository::set_deactivated_at(conn, &id, Some(&now), &now, &ctx)?;
    if identity::current_staff_id(conn)?.as_deref() == Some(id.as_str()) {
        sign_out_staff(conn)?;
    }
    sync::record_upsert(
        conn,
        "staff",
        &staff.id,
        staff_sync_payload(conn, &staff)?,
        &ctx,
    )?;
    Ok(staff)
}

pub fn reactivate_staff(conn: &Connection, id: &str) -> Result<Staff, AppError> {
    let id = parse_entity_id(id)?;
    require_admin(conn)?;
    let existing = get_staff(conn, &id)?;
    if existing.deactivated_at.is_none() {
        return Ok(existing);
    }
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let staff = repository::set_deactivated_at(conn, &id, None, &now, &ctx)?;
    sync::record_upsert(
        conn,
        "staff",
        &staff.id,
        staff_sync_payload(conn, &staff)?,
        &ctx,
    )?;
    Ok(staff)
}

pub fn change_staff_role(conn: &Connection, id: &str, role: StaffRole) -> Result<Staff, AppError> {
    let id = parse_entity_id(id)?;
    require_admin(conn)?;
    let existing = get_staff(conn, &id)?;
    if existing.role == role {
        return Ok(existing);
    }
    if existing.role == StaffRole::Admin && role == StaffRole::Staff {
        ensure_not_last_admin(conn, &existing, false)?;
    }
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let staff = repository::set_role(conn, &id, role, &now, &ctx)?;
    sync::record_upsert(
        conn,
        "staff",
        &staff.id,
        staff_sync_payload(conn, &staff)?,
        &ctx,
    )?;
    Ok(staff)
}

pub fn set_staff_pin(conn: &Connection, id: &str, input: StaffPinInput) -> Result<(), AppError> {
    let id = parse_entity_id(id)?;
    let new_pin = validate_pin(&input.new_pin)?;
    let existing = get_staff(conn, &id)?;
    let session = get_current_session(conn)?;
    let is_self = session.as_ref().map(|s| s.staff.id.as_str()) == Some(id.as_str());
    let is_admin = session
        .as_ref()
        .map(|s| s.staff.role == StaffRole::Admin)
        .unwrap_or(false);

    if is_self {
        let current = input.current_pin.as_deref().unwrap_or("");
        let current = validate_pin(current).map_err(|_| AppError::invalid_credentials())?;
        let (salt, hash) = repository::get_pin_secrets(conn, &id)?.ok_or(AppError::NotFound)?;
        if !verify_pin(&current, &salt, &hash) {
            return Err(AppError::invalid_credentials());
        }
    } else if !is_admin {
        return Err(AppError::forbidden("Only an admin can reset another PIN."));
    }

    let _ = existing;
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let (salt, hash) = hash_pin(&new_pin)?;
    repository::set_pin(conn, &id, &salt, &hash, &now, &ctx)?;
    let staff = get_staff(conn, &id)?;
    sync::record_upsert(conn, "staff", &id, staff_sync_payload(conn, &staff)?, &ctx)?;
    Ok(())
}

pub fn sign_in_staff(conn: &Connection, staff_id: &str, pin: &str) -> Result<Session, AppError> {
    let id = parse_entity_id_or_unauthorized(staff_id)?;
    let pin = validate_pin(pin).map_err(|_| AppError::invalid_credentials())?;
    let staff =
        repository::get_staff_by_id(conn, &id)?.ok_or_else(AppError::invalid_credentials)?;
    if staff.deactivated_at.is_some() {
        return Err(AppError::forbidden("This staff account is deactivated."));
    }
    let (salt, hash) =
        repository::get_pin_secrets(conn, &id)?.ok_or_else(AppError::invalid_credentials)?;
    if !verify_pin(&pin, &salt, &hash) {
        return Err(AppError::invalid_credentials());
    }
    let now = now_utc_rfc3339()?;
    conn.execute(
        "UPDATE local_identity SET current_staff_id = ?1, session_started_at = ?2 WHERE id = 1",
        params![id, now],
    )?;
    get_current_session(conn)?.ok_or(AppError::Internal {
        message: "session missing after sign-in".into(),
    })
}

pub fn sign_out_staff(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "UPDATE local_identity SET current_staff_id = NULL, session_started_at = NULL WHERE id = 1",
        [],
    )?;
    Ok(())
}

pub fn get_current_session(conn: &Connection) -> Result<Option<Session>, AppError> {
    let identity = identity::require_local_identity(conn)?;
    let Some(staff_id) = identity.current_staff_id else {
        return Ok(None);
    };
    let Some(staff) = repository::get_staff_by_id(conn, &staff_id)? else {
        sign_out_staff(conn)?;
        return Ok(None);
    };
    if staff.deactivated_at.is_some() {
        sign_out_staff(conn)?;
        return Ok(None);
    }
    let started_at = identity.session_started_at.unwrap_or_default();
    Ok(Some(Session {
        staff,
        device_id: identity.device_id,
        device_name: identity.device_name,
        device_code: identity.device_code,
        started_at,
    }))
}

fn ensure_not_last_admin(
    conn: &Connection,
    staff: &Staff,
    deactivating: bool,
) -> Result<(), AppError> {
    if staff.role != StaffRole::Admin || staff.deactivated_at.is_some() {
        return Ok(());
    }
    let others = repository::count_active_admins(conn, Some(&staff.id))?;
    if others == 0 {
        let msg = if deactivating {
            "Cannot deactivate the last admin."
        } else {
            "Cannot demote the last admin."
        };
        return Err(AppError::forbidden(msg));
    }
    Ok(())
}

fn parse_entity_id_or_unauthorized(id: &str) -> Result<String, AppError> {
    parse_entity_id(id).map_err(|_| AppError::invalid_credentials())
}

pub fn staff_sync_payload(conn: &Connection, staff: &Staff) -> Result<serde_json::Value, AppError> {
    let (salt, hash) = repository::get_pin_secrets(conn, &staff.id)?.unwrap_or_default();
    Ok(serde_json::json!({
        "id": staff.id,
        "teamId": staff.team_id,
        "name": staff.name,
        "role": staff.role.as_str(),
        "deactivatedAt": staff.deactivated_at,
        "createdAt": staff.created_at,
        "updatedAt": staff.updated_at,
        "pinSalt": salt,
        "pinHash": hash,
        "updatedByStaffId": identity::current_staff_id(conn)?,
    }))
}
