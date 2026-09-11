use rusqlite::{params, Connection};
use serde_json::Value;

use super::{s, s_req};
use crate::domain::sync::hlc::Hlc;
use crate::error::AppError;

pub(super) fn upsert_staff(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO staff (
            id, team_id, name, role, pin_salt, pin_hash, deactivated_at,
            created_at, updated_at, hlc_wall_ms, hlc_counter, origin_device_id,
            updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
         ON CONFLICT(id) DO UPDATE SET
            team_id = excluded.team_id, name = excluded.name, role = excluded.role,
            pin_salt = excluded.pin_salt, pin_hash = excluded.pin_hash,
            deactivated_at = excluded.deactivated_at, updated_at = excluded.updated_at,
            hlc_wall_ms = excluded.hlc_wall_ms, hlc_counter = excluded.hlc_counter,
            origin_device_id = excluded.origin_device_id,
            updated_by_staff_id = excluded.updated_by_staff_id, deleted_at = excluded.deleted_at",
        params![
            s_req(v, "id")?,
            s(v, "teamId"),
            s_req(v, "name")?,
            s(v, "role").unwrap_or_else(|| "staff".into()),
            s(v, "pinSalt").unwrap_or_default(),
            s(v, "pinHash").unwrap_or_default(),
            s(v, "deactivatedAt"),
            s(v, "createdAt").unwrap_or_default(),
            s(v, "updatedAt").unwrap_or_default(),
            hlc.wall,
            hlc.counter,
            hlc.origin_device_id,
            s(v, "updatedByStaffId"),
            s(v, "deletedAt"),
        ],
    )?;
    Ok(())
}

pub(super) fn upsert_team(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO teams (
            id, name, created_at, updated_at, hlc_wall_ms, hlc_counter,
            origin_device_id, updated_by_staff_id, deleted_at, pin_hash
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name, updated_at = excluded.updated_at,
            hlc_wall_ms = excluded.hlc_wall_ms, hlc_counter = excluded.hlc_counter,
            origin_device_id = excluded.origin_device_id,
            updated_by_staff_id = excluded.updated_by_staff_id, deleted_at = excluded.deleted_at,
            pin_hash = COALESCE(excluded.pin_hash, teams.pin_hash)",
        params![
            s_req(v, "id")?,
            s_req(v, "name")?,
            s(v, "createdAt").unwrap_or_default(),
            s(v, "updatedAt").unwrap_or_default(),
            hlc.wall,
            hlc.counter,
            hlc.origin_device_id,
            s(v, "updatedByStaffId"),
            s(v, "deletedAt"),
            s(v, "pinHash"),
        ],
    )?;
    Ok(())
}

pub(super) fn upsert_team_device(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO team_devices (
            id, team_id, device_code, device_name, joined_at, removed_at,
            created_at, updated_at, hlc_wall_ms, hlc_counter, origin_device_id,
            updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
         ON CONFLICT(id) DO UPDATE SET
            device_name = excluded.device_name, removed_at = excluded.removed_at,
            updated_at = excluded.updated_at, hlc_wall_ms = excluded.hlc_wall_ms,
            hlc_counter = excluded.hlc_counter, origin_device_id = excluded.origin_device_id,
            updated_by_staff_id = excluded.updated_by_staff_id, deleted_at = excluded.deleted_at",
        params![
            s_req(v, "id")?,
            s_req(v, "teamId")?,
            s_req(v, "deviceCode")?,
            s_req(v, "deviceName")?,
            s(v, "joinedAt").unwrap_or_default(),
            s(v, "removedAt"),
            s(v, "createdAt").unwrap_or_default(),
            s(v, "updatedAt").unwrap_or_default(),
            hlc.wall,
            hlc.counter,
            hlc.origin_device_id,
            s(v, "updatedByStaffId"),
            s(v, "deletedAt"),
        ],
    )?;
    Ok(())
}

pub(super) fn upsert_team_invite(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO team_invites (
            id, team_id, code_hash, created_by_staff_id, expires_at, revoked_at,
            created_at, updated_at, hlc_wall_ms, hlc_counter, origin_device_id,
            updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
         ON CONFLICT(id) DO UPDATE SET
            revoked_at = excluded.revoked_at, updated_at = excluded.updated_at,
            hlc_wall_ms = excluded.hlc_wall_ms, hlc_counter = excluded.hlc_counter,
            origin_device_id = excluded.origin_device_id,
            updated_by_staff_id = excluded.updated_by_staff_id, deleted_at = excluded.deleted_at",
        params![
            s_req(v, "id")?,
            s_req(v, "teamId")?,
            s(v, "codeHash").unwrap_or_default(),
            s_req(v, "createdByStaffId")?,
            s_req(v, "expiresAt")?,
            s(v, "revokedAt"),
            s(v, "createdAt").unwrap_or_default(),
            s(v, "updatedAt").unwrap_or_default(),
            hlc.wall,
            hlc.counter,
            hlc.origin_device_id,
            s(v, "updatedByStaffId"),
            s(v, "deletedAt"),
        ],
    )?;
    Ok(())
}

pub(super) fn upsert_company(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO companies (
            id, legal_name, trade_name, tax_id, address, phone, email, website,
            logo_path, logo_content_hash, is_default, created_at, updated_at, archived_at,
            hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
         ON CONFLICT(id) DO UPDATE SET
            legal_name = excluded.legal_name, trade_name = excluded.trade_name,
            tax_id = excluded.tax_id, address = excluded.address, phone = excluded.phone,
            email = excluded.email, website = excluded.website, logo_path = excluded.logo_path,
            logo_content_hash = excluded.logo_content_hash, is_default = excluded.is_default,
            updated_at = excluded.updated_at, archived_at = excluded.archived_at,
            hlc_wall_ms = excluded.hlc_wall_ms, hlc_counter = excluded.hlc_counter,
            origin_device_id = excluded.origin_device_id,
            updated_by_staff_id = excluded.updated_by_staff_id, deleted_at = excluded.deleted_at",
        params![
            s_req(v, "id")?,
            s_req(v, "legalName")?,
            s(v, "tradeName"),
            s(v, "taxId"),
            s(v, "address"),
            s(v, "phone"),
            s(v, "email"),
            s(v, "website"),
            s(v, "logoPath"),
            s(v, "logoContentHash"),
            i64::from(v.get("isDefault").and_then(|x| x.as_bool()).unwrap_or(false)),
            s(v, "createdAt").unwrap_or_default(),
            s(v, "updatedAt").unwrap_or_default(),
            s(v, "archivedAt"),
            hlc.wall,
            hlc.counter,
            hlc.origin_device_id,
            s(v, "updatedByStaffId"),
            s(v, "deletedAt"),
        ],
    )?;
    Ok(())
}

pub(super) fn apply_shop_settings(conn: &Connection, v: &Value) -> Result<(), AppError> {
    if let Some(tax) = s(v, "taxRatePercent") {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('tax_rate_percent', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![tax],
        )?;
    }
    if let Some(currency) = s(v, "currency") {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('currency', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![currency],
        )?;
    }
    Ok(())
}

pub(super) fn json_field(v: &Value, object_key: &str, raw_key: &str) -> Result<String, AppError> {
    if let Some(raw) = s(v, raw_key) {
        return Ok(raw);
    }
    match v.get(object_key) {
        Some(Value::Null) | None => Err(AppError::Internal {
            message: format!("sync payload missing {object_key}"),
        }),
        Some(other) => serde_json::to_string(other).map_err(|err| AppError::Internal {
            message: format!("failed to serialize {object_key}: {err}"),
        }),
    }
}

pub(super) fn document_type_slug(v: &Value) -> Result<String, AppError> {
    let raw = s_req(v, "documentType")?;
    Ok(match raw.as_str() {
        "entranceSigned" => "entrance_signed".into(),
        "diagnosisSigned" => "diagnosis_signed".into(),
        "summarySigned" => "summary_signed".into(),
        other => other.to_string(),
    })
}
