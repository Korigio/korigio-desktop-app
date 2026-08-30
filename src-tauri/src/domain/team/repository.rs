use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::staff::StaffRole;
use crate::domain::sync::WriteContext;
use crate::domain::team::types::{TeamDevice, TeamInvite};
use crate::error::AppError;

pub fn insert_team(
    conn: &Connection,
    id: &str,
    name: &str,
    pin_hash: Option<&str>,
    now: &str,
    ctx: &WriteContext,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO teams (
            id, name, created_at, updated_at, hlc_wall_ms, hlc_counter,
            origin_device_id, updated_by_staff_id, deleted_at, pin_hash
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, ?9)",
        params![
            id,
            name,
            now,
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id,
            pin_hash
        ],
    )?;
    Ok(())
}

pub fn get_team_row(
    conn: &Connection,
    id: &str,
) -> Result<Option<(String, String, String, String)>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, created_at, updated_at FROM teams WHERE id = ?1 AND deleted_at IS NULL",
    )?;
    Ok(stmt
        .query_row(params![id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .optional()?)
}

pub fn insert_team_device(
    conn: &Connection,
    id: &str,
    team_id: &str,
    device_code: &str,
    device_name: &str,
    now: &str,
    ctx: &WriteContext,
) -> Result<TeamDevice, AppError> {
    conn.execute(
        "INSERT INTO team_devices (
            id, team_id, device_code, device_name, joined_at, removed_at,
            created_at, updated_at, hlc_wall_ms, hlc_counter, origin_device_id,
            updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7, ?8, ?9, ?10, ?11, NULL)",
        params![
            id,
            team_id,
            device_code,
            device_name,
            now,
            now,
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id
        ],
    )?;
    get_team_device(conn, id, id)?.ok_or(AppError::Internal {
        message: "team device missing after insert".into(),
    })
}

pub fn get_team_device(
    conn: &Connection,
    id: &str,
    this_device_id: &str,
) -> Result<Option<TeamDevice>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, device_code, device_name, joined_at, removed_at
         FROM team_devices WHERE id = ?1 AND deleted_at IS NULL",
    )?;
    Ok(stmt
        .query_row(params![id], |row| {
            let device_id: String = row.get(0)?;
            Ok(TeamDevice {
                id: device_id.clone(),
                device_code: row.get(1)?,
                device_name: row.get(2)?,
                joined_at: row.get(3)?,
                removed_at: row.get(4)?,
                is_this_device: device_id == this_device_id,
            })
        })
        .optional()?)
}

pub fn list_team_devices(
    conn: &Connection,
    team_id: &str,
    this_device_id: &str,
) -> Result<Vec<TeamDevice>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, device_code, device_name, joined_at, removed_at
         FROM team_devices
         WHERE team_id = ?1 AND deleted_at IS NULL
         ORDER BY device_code ASC",
    )?;
    let rows = stmt
        .query_map(params![team_id], |row| {
            let device_id: String = row.get(0)?;
            Ok(TeamDevice {
                id: device_id.clone(),
                device_code: row.get(1)?,
                device_name: row.get(2)?,
                joined_at: row.get(3)?,
                removed_at: row.get(4)?,
                is_this_device: device_id == this_device_id,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn count_active_devices(conn: &Connection, team_id: &str) -> Result<i64, AppError> {
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM team_devices
         WHERE team_id = ?1 AND deleted_at IS NULL AND removed_at IS NULL",
        params![team_id],
        |row| row.get(0),
    )?)
}

pub fn used_device_codes(conn: &Connection, team_id: &str) -> Result<Vec<String>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT device_code FROM team_devices
         WHERE team_id = ?1 AND deleted_at IS NULL AND removed_at IS NULL",
    )?;
    let rows = stmt
        .query_map(params![team_id], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn set_device_removed(
    conn: &Connection,
    id: &str,
    removed_at: &str,
    now: &str,
    ctx: &WriteContext,
) -> Result<TeamDevice, AppError> {
    conn.execute(
        "UPDATE team_devices SET removed_at = ?1, updated_at = ?2,
            hlc_wall_ms = ?3, hlc_counter = ?4, origin_device_id = ?5, updated_by_staff_id = ?6
         WHERE id = ?7",
        params![
            removed_at,
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id,
            id
        ],
    )?;
    let identity = crate::domain::identity::require_local_identity(conn)?;
    get_team_device(conn, id, &identity.device_id)?.ok_or(AppError::NotFound)
}

pub fn rename_device(
    conn: &Connection,
    id: &str,
    name: &str,
    now: &str,
    ctx: &WriteContext,
) -> Result<TeamDevice, AppError> {
    conn.execute(
        "UPDATE team_devices SET device_name = ?1, updated_at = ?2,
            hlc_wall_ms = ?3, hlc_counter = ?4, origin_device_id = ?5, updated_by_staff_id = ?6
         WHERE id = ?7 AND deleted_at IS NULL",
        params![
            name,
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id,
            id
        ],
    )?;
    get_team_device(conn, id, id)?.ok_or(AppError::NotFound)
}

pub fn insert_invite(
    conn: &Connection,
    id: &str,
    team_id: &str,
    code_hash: &str,
    created_by: &str,
    expires_at: &str,
    now: &str,
    ctx: &WriteContext,
) -> Result<TeamInvite, AppError> {
    conn.execute(
        "INSERT INTO team_invites (
            id, team_id, code_hash, created_by_staff_id, expires_at, revoked_at,
            created_at, updated_at, hlc_wall_ms, hlc_counter, origin_device_id,
            updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7, ?8, ?9, ?10, ?11, NULL)",
        params![
            id,
            team_id,
            code_hash,
            created_by,
            expires_at,
            now,
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id
        ],
    )?;
    get_invite(conn, id, None)?.ok_or(AppError::Internal {
        message: "invite missing after insert".into(),
    })
}

pub fn get_invite(
    conn: &Connection,
    id: &str,
    plaintext: Option<String>,
) -> Result<Option<TeamInvite>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, expires_at, revoked_at, created_at FROM team_invites
         WHERE id = ?1 AND deleted_at IS NULL",
    )?;
    Ok(stmt
        .query_row(params![id], |row| {
            Ok(TeamInvite {
                id: row.get(0)?,
                code: plaintext.clone(),
                expires_at: row.get(1)?,
                revoked_at: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .optional()?)
}

pub fn list_invites(conn: &Connection, team_id: &str) -> Result<Vec<TeamInvite>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, expires_at, revoked_at, created_at FROM team_invites
         WHERE team_id = ?1 AND deleted_at IS NULL
         ORDER BY created_at DESC",
    )?;
    let rows = stmt
        .query_map(params![team_id], |row| {
            Ok(TeamInvite {
                id: row.get(0)?,
                code: None,
                expires_at: row.get(1)?,
                revoked_at: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn find_invite_by_hash(
    conn: &Connection,
    code_hash: &str,
) -> Result<Option<(String, String, Option<String>)>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, expires_at, revoked_at FROM team_invites
         WHERE code_hash = ?1 AND deleted_at IS NULL
         ORDER BY created_at DESC LIMIT 1",
    )?;
    Ok(stmt
        .query_row(params![code_hash], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .optional()?)
}

pub fn revoke_invite(
    conn: &Connection,
    id: &str,
    now: &str,
    ctx: &WriteContext,
) -> Result<TeamInvite, AppError> {
    conn.execute(
        "UPDATE team_invites SET revoked_at = ?1, updated_at = ?2,
            hlc_wall_ms = ?3, hlc_counter = ?4, origin_device_id = ?5, updated_by_staff_id = ?6
         WHERE id = ?7 AND deleted_at IS NULL",
        params![
            now,
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id,
            id
        ],
    )?;
    get_invite(conn, id, None)?.ok_or(AppError::NotFound)
}

pub fn get_team_pin_hash(conn: &Connection, team_id: &str) -> Result<Option<String>, AppError> {
    let mut stmt =
        conn.prepare("SELECT pin_hash FROM teams WHERE id = ?1 AND deleted_at IS NULL")?;
    Ok(stmt
        .query_row(params![team_id], |row| row.get(0))
        .optional()?)
}

pub fn set_team_pin_hash(
    conn: &Connection,
    team_id: &str,
    pin_hash: &str,
    now: &str,
    ctx: &WriteContext,
) -> Result<(), AppError> {
    conn.execute(
        "UPDATE teams SET pin_hash = ?1, updated_at = ?2,
            hlc_wall_ms = ?3, hlc_counter = ?4, origin_device_id = ?5, updated_by_staff_id = ?6
         WHERE id = ?7 AND deleted_at IS NULL",
        params![
            pin_hash,
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id,
            team_id
        ],
    )?;
    Ok(())
}

pub fn list_active_staff_gigs(
    conn: &Connection,
    team_id: &str,
) -> Result<Vec<(String, String, StaffRole, i64)>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.name, s.role,
            (SELECT COUNT(*) FROM repairs r
             WHERE r.assigned_to_staff_id = s.id AND r.deleted_at IS NULL)
         FROM staff s
         WHERE s.team_id = ?1 AND s.deleted_at IS NULL AND s.deactivated_at IS NULL",
    )?;
    let rows = stmt
        .query_map(params![team_id], |row| {
            let role_raw: String = row.get(2)?;
            Ok((
                row.get(0)?,
                row.get(1)?,
                StaffRole::parse(&role_raw).unwrap_or(StaffRole::Staff),
                row.get(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn presence_staff_ids_for_devices(
    conn: &Connection,
    device_ids: &[String],
) -> Result<Vec<String>, AppError> {
    if device_ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut ids = Vec::new();
    let mut stmt = conn
        .prepare("SELECT staff_id FROM presence WHERE device_id = ?1 AND staff_id IS NOT NULL")?;
    for device_id in device_ids {
        if let Some(staff_id) = stmt
            .query_row(params![device_id], |row| row.get::<_, Option<String>>(0))
            .optional()?
            .flatten()
        {
            ids.push(staff_id);
        }
    }
    Ok(ids)
}

pub fn update_local_identity_team(
    conn: &Connection,
    team_id: Option<&str>,
    team_psk: Option<&str>,
    device_code: Option<&str>,
    device_name: Option<&str>,
    team_pin: Option<&str>,
) -> Result<(), AppError> {
    conn.execute(
        "UPDATE local_identity SET
            team_id = ?1,
            team_psk = ?2,
            device_code = COALESCE(?3, device_code),
            device_name = COALESCE(?4, device_name),
            team_pin = ?5
         WHERE id = 1",
        params![team_id, team_psk, device_code, device_name, team_pin],
    )?;
    Ok(())
}
