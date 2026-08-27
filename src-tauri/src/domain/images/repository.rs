use rusqlite::{Connection, OptionalExtension, params};

use crate::domain::images::types::RepairImage;
use crate::domain::images::validation::ValidatedUpdateInput;
use crate::error::AppError;

pub fn list_by_repair_id(
    conn: &Connection,
    repair_id: i64,
) -> Result<Vec<RepairImage>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, repair_id, original_path, thumb_path, caption, sort_order, created_at
         FROM repair_images
         WHERE repair_id = ?1
         ORDER BY sort_order ASC, id ASC",
    )?;
    let rows = stmt
        .query_map(params![repair_id], map_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn count_by_repair_id(conn: &Connection, repair_id: i64) -> Result<i64, AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM repair_images WHERE repair_id = ?1",
        params![repair_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

pub fn next_sort_order(conn: &Connection, repair_id: i64) -> Result<i64, AppError> {
    let max: Option<i64> = conn
        .query_row(
            "SELECT MAX(sort_order) FROM repair_images WHERE repair_id = ?1",
            params![repair_id],
            |row| row.get(0),
        )
        .optional()?
        .flatten();
    Ok(max.map(|v| v + 1).unwrap_or(0))
}

pub fn get_by_id(conn: &Connection, id: i64) -> Result<Option<RepairImage>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, repair_id, original_path, thumb_path, caption, sort_order, created_at
         FROM repair_images WHERE id = ?1",
    )?;
    let row = stmt.query_row(params![id], map_row).optional()?;
    Ok(row)
}

pub fn insert(
    conn: &Connection,
    repair_id: i64,
    original_path: &str,
    thumb_path: Option<&str>,
    sort_order: i64,
    created_at: &str,
) -> Result<RepairImage, AppError> {
    conn.execute(
        "INSERT INTO repair_images (repair_id, original_path, thumb_path, caption, sort_order, created_at)
         VALUES (?1, ?2, ?3, NULL, ?4, ?5)",
        params![repair_id, original_path, thumb_path, sort_order, created_at],
    )?;
    let id = conn.last_insert_rowid();
    get_by_id(conn, id)?.ok_or(AppError::Internal {
        message: "repair image missing after insert".into(),
    })
}

pub fn update(
    conn: &Connection,
    id: i64,
    input: &ValidatedUpdateInput,
) -> Result<RepairImage, AppError> {
    let existing = get_by_id(conn, id)?.ok_or(AppError::NotFound)?;

    let caption = match &input.caption {
        Some(value) => value.clone(),
        None => existing.caption.clone(),
    };
    let sort_order = input.sort_order.unwrap_or(existing.sort_order);

    let updated = conn.execute(
        "UPDATE repair_images SET caption = ?1, sort_order = ?2 WHERE id = ?3",
        params![caption, sort_order, id],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    get_by_id(conn, id)?.ok_or(AppError::NotFound)
}

pub fn delete(conn: &Connection, id: i64) -> Result<(), AppError> {
    let updated = conn.execute("DELETE FROM repair_images WHERE id = ?1", params![id])?;
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
    })
}
