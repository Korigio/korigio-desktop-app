use rusqlite::{Connection, OptionalExtension, params};

use crate::domain::devices::types::Device;
use crate::domain::devices::validation::ValidatedDeviceInput;
use crate::error::AppError;

pub fn insert_device(
    conn: &Connection,
    input: &ValidatedDeviceInput,
    now: &str,
) -> Result<Device, AppError> {
    conn.execute(
        "INSERT INTO devices (
            customer_id, device_type, manufacturer, model, serial_number,
            accessories, notes, created_at, updated_at, archived_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL)",
        params![
            input.customer_id,
            input.device_type,
            input.manufacturer,
            input.model,
            input.serial_number,
            input.accessories,
            input.notes,
            now,
            now
        ],
    )?;
    let id = conn.last_insert_rowid();
    get_device_by_id(conn, id)?.ok_or(AppError::Internal {
        message: "device missing after insert".into(),
    })
}

pub fn update_device(
    conn: &Connection,
    id: i64,
    input: &ValidatedDeviceInput,
    now: &str,
) -> Result<Device, AppError> {
    let updated = conn.execute(
        "UPDATE devices SET
            device_type = ?1,
            manufacturer = ?2,
            model = ?3,
            serial_number = ?4,
            accessories = ?5,
            notes = ?6,
            updated_at = ?7
         WHERE id = ?8 AND archived_at IS NULL",
        params![
            input.device_type,
            input.manufacturer,
            input.model,
            input.serial_number,
            input.accessories,
            input.notes,
            now,
            id
        ],
    )?;
    if updated == 0 {
        return match get_device_by_id(conn, id)? {
            Some(_) => Err(AppError::Validation {
                field: None,
                message: "Archived devices cannot be edited. Unarchive first.".into(),
            }),
            None => Err(AppError::NotFound),
        };
    }
    get_device_by_id(conn, id)?.ok_or(AppError::NotFound)
}

pub fn get_device_by_id(conn: &Connection, id: i64) -> Result<Option<Device>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, customer_id, device_type, manufacturer, model, serial_number,
                accessories, notes, created_at, updated_at, archived_at
         FROM devices WHERE id = ?1",
    )?;
    let device = stmt
        .query_row(params![id], map_device)
        .optional()?;
    Ok(device)
}

pub fn set_archived_at(
    conn: &Connection,
    id: i64,
    archived_at: Option<&str>,
    now: &str,
) -> Result<Device, AppError> {
    let updated = conn.execute(
        "UPDATE devices SET archived_at = ?1, updated_at = ?2 WHERE id = ?3",
        params![archived_at, now, id],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    get_device_by_id(conn, id)?.ok_or(AppError::NotFound)
}

fn like_pattern(search: Option<&str>) -> Option<String> {
    search
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            let escaped = s
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_");
            format!("%{escaped}%")
        })
}

pub fn list_devices(
    conn: &Connection,
    search: Option<&str>,
    customer_id: Option<i64>,
    include_archived: bool,
    limit: u32,
    offset: u32,
) -> Result<(Vec<Device>, i64), AppError> {
    let pattern = like_pattern(search);

    let mut where_parts: Vec<String> = Vec::new();
    if !include_archived {
        where_parts.push("archived_at IS NULL".into());
    }
    if customer_id.is_some() {
        where_parts.push("customer_id = ?1".into());
    }
    if pattern.is_some() {
        let placeholder = if customer_id.is_some() { "?2" } else { "?1" };
        where_parts.push(format!(
            "(IFNULL(device_type, '') LIKE {placeholder} ESCAPE '\\'
              OR IFNULL(manufacturer, '') LIKE {placeholder} ESCAPE '\\'
              OR IFNULL(model, '') LIKE {placeholder} ESCAPE '\\'
              OR IFNULL(serial_number, '') LIKE {placeholder} ESCAPE '\\')"
        ));
    }

    let where_sql = if where_parts.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_parts.join(" AND "))
    };

    let (limit_ph, offset_ph) = match (customer_id.is_some(), pattern.is_some()) {
        (false, false) => ("?1", "?2"),
        (true, false) | (false, true) => ("?2", "?3"),
        (true, true) => ("?3", "?4"),
    };

    let count_sql = format!("SELECT COUNT(*) FROM devices {where_sql}");
    let list_sql = format!(
        "SELECT id, customer_id, device_type, manufacturer, model, serial_number,
                accessories, notes, created_at, updated_at, archived_at
         FROM devices
         {where_sql}
         ORDER BY updated_at DESC, id DESC
         LIMIT {limit_ph} OFFSET {offset_ph}"
    );

    let total: i64 = match (customer_id, pattern.as_deref()) {
        (None, None) => conn.query_row(&count_sql, [], |row| row.get(0))?,
        (Some(cid), None) => conn.query_row(&count_sql, params![cid], |row| row.get(0))?,
        (None, Some(q)) => conn.query_row(&count_sql, params![q], |row| row.get(0))?,
        (Some(cid), Some(q)) => {
            conn.query_row(&count_sql, params![cid, q], |row| row.get(0))?
        }
    };

    let mut stmt = conn.prepare(&list_sql)?;
    let rows = match (customer_id, pattern.as_deref()) {
        (None, None) => stmt
            .query_map(params![limit, offset], map_device)?
            .collect::<Result<Vec<_>, _>>()?,
        (Some(cid), None) => stmt
            .query_map(params![cid, limit, offset], map_device)?
            .collect::<Result<Vec<_>, _>>()?,
        (None, Some(q)) => stmt
            .query_map(params![q, limit, offset], map_device)?
            .collect::<Result<Vec<_>, _>>()?,
        (Some(cid), Some(q)) => stmt
            .query_map(params![cid, q, limit, offset], map_device)?
            .collect::<Result<Vec<_>, _>>()?,
    };

    Ok((rows, total))
}

fn map_device(row: &rusqlite::Row<'_>) -> rusqlite::Result<Device> {
    Ok(Device {
        id: row.get(0)?,
        customer_id: row.get(1)?,
        device_type: row.get(2)?,
        manufacturer: row.get(3)?,
        model: row.get(4)?,
        serial_number: row.get(5)?,
        accessories: row.get(6)?,
        notes: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        archived_at: row.get(10)?,
    })
}
