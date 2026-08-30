//! This-PC local_identity and write-session helpers.

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::ids::new_entity_id;
use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct LocalIdentity {
    pub device_id: String,
    pub device_code: Option<String>,
    pub device_name: String,
    pub team_id: Option<String>,
    pub team_psk: Option<String>,
    pub team_pin: Option<String>,
    pub current_staff_id: Option<String>,
    pub session_started_at: Option<String>,
    pub hlc_wall_ms: i64,
    pub hlc_counter: i64,
}

pub fn default_device_name() -> String {
    for key in ["COMPUTERNAME", "HOSTNAME"] {
        if let Ok(name) = std::env::var(key) {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    #[cfg(unix)]
    {
        if let Ok(name) = std::fs::read_to_string("/etc/hostname") {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    "PC".into()
}

pub fn ensure_local_identity(conn: &Connection) -> Result<LocalIdentity, AppError> {
    if let Some(existing) = get_local_identity(conn)? {
        return Ok(existing);
    }
    let device_id = new_entity_id();
    let device_name = default_device_name();
    conn.execute(
        "INSERT INTO local_identity (
            id, device_id, device_code, device_name, team_id, team_psk,
            current_staff_id, session_started_at, hlc_wall_ms, hlc_counter
         ) VALUES (1, ?1, NULL, ?2, NULL, NULL, NULL, NULL, 0, 0)",
        params![device_id, device_name],
    )?;
    get_local_identity(conn)?.ok_or(AppError::Internal {
        message: "local_identity missing after insert".into(),
    })
}

pub fn get_local_identity(conn: &Connection) -> Result<Option<LocalIdentity>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT device_id, device_code, device_name, team_id, team_psk, team_pin,
                current_staff_id, session_started_at, hlc_wall_ms, hlc_counter
         FROM local_identity WHERE id = 1",
    )?;
    let row = stmt
        .query_row([], |row| {
            Ok(LocalIdentity {
                device_id: row.get(0)?,
                device_code: row.get(1)?,
                device_name: row.get(2)?,
                team_id: row.get(3)?,
                team_psk: row.get(4)?,
                team_pin: row.get(5)?,
                current_staff_id: row.get(6)?,
                session_started_at: row.get(7)?,
                hlc_wall_ms: row.get(8)?,
                hlc_counter: row.get(9)?,
            })
        })
        .optional()?;
    Ok(row)
}

pub fn require_local_identity(conn: &Connection) -> Result<LocalIdentity, AppError> {
    get_local_identity(conn)?.ok_or(AppError::Internal {
        message: "local_identity is not initialized".into(),
    })
}

/// Solo writes are allowed unsigned; team-mode writes require a session.
pub fn require_write_access(conn: &Connection) -> Result<LocalIdentity, AppError> {
    let identity = require_local_identity(conn)?;
    if identity.team_id.is_some() && identity.current_staff_id.is_none() {
        return Err(AppError::unauthorized());
    }
    Ok(identity)
}

pub fn current_staff_id(conn: &Connection) -> Result<Option<String>, AppError> {
    Ok(require_local_identity(conn)?.current_staff_id)
}

pub fn repair_device_code(conn: &Connection) -> Result<String, AppError> {
    Ok(require_local_identity(conn)?
        .device_code
        .unwrap_or_else(|| "AA".into()))
}
