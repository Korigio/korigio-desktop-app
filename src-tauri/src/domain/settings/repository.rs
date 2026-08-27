use rusqlite::{Connection, OptionalExtension, params};

use crate::error::AppError;

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, AppError> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let value = stmt
        .query_row(params![key], |row| row.get::<_, String>(0))
        .optional()?;
    Ok(value)
}

pub fn upsert_setting(conn: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}
