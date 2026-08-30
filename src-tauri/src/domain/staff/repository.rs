use rusqlite::{params, Connection, OptionalExtension};

use crate::db::repository::like_pattern;
use crate::domain::staff::types::{Staff, StaffRole};
use crate::domain::sync::WriteContext;
use crate::error::AppError;

pub struct StaffInsert {
    pub id: String,
    pub team_id: Option<String>,
    pub name: String,
    pub role: StaffRole,
    pub pin_salt: String,
    pub pin_hash: String,
}

pub fn insert_staff(
    conn: &Connection,
    input: &StaffInsert,
    now: &str,
    ctx: &WriteContext,
) -> Result<Staff, AppError> {
    conn.execute(
        "INSERT INTO staff (
            id, team_id, name, role, pin_salt, pin_hash, deactivated_at,
            created_at, updated_at, hlc_wall_ms, hlc_counter, origin_device_id,
            updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, ?7, ?8, ?9, ?10, ?11, ?12, NULL)",
        params![
            input.id,
            input.team_id,
            input.name,
            input.role.as_str(),
            input.pin_salt,
            input.pin_hash,
            now,
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id,
        ],
    )?;
    get_staff_by_id(conn, &input.id)?.ok_or(AppError::Internal {
        message: "staff missing after insert".into(),
    })
}

pub fn update_staff_name(
    conn: &Connection,
    id: &str,
    name: &str,
    now: &str,
    ctx: &WriteContext,
) -> Result<Staff, AppError> {
    let updated = conn.execute(
        "UPDATE staff SET name = ?1, updated_at = ?2, hlc_wall_ms = ?3, hlc_counter = ?4,
            origin_device_id = ?5, updated_by_staff_id = ?6
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
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    get_staff_by_id(conn, id)?.ok_or(AppError::NotFound)
}

pub fn set_role(
    conn: &Connection,
    id: &str,
    role: StaffRole,
    now: &str,
    ctx: &WriteContext,
) -> Result<Staff, AppError> {
    let updated = conn.execute(
        "UPDATE staff SET role = ?1, updated_at = ?2, hlc_wall_ms = ?3, hlc_counter = ?4,
            origin_device_id = ?5, updated_by_staff_id = ?6
         WHERE id = ?7 AND deleted_at IS NULL",
        params![
            role.as_str(),
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id,
            id
        ],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    get_staff_by_id(conn, id)?.ok_or(AppError::NotFound)
}

pub fn set_deactivated_at(
    conn: &Connection,
    id: &str,
    deactivated_at: Option<&str>,
    now: &str,
    ctx: &WriteContext,
) -> Result<Staff, AppError> {
    let updated = conn.execute(
        "UPDATE staff SET deactivated_at = ?1, updated_at = ?2, hlc_wall_ms = ?3, hlc_counter = ?4,
            origin_device_id = ?5, updated_by_staff_id = ?6
         WHERE id = ?7 AND deleted_at IS NULL",
        params![
            deactivated_at,
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id,
            id
        ],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    get_staff_by_id(conn, id)?.ok_or(AppError::NotFound)
}

pub fn set_pin(
    conn: &Connection,
    id: &str,
    salt: &str,
    hash: &str,
    now: &str,
    ctx: &WriteContext,
) -> Result<(), AppError> {
    let updated = conn.execute(
        "UPDATE staff SET pin_salt = ?1, pin_hash = ?2, updated_at = ?3,
            hlc_wall_ms = ?4, hlc_counter = ?5, origin_device_id = ?6, updated_by_staff_id = ?7
         WHERE id = ?8 AND deleted_at IS NULL",
        params![
            salt,
            hash,
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id,
            id
        ],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub fn set_team_id(
    conn: &Connection,
    id: &str,
    team_id: Option<&str>,
    now: &str,
    ctx: &WriteContext,
) -> Result<Staff, AppError> {
    conn.execute(
        "UPDATE staff SET team_id = ?1, updated_at = ?2, hlc_wall_ms = ?3, hlc_counter = ?4,
            origin_device_id = ?5, updated_by_staff_id = ?6
         WHERE id = ?7",
        params![
            team_id,
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id,
            id
        ],
    )?;
    get_staff_by_id(conn, id)?.ok_or(AppError::NotFound)
}

pub fn get_staff_by_id(conn: &Connection, id: &str) -> Result<Option<Staff>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, team_id, name, role, deactivated_at, created_at, updated_at
         FROM staff WHERE id = ?1 AND deleted_at IS NULL",
    )?;
    Ok(stmt.query_row(params![id], map_staff).optional()?)
}

pub fn get_pin_secrets(conn: &Connection, id: &str) -> Result<Option<(String, String)>, AppError> {
    let mut stmt =
        conn.prepare("SELECT pin_salt, pin_hash FROM staff WHERE id = ?1 AND deleted_at IS NULL")?;
    Ok(stmt
        .query_row(params![id], |row| Ok((row.get(0)?, row.get(1)?)))
        .optional()?)
}

pub fn count_staff(conn: &Connection) -> Result<i64, AppError> {
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM staff WHERE deleted_at IS NULL",
        [],
        |row| row.get(0),
    )?)
}

pub fn count_active_admins(conn: &Connection, except_id: Option<&str>) -> Result<i64, AppError> {
    match except_id {
        Some(id) => Ok(conn.query_row(
            "SELECT COUNT(*) FROM staff
             WHERE deleted_at IS NULL AND deactivated_at IS NULL AND role = 'admin' AND id != ?1",
            params![id],
            |row| row.get(0),
        )?),
        None => Ok(conn.query_row(
            "SELECT COUNT(*) FROM staff
             WHERE deleted_at IS NULL AND deactivated_at IS NULL AND role = 'admin'",
            [],
            |row| row.get(0),
        )?),
    }
}

pub fn list_staff(
    conn: &Connection,
    search: Option<&str>,
    include_deactivated: bool,
) -> Result<(Vec<Staff>, i64), AppError> {
    let pattern = like_pattern(search);
    let total: i64 = match (&pattern, include_deactivated) {
        (None, false) => conn.query_row(
            "SELECT COUNT(*) FROM staff WHERE deleted_at IS NULL AND deactivated_at IS NULL",
            [],
            |row| row.get(0),
        )?,
        (None, true) => conn.query_row(
            "SELECT COUNT(*) FROM staff WHERE deleted_at IS NULL",
            [],
            |row| row.get(0),
        )?,
        (Some(p), false) => conn.query_row(
            "SELECT COUNT(*) FROM staff
             WHERE deleted_at IS NULL AND deactivated_at IS NULL AND name LIKE ?1 ESCAPE '\\'",
            params![p],
            |row| row.get(0),
        )?,
        (Some(p), true) => conn.query_row(
            "SELECT COUNT(*) FROM staff
             WHERE deleted_at IS NULL AND name LIKE ?1 ESCAPE '\\'",
            params![p],
            |row| row.get(0),
        )?,
    };

    let sql = match (&pattern, include_deactivated) {
        (None, false) => {
            "SELECT id, team_id, name, role, deactivated_at, created_at, updated_at
             FROM staff WHERE deleted_at IS NULL AND deactivated_at IS NULL
             ORDER BY name COLLATE NOCASE ASC, id ASC"
        }
        (None, true) => {
            "SELECT id, team_id, name, role, deactivated_at, created_at, updated_at
             FROM staff WHERE deleted_at IS NULL
             ORDER BY name COLLATE NOCASE ASC, id ASC"
        }
        (Some(_), false) => {
            "SELECT id, team_id, name, role, deactivated_at, created_at, updated_at
             FROM staff WHERE deleted_at IS NULL AND deactivated_at IS NULL
               AND name LIKE ?1 ESCAPE '\\'
             ORDER BY name COLLATE NOCASE ASC, id ASC"
        }
        (Some(_), true) => {
            "SELECT id, team_id, name, role, deactivated_at, created_at, updated_at
             FROM staff WHERE deleted_at IS NULL AND name LIKE ?1 ESCAPE '\\'
             ORDER BY name COLLATE NOCASE ASC, id ASC"
        }
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = match &pattern {
        None => stmt
            .query_map([], map_staff)?
            .collect::<Result<Vec<_>, _>>()?,
        Some(p) => stmt
            .query_map(params![p], map_staff)?
            .collect::<Result<Vec<_>, _>>()?,
    };
    Ok((rows, total))
}

fn map_staff(row: &rusqlite::Row<'_>) -> rusqlite::Result<Staff> {
    let role_raw: String = row.get(3)?;
    Ok(Staff {
        id: row.get(0)?,
        team_id: row.get(1)?,
        name: row.get(2)?,
        role: StaffRole::parse(&role_raw).unwrap_or(StaffRole::Staff),
        deactivated_at: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}
