//! Append `sync_changes` after local replicated writes.

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;

use crate::db::repository::now_utc_rfc3339;
use crate::domain::identity;
use crate::domain::ids::new_entity_id;
use crate::domain::sync::hlc::{tick_hlc_value, Hlc};
use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct WriteContext {
    pub hlc: Hlc,
    pub staff_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SyncChange {
    pub id: String,
    pub entity_table: String,
    pub entity_id: String,
    pub op: String,
    pub payload_json: String,
    pub hlc: Hlc,
    pub created_at: String,
}

pub fn begin_write(conn: &Connection) -> Result<WriteContext, AppError> {
    identity::require_write_access(conn)?;
    let hlc = tick_hlc_value(conn)?;
    let staff_id = identity::current_staff_id(conn)?;
    Ok(WriteContext { hlc, staff_id })
}

pub fn record_upsert(
    conn: &Connection,
    table: &str,
    entity_id: &str,
    payload: Value,
    ctx: &WriteContext,
) -> Result<String, AppError> {
    let id = insert_change(conn, None, table, entity_id, "upsert", payload, &ctx.hlc)?;
    crate::domain::sync::notify_local_change();
    Ok(id)
}

pub fn record_delete(
    conn: &Connection,
    table: &str,
    entity_id: &str,
    payload: Value,
    ctx: &WriteContext,
) -> Result<String, AppError> {
    let id = insert_change(conn, None, table, entity_id, "delete", payload, &ctx.hlc)?;
    crate::domain::sync::notify_local_change();
    Ok(id)
}

pub fn insert_change(
    conn: &Connection,
    change_id: Option<&str>,
    table: &str,
    entity_id: &str,
    op: &str,
    payload: Value,
    hlc: &Hlc,
) -> Result<String, AppError> {
    let id = match change_id {
        Some(existing) => existing.to_string(),
        None => new_entity_id(),
    };
    if change_exists(conn, &id)? {
        return Ok(id);
    }
    let now = now_utc_rfc3339()?;
    let json = serde_json::to_string(&payload).map_err(|err| AppError::Internal {
        message: format!("failed to serialize sync payload: {err}"),
    })?;
    conn.execute(
        "INSERT INTO sync_changes (
            id, entity_table, entity_id, op, payload_json,
            hlc_wall_ms, hlc_counter, origin_device_id, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id,
            table,
            entity_id,
            op,
            json,
            hlc.wall,
            hlc.counter,
            hlc.origin_device_id,
            now
        ],
    )?;
    Ok(id)
}

pub fn change_exists(conn: &Connection, id: &str) -> Result<bool, AppError> {
    let found: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM sync_changes WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(found.is_some())
}

pub fn get_change(conn: &Connection, id: &str) -> Result<Option<SyncChange>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, entity_table, entity_id, op, payload_json,
                hlc_wall_ms, hlc_counter, origin_device_id, created_at
         FROM sync_changes WHERE id = ?1",
    )?;
    let row = stmt
        .query_row(params![id], |row| {
            Ok(SyncChange {
                id: row.get(0)?,
                entity_table: row.get(1)?,
                entity_id: row.get(2)?,
                op: row.get(3)?,
                payload_json: row.get(4)?,
                hlc: Hlc {
                    wall: row.get(5)?,
                    counter: row.get(6)?,
                    origin_device_id: row.get(7)?,
                },
                created_at: row.get(8)?,
            })
        })
        .optional()?;
    Ok(row)
}

pub fn changes_after(
    conn: &Connection,
    after: &Hlc,
    limit: i64,
) -> Result<Vec<SyncChange>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, entity_table, entity_id, op, payload_json,
                hlc_wall_ms, hlc_counter, origin_device_id, created_at
         FROM sync_changes
         WHERE hlc_wall_ms > ?1
            OR (hlc_wall_ms = ?1 AND hlc_counter > ?2)
            OR (hlc_wall_ms = ?1 AND hlc_counter = ?2 AND origin_device_id > ?3)
         ORDER BY hlc_wall_ms ASC, hlc_counter ASC, origin_device_id ASC
         LIMIT ?4",
    )?;
    let rows = stmt
        .query_map(
            params![after.wall, after.counter, after.origin_device_id, limit],
            |row| {
                Ok(SyncChange {
                    id: row.get(0)?,
                    entity_table: row.get(1)?,
                    entity_id: row.get(2)?,
                    op: row.get(3)?,
                    payload_json: row.get(4)?,
                    hlc: Hlc {
                        wall: row.get(5)?,
                        counter: row.get(6)?,
                        origin_device_id: row.get(7)?,
                    },
                    created_at: row.get(8)?,
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get_peer_cursor(conn: &Connection, peer_device_id: &str) -> Result<Hlc, AppError> {
    let row = conn
        .query_row(
            "SELECT wall, counter, origin_device_id FROM sync_peer_cursors WHERE peer_device_id = ?1",
            params![peer_device_id],
            |row| {
                Ok(Hlc {
                    wall: row.get(0)?,
                    counter: row.get(1)?,
                    origin_device_id: row.get(2)?,
                })
            },
        )
        .optional()?;
    Ok(row.unwrap_or_else(|| Hlc::new(0, 0, String::new())))
}

pub fn set_peer_cursor(conn: &Connection, peer_device_id: &str, hlc: &Hlc) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO sync_peer_cursors (peer_device_id, wall, counter, origin_device_id)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(peer_device_id) DO UPDATE SET
            wall = excluded.wall,
            counter = excluded.counter,
            origin_device_id = excluded.origin_device_id",
        params![peer_device_id, hlc.wall, hlc.counter, hlc.origin_device_id],
    )?;
    Ok(())
}
