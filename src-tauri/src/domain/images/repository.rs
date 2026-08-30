use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::images::types::RepairImage;
use crate::domain::images::validation::ValidatedUpdateInput;
use crate::domain::sync::WriteContext;
use crate::error::AppError;

pub fn list_by_repair_id(conn: &Connection, repair_id: &str) -> Result<Vec<RepairImage>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, repair_id, original_path, thumb_path, caption, sort_order, created_at, content_hash
         FROM repair_images
         WHERE repair_id = ?1 AND deleted_at IS NULL
         ORDER BY sort_order ASC, id ASC",
    )?;
    let rows = stmt
        .query_map(params![repair_id], map_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn count_by_repair_id(conn: &Connection, repair_id: &str) -> Result<i64, AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM repair_images WHERE repair_id = ?1 AND deleted_at IS NULL",
        params![repair_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

pub fn next_sort_order(conn: &Connection, repair_id: &str) -> Result<i64, AppError> {
    let max: Option<i64> = conn
        .query_row(
            "SELECT MAX(sort_order) FROM repair_images WHERE repair_id = ?1 AND deleted_at IS NULL",
            params![repair_id],
            |row| row.get(0),
        )
        .optional()?
        .flatten();
    Ok(max.map(|v| v + 1).unwrap_or(0))
}

pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<RepairImage>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, repair_id, original_path, thumb_path, caption, sort_order, created_at, content_hash
         FROM repair_images WHERE id = ?1 AND deleted_at IS NULL",
    )?;
    let row = stmt.query_row(params![id], map_row).optional()?;
    Ok(row)
}

pub fn insert(
    conn: &Connection,
    repair_id: &str,
    original_path: &str,
    thumb_path: Option<&str>,
    content_hash: &str,
    sort_order: i64,
    created_at: &str,
    ctx: &WriteContext,
) -> Result<RepairImage, AppError> {
    let id = crate::domain::ids::new_entity_id();
    conn.execute(
        "INSERT INTO repair_images (
            id, repair_id, original_path, thumb_path, caption, sort_order, created_at, content_hash,
            updated_at, hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, NULL)",
        params![
            id,
            repair_id,
            original_path,
            thumb_path,
            sort_order,
            created_at,
            content_hash,
            created_at,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id
        ],
    )?;
    get_by_id(conn, &id)?.ok_or(AppError::Internal {
        message: "repair image missing after insert".into(),
    })
}

pub fn update(
    conn: &Connection,
    id: &str,
    input: &ValidatedUpdateInput,
    ctx: &WriteContext,
) -> Result<RepairImage, AppError> {
    let existing = get_by_id(conn, id)?.ok_or(AppError::NotFound)?;

    let caption = match &input.caption {
        Some(value) => value.clone(),
        None => existing.caption.clone(),
    };
    let sort_order = input.sort_order.unwrap_or(existing.sort_order);
    let now = crate::db::repository::now_utc_rfc3339()?;

    let updated = conn.execute(
        "UPDATE repair_images SET caption = ?1, sort_order = ?2, updated_at = ?3,
            hlc_wall_ms = ?4, hlc_counter = ?5, origin_device_id = ?6, updated_by_staff_id = ?7
         WHERE id = ?8 AND deleted_at IS NULL",
        params![
            caption,
            sort_order,
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
    get_by_id(conn, id)?.ok_or(AppError::NotFound)
}

pub fn delete(conn: &Connection, id: &str, ctx: &WriteContext) -> Result<(), AppError> {
    let now = crate::db::repository::now_utc_rfc3339()?;
    let updated = conn.execute(
        "UPDATE repair_images SET deleted_at = ?1, updated_at = ?1,
            hlc_wall_ms = ?2, hlc_counter = ?3, origin_device_id = ?4, updated_by_staff_id = ?5
         WHERE id = ?6 AND deleted_at IS NULL",
        params![
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

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RepairImage> {
    Ok(RepairImage {
        id: row.get(0)?,
        repair_id: row.get(1)?,
        original_path: row.get(2)?,
        thumb_path: row.get(3)?,
        caption: row.get(4)?,
        sort_order: row.get(5)?,
        created_at: row.get(6)?,
        content_hash: row.get(7)?,
    })
}
