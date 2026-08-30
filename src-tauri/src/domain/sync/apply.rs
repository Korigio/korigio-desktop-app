//! Apply a remote `sync_changes` row with last-write-wins.

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;

use crate::domain::sync::hlc::{hlc_equal, hlc_greater, Hlc};
use crate::domain::sync::record::{change_exists, insert_change, SyncChange};
use crate::error::AppError;

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

fn upsert_customer(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO customers (
            id, name, phone, email, address, notes, created_at, updated_at, archived_at,
            hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name, phone = excluded.phone, email = excluded.email,
            address = excluded.address, notes = excluded.notes, updated_at = excluded.updated_at,
            archived_at = excluded.archived_at, hlc_wall_ms = excluded.hlc_wall_ms,
            hlc_counter = excluded.hlc_counter, origin_device_id = excluded.origin_device_id,
            updated_by_staff_id = excluded.updated_by_staff_id, deleted_at = excluded.deleted_at",
        params![
            s_req(v, "id")?,
            s_req(v, "name")?,
            s(v, "phone"),
            s(v, "email"),
            s(v, "address"),
            s(v, "notes"),
            s(v, "createdAt").unwrap_or_else(|| s(v, "updatedAt").unwrap_or_default()),
            s_req(v, "updatedAt").or_else(|_| s_req(v, "createdAt"))?,
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

fn upsert_device(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO devices (
            id, customer_id, device_type, manufacturer, model, serial_number,
            accessories, notes, created_at, updated_at, archived_at,
            hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
         ON CONFLICT(id) DO UPDATE SET
            device_type = excluded.device_type, manufacturer = excluded.manufacturer,
            model = excluded.model, serial_number = excluded.serial_number,
            accessories = excluded.accessories, notes = excluded.notes,
            updated_at = excluded.updated_at, archived_at = excluded.archived_at,
            hlc_wall_ms = excluded.hlc_wall_ms, hlc_counter = excluded.hlc_counter,
            origin_device_id = excluded.origin_device_id,
            updated_by_staff_id = excluded.updated_by_staff_id, deleted_at = excluded.deleted_at",
        params![
            s_req(v, "id")?,
            s_req(v, "customerId")?,
            s(v, "deviceType"),
            s(v, "manufacturer"),
            s(v, "model"),
            s(v, "serialNumber"),
            s(v, "accessories"),
            s(v, "notes"),
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

fn upsert_repair(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO repairs (
            id, repair_number, customer_id, device_id, company_id, assigned_to_staff_id,
            status, received_at, reported_problem, accessories_received, device_condition,
            diagnosis_notes, work_performed, notes, expected_pickup_at,
            estimate_base_cents, estimate_tax_rate_bps, estimate_tax_cents, estimate_gross_cents,
            ready_at, collected_at, created_at, updated_at, archived_at,
            hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
            ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29
         )
         ON CONFLICT(id) DO UPDATE SET
            assigned_to_staff_id = excluded.assigned_to_staff_id,
            status = excluded.status, reported_problem = excluded.reported_problem,
            accessories_received = excluded.accessories_received,
            device_condition = excluded.device_condition, diagnosis_notes = excluded.diagnosis_notes,
            work_performed = excluded.work_performed, notes = excluded.notes,
            expected_pickup_at = excluded.expected_pickup_at,
            estimate_base_cents = excluded.estimate_base_cents,
            estimate_tax_rate_bps = excluded.estimate_tax_rate_bps,
            estimate_tax_cents = excluded.estimate_tax_cents,
            estimate_gross_cents = excluded.estimate_gross_cents,
            ready_at = excluded.ready_at, collected_at = excluded.collected_at,
            updated_at = excluded.updated_at, archived_at = excluded.archived_at,
            hlc_wall_ms = excluded.hlc_wall_ms, hlc_counter = excluded.hlc_counter,
            origin_device_id = excluded.origin_device_id,
            updated_by_staff_id = excluded.updated_by_staff_id, deleted_at = excluded.deleted_at",
        params![
            s_req(v, "id")?,
            s_req(v, "repairNumber")?,
            s_req(v, "customerId")?,
            s_req(v, "deviceId")?,
            s(v, "companyId"),
            s(v, "assignedToStaffId"),
            s_req(v, "status")?,
            s(v, "receivedAt").unwrap_or_default(),
            s(v, "reportedProblem"),
            s(v, "accessoriesReceived"),
            s(v, "deviceCondition"),
            s(v, "diagnosisNotes"),
            s(v, "workPerformed"),
            s(v, "notes"),
            s(v, "expectedPickupAt"),
            i64_opt(v, "estimateBaseCents"),
            i64_opt(v, "estimateTaxRateBps"),
            i64_opt(v, "estimateTaxCents"),
            i64_opt(v, "estimateGrossCents"),
            s(v, "readyAt"),
            s(v, "collectedAt"),
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

fn upsert_staff(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
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

fn upsert_team(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
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

fn upsert_team_device(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
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

fn upsert_team_invite(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
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

fn upsert_company(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
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

fn apply_shop_settings(conn: &Connection, v: &Value) -> Result<(), AppError> {
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

fn json_field(v: &Value, object_key: &str, raw_key: &str) -> Result<String, AppError> {
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

fn document_type_slug(v: &Value) -> Result<String, AppError> {
    let raw = s_req(v, "documentType")?;
    Ok(match raw.as_str() {
        "entranceSigned" => "entrance_signed".into(),
        "diagnosisSigned" => "diagnosis_signed".into(),
        "summarySigned" => "summary_signed".into(),
        other => other.to_string(),
    })
}

fn upsert_diagnosis_template(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
    let body_json = json_field(v, "body", "bodyJson")?;
    conn.execute(
        "INSERT INTO diagnosis_templates (
            id, name, body_json, created_at, updated_at,
            hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name, body_json = excluded.body_json,
            updated_at = excluded.updated_at, hlc_wall_ms = excluded.hlc_wall_ms,
            hlc_counter = excluded.hlc_counter, origin_device_id = excluded.origin_device_id,
            updated_by_staff_id = excluded.updated_by_staff_id, deleted_at = excluded.deleted_at",
        params![
            s_req(v, "id")?,
            s_req(v, "name")?,
            body_json,
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

fn upsert_repair_diagnosis(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
    let result_json = json_field(v, "result", "resultJson")?;
    conn.execute(
        "INSERT INTO repair_diagnosis (
            id, repair_id, template_id, result_json, created_at, updated_at,
            hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
         ON CONFLICT(id) DO UPDATE SET
            template_id = excluded.template_id, result_json = excluded.result_json,
            updated_at = excluded.updated_at, hlc_wall_ms = excluded.hlc_wall_ms,
            hlc_counter = excluded.hlc_counter, origin_device_id = excluded.origin_device_id,
            updated_by_staff_id = excluded.updated_by_staff_id, deleted_at = excluded.deleted_at",
        params![
            s_req(v, "id")?,
            s_req(v, "repairId")?,
            s(v, "templateId"),
            result_json,
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

fn upsert_repair_image(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO repair_images (
            id, repair_id, original_path, thumb_path, caption, sort_order, created_at,
            content_hash, updated_at, hlc_wall_ms, hlc_counter, origin_device_id,
            updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
         ON CONFLICT(id) DO UPDATE SET
            original_path = excluded.original_path, thumb_path = excluded.thumb_path,
            caption = excluded.caption, sort_order = excluded.sort_order,
            content_hash = excluded.content_hash, updated_at = excluded.updated_at,
            hlc_wall_ms = excluded.hlc_wall_ms, hlc_counter = excluded.hlc_counter,
            origin_device_id = excluded.origin_device_id,
            updated_by_staff_id = excluded.updated_by_staff_id, deleted_at = excluded.deleted_at",
        params![
            s_req(v, "id")?,
            s_req(v, "repairId")?,
            s_req(v, "originalPath")?,
            s(v, "thumbPath"),
            s(v, "caption"),
            i64_opt(v, "sortOrder").unwrap_or(0),
            s(v, "createdAt").unwrap_or_default(),
            s(v, "contentHash").unwrap_or_default(),
            s(v, "updatedAt").unwrap_or_else(|| s(v, "createdAt").unwrap_or_default()),
            hlc.wall,
            hlc.counter,
            hlc.origin_device_id,
            s(v, "updatedByStaffId"),
            s(v, "deletedAt"),
        ],
    )?;
    Ok(())
}

fn upsert_repair_document(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO repair_documents (
            id, repair_id, document_type, file_path, original_filename, content_hash,
            created_at, updated_at, hlc_wall_ms, hlc_counter, origin_device_id,
            updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
         ON CONFLICT(id) DO UPDATE SET
            file_path = excluded.file_path, original_filename = excluded.original_filename,
            content_hash = excluded.content_hash, updated_at = excluded.updated_at,
            hlc_wall_ms = excluded.hlc_wall_ms, hlc_counter = excluded.hlc_counter,
            origin_device_id = excluded.origin_device_id,
            updated_by_staff_id = excluded.updated_by_staff_id, deleted_at = excluded.deleted_at",
        params![
            s_req(v, "id")?,
            s_req(v, "repairId")?,
            document_type_slug(v)?,
            s_req(v, "filePath")?,
            s(v, "originalFilename").unwrap_or_else(|| "document".into()),
            s(v, "contentHash").unwrap_or_default(),
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

fn upsert_content_blob(conn: &Connection, v: &Value) -> Result<(), AppError> {
    let hash = s(v, "contentHash")
        .or_else(|| s(v, "id"))
        .ok_or(AppError::Internal {
            message: "sync payload missing contentHash".into(),
        })?;
    let kind = s(v, "kind").unwrap_or_else(|| "document".into());
    let created_at = s(v, "createdAt").unwrap_or_default();
    crate::domain::sync::blobs::upsert_blob_row(
        conn,
        &hash,
        i64_opt(v, "byteSize").unwrap_or(0),
        &kind,
        &created_at,
    )
}
