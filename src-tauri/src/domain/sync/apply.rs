//! Apply a remote `sync_changes` row with last-write-wins.

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;

use crate::domain::sync::hlc::{hlc_equal, hlc_greater, Hlc};
use crate::domain::sync::record::{change_exists, insert_change, SyncChange};
use crate::error::AppError;

mod core_rows;
mod team_rows;
mod workflow_rows;

use core_rows::{upsert_customer, upsert_device, upsert_repair};
use team_rows::{
    apply_shop_settings, upsert_company, upsert_staff, upsert_team, upsert_team_device,
    upsert_team_invite,
};
use workflow_rows::{
    upsert_content_blob, upsert_diagnosis_template, upsert_repair_diagnosis,
    upsert_repair_document, upsert_repair_image,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyOutcome {
    Applied,
    IgnoredOlder,
    Idempotent,
}

pub fn apply_remote_change(
    conn: &Connection,
    change: &SyncChange,
) -> Result<ApplyOutcome, AppError> {
    if change_exists(conn, &change.id)? {
        return Ok(ApplyOutcome::Idempotent);
    }

    if change.entity_table == "staff" && change.op == "upsert" {
        reject_unauthorized_role_change(conn, change)?;
    }

    let local = load_local_hlc(conn, &change.entity_table, &change.entity_id)?;
    if let Some(local_hlc) = local {
        if hlc_equal(&local_hlc, &change.hlc) {
            insert_change(
                conn,
                Some(&change.id),
                &change.entity_table,
                &change.entity_id,
                &change.op,
                parse_payload(&change.payload_json)?,
                &change.hlc,
            )?;
            return Ok(ApplyOutcome::Idempotent);
        }
        if hlc_greater(&local_hlc, &change.hlc) {
            insert_change(
                conn,
                Some(&change.id),
                &change.entity_table,
                &change.entity_id,
                &change.op,
                parse_payload(&change.payload_json)?,
                &change.hlc,
            )?;
            return Ok(ApplyOutcome::IgnoredOlder);
        }
    }

    apply_payload(conn, change)?;
    insert_change(
        conn,
        Some(&change.id),
        &change.entity_table,
        &change.entity_id,
        &change.op,
        parse_payload(&change.payload_json)?,
        &change.hlc,
    )?;
    Ok(ApplyOutcome::Applied)
}

fn parse_payload(json: &str) -> Result<Value, AppError> {
    serde_json::from_str(json).map_err(|err| AppError::Internal {
        message: format!("invalid sync payload: {err}"),
    })
}

fn load_local_hlc(
    conn: &Connection,
    table: &str,
    entity_id: &str,
) -> Result<Option<Hlc>, AppError> {
    let sql = match table {
        "customers" => {
            "SELECT hlc_wall_ms, hlc_counter, origin_device_id FROM customers WHERE id = ?1"
        }
        "devices" => {
            "SELECT hlc_wall_ms, hlc_counter, origin_device_id FROM devices WHERE id = ?1"
        }
        "repairs" => {
            "SELECT hlc_wall_ms, hlc_counter, origin_device_id FROM repairs WHERE id = ?1"
        }
        "staff" => "SELECT hlc_wall_ms, hlc_counter, origin_device_id FROM staff WHERE id = ?1",
        "teams" => "SELECT hlc_wall_ms, hlc_counter, origin_device_id FROM teams WHERE id = ?1",
        "team_devices" => {
            "SELECT hlc_wall_ms, hlc_counter, origin_device_id FROM team_devices WHERE id = ?1"
        }
        "team_invites" => {
            "SELECT hlc_wall_ms, hlc_counter, origin_device_id FROM team_invites WHERE id = ?1"
        }
        "companies" => {
            "SELECT hlc_wall_ms, hlc_counter, origin_device_id FROM companies WHERE id = ?1"
        }
        "diagnosis_templates" => {
            "SELECT hlc_wall_ms, hlc_counter, origin_device_id FROM diagnosis_templates WHERE id = ?1"
        }
        "repair_images" => {
            "SELECT hlc_wall_ms, hlc_counter, origin_device_id FROM repair_images WHERE id = ?1"
        }
        "repair_documents" => {
            "SELECT hlc_wall_ms, hlc_counter, origin_device_id FROM repair_documents WHERE id = ?1"
        }
        "repair_diagnosis" => {
            "SELECT hlc_wall_ms, hlc_counter, origin_device_id FROM repair_diagnosis WHERE id = ?1"
        }
        "shop_settings" => return Ok(None),
        "content_blobs" => return Ok(None),
        _ => return Ok(None),
    };
    let row = conn
        .query_row(sql, params![entity_id], |row| {
            Ok(Hlc {
                wall: row.get(0)?,
                counter: row.get(1)?,
                origin_device_id: row.get(2)?,
            })
        })
        .optional()?;
    Ok(row)
}

fn reject_unauthorized_role_change(conn: &Connection, change: &SyncChange) -> Result<(), AppError> {
    let payload = parse_payload(&change.payload_json)?;
    let Some(role) = payload.get("role").and_then(|v| v.as_str()) else {
        return Ok(());
    };
    let actor = payload
        .get("updatedByStaffId")
        .and_then(|v| v.as_str())
        .or(None);
    let Some(actor_id) = actor else {
        return Ok(());
    };
    let existing: Option<String> = conn
        .query_row(
            "SELECT role FROM staff WHERE id = ?1",
            params![change.entity_id],
            |row| row.get(0),
        )
        .optional()?;
    if existing.as_deref() == Some(role) {
        return Ok(());
    }
    let actor_role: Option<String> = conn
        .query_row(
            "SELECT role FROM staff WHERE id = ?1 AND deactivated_at IS NULL",
            params![actor_id],
            |row| row.get(0),
        )
        .optional()?;
    if actor_role.as_deref() != Some("admin") {
        return Err(AppError::forbidden("Only an admin can change staff roles."));
    }
    Ok(())
}

fn apply_payload(conn: &Connection, change: &SyncChange) -> Result<(), AppError> {
    let payload = parse_payload(&change.payload_json)?;
    if change.op == "delete" {
        mark_deleted(conn, &change.entity_table, &change.entity_id, &change.hlc)?;
        return Ok(());
    }
    match change.entity_table.as_str() {
        "customers" => upsert_customer(conn, &payload, &change.hlc),
        "devices" => upsert_device(conn, &payload, &change.hlc),
        "repairs" => upsert_repair(conn, &payload, &change.hlc),
        "staff" => upsert_staff(conn, &payload, &change.hlc),
        "teams" => upsert_team(conn, &payload, &change.hlc),
        "team_devices" => upsert_team_device(conn, &payload, &change.hlc),
        "team_invites" => upsert_team_invite(conn, &payload, &change.hlc),
        "companies" => upsert_company(conn, &payload, &change.hlc),
        "diagnosis_templates" => upsert_diagnosis_template(conn, &payload, &change.hlc),
        "repair_diagnosis" => upsert_repair_diagnosis(conn, &payload, &change.hlc),
        "repair_images" => upsert_repair_image(conn, &payload, &change.hlc),
        "repair_documents" => upsert_repair_document(conn, &payload, &change.hlc),
        "content_blobs" => upsert_content_blob(conn, &payload),
        "shop_settings" => apply_shop_settings(conn, &payload),
        _ => Ok(()),
    }
}

/// Apply a join-snapshot row without appending a new `sync_changes` id.
pub fn apply_snapshot_row(conn: &Connection, table: &str, payload: Value) -> Result<(), AppError> {
    let hlc = hlc_from_payload(&payload);
    let entity_id = s(&payload, "id")
        .or_else(|| s(&payload, "contentHash"))
        .unwrap_or_else(|| table.to_string());
    let payload_json = serde_json::to_string(&payload).map_err(|err| AppError::Internal {
        message: format!("invalid snapshot payload: {err}"),
    })?;
    let change = SyncChange {
        id: String::new(),
        entity_table: table.to_string(),
        entity_id,
        op: "upsert".into(),
        payload_json,
        hlc,
        created_at: crate::db::repository::now_utc_rfc3339().unwrap_or_default(),
    };
    apply_payload(conn, &change)
}

fn hlc_from_payload(v: &Value) -> Hlc {
    Hlc {
        wall: i64_opt(v, "hlcWallMs").unwrap_or(0),
        counter: i64_opt(v, "hlcCounter").unwrap_or(0),
        origin_device_id: s(v, "originDeviceId").unwrap_or_default(),
    }
}

fn mark_deleted(conn: &Connection, table: &str, id: &str, hlc: &Hlc) -> Result<(), AppError> {
    let sql = match table {
        "repair_images" => {
            "UPDATE repair_images SET deleted_at = COALESCE(deleted_at, ?2), hlc_wall_ms = ?3, hlc_counter = ?4, origin_device_id = ?5 WHERE id = ?1"
        }
        "repair_documents" => {
            "UPDATE repair_documents SET deleted_at = COALESCE(deleted_at, ?2), hlc_wall_ms = ?3, hlc_counter = ?4, origin_device_id = ?5 WHERE id = ?1"
        }
        "diagnosis_templates" => {
            "UPDATE diagnosis_templates SET deleted_at = COALESCE(deleted_at, ?2), hlc_wall_ms = ?3, hlc_counter = ?4, origin_device_id = ?5 WHERE id = ?1"
        }
        "staff" => {
            "UPDATE staff SET deleted_at = COALESCE(deleted_at, ?2), hlc_wall_ms = ?3, hlc_counter = ?4, origin_device_id = ?5 WHERE id = ?1"
        }
        _ => return Ok(()),
    };
    let now = crate::db::repository::now_utc_rfc3339()?;
    conn.execute(
        sql,
        params![id, now, hlc.wall, hlc.counter, hlc.origin_device_id],
    )?;
    Ok(())
}

fn s(v: &Value, key: &str) -> Option<String> {
    match v.get(key) {
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Null) | None => None,
        other => other.and_then(|x| x.as_str().map(str::to_string)),
    }
}

fn s_req(v: &Value, key: &str) -> Result<String, AppError> {
    s(v, key).ok_or(AppError::Internal {
        message: format!("sync payload missing {key}"),
    })
}

fn i64_opt(v: &Value, key: &str) -> Option<i64> {
    v.get(key).and_then(|x| x.as_i64())
}
