//! Join snapshot dump in FK order.

use rusqlite::Connection;
use serde_json::{json, Value};

use crate::domain::settings::{get_shop_settings, types::ShopSettings, types::SHOP_SETTINGS_ID};
use crate::error::AppError;
pub const SNAPSHOT_SCHEMA_VERSION: i64 = 10;

pub const SNAPSHOT_TABLES: &[&str] = &[
    "teams",
    "staff",
    "team_devices",
    "team_invites",
    "companies",
    "customers",
    "devices",
    "diagnosis_templates",
    "repairs",
    "repair_diagnosis",
    "repair_images",
    "repair_documents",
    "content_blobs",
    "shop_settings",
];

#[derive(Debug, Clone)]
pub struct SnapshotRow {
    pub table: String,
    pub payload: Value,
}

pub fn dump_snapshot(conn: &Connection) -> Result<(i64, Vec<SnapshotRow>), AppError> {
    let mut rows = Vec::new();
    for table in SNAPSHOT_TABLES {
        rows.extend(dump_table(conn, table)?);
    }
    Ok((SNAPSHOT_SCHEMA_VERSION, rows))
}

fn dump_table(conn: &Connection, table: &str) -> Result<Vec<SnapshotRow>, AppError> {
    match table {
        "teams" => dump_simple(
            conn,
            table,
            "SELECT id, name, created_at, updated_at, hlc_wall_ms, hlc_counter,
                    origin_device_id, updated_by_staff_id, deleted_at, pin_hash
             FROM teams",
            |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "createdAt": row.get::<_, String>(2)?,
                    "updatedAt": row.get::<_, String>(3)?,
                    "hlcWallMs": row.get::<_, i64>(4)?,
                    "hlcCounter": row.get::<_, i64>(5)?,
                    "originDeviceId": row.get::<_, String>(6)?,
                    "updatedByStaffId": row.get::<_, Option<String>>(7)?,
                    "deletedAt": row.get::<_, Option<String>>(8)?,
                    "pinHash": row.get::<_, Option<String>>(9)?,
                }))
            },
        ),
        "staff" => dump_simple(
            conn,
            table,
            "SELECT id, team_id, name, role, pin_salt, pin_hash, deactivated_at,
                    created_at, updated_at, hlc_wall_ms, hlc_counter,
                    origin_device_id, updated_by_staff_id, deleted_at
             FROM staff",
            |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "teamId": row.get::<_, Option<String>>(1)?,
                    "name": row.get::<_, String>(2)?,
                    "role": row.get::<_, String>(3)?,
                    "pinSalt": row.get::<_, String>(4)?,
                    "pinHash": row.get::<_, String>(5)?,
                    "deactivatedAt": row.get::<_, Option<String>>(6)?,
                    "createdAt": row.get::<_, String>(7)?,
                    "updatedAt": row.get::<_, String>(8)?,
                    "hlcWallMs": row.get::<_, i64>(9)?,
                    "hlcCounter": row.get::<_, i64>(10)?,
                    "originDeviceId": row.get::<_, String>(11)?,
                    "updatedByStaffId": row.get::<_, Option<String>>(12)?,
                    "deletedAt": row.get::<_, Option<String>>(13)?,
                }))
            },
        ),
        "team_devices" => dump_simple(
            conn,
            table,
            "SELECT id, team_id, device_code, device_name, joined_at, removed_at,
                    created_at, updated_at, hlc_wall_ms, hlc_counter,
                    origin_device_id, updated_by_staff_id, deleted_at
             FROM team_devices",
            |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "teamId": row.get::<_, String>(1)?,
                    "deviceCode": row.get::<_, String>(2)?,
                    "deviceName": row.get::<_, String>(3)?,
                    "joinedAt": row.get::<_, String>(4)?,
                    "removedAt": row.get::<_, Option<String>>(5)?,
                    "createdAt": row.get::<_, String>(6)?,
                    "updatedAt": row.get::<_, String>(7)?,
                    "hlcWallMs": row.get::<_, i64>(8)?,
                    "hlcCounter": row.get::<_, i64>(9)?,
                    "originDeviceId": row.get::<_, String>(10)?,
                    "updatedByStaffId": row.get::<_, Option<String>>(11)?,
                    "deletedAt": row.get::<_, Option<String>>(12)?,
                }))
            },
        ),
        "team_invites" => dump_simple(
            conn,
            table,
            "SELECT id, team_id, code_hash, created_by_staff_id, expires_at, revoked_at,
                    created_at, updated_at, hlc_wall_ms, hlc_counter,
                    origin_device_id, updated_by_staff_id, deleted_at
             FROM team_invites",
            |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "teamId": row.get::<_, String>(1)?,
                    "codeHash": row.get::<_, String>(2)?,
                    "createdByStaffId": row.get::<_, String>(3)?,
                    "expiresAt": row.get::<_, String>(4)?,
                    "revokedAt": row.get::<_, Option<String>>(5)?,
                    "createdAt": row.get::<_, String>(6)?,
                    "updatedAt": row.get::<_, String>(7)?,
                    "hlcWallMs": row.get::<_, i64>(8)?,
                    "hlcCounter": row.get::<_, i64>(9)?,
                    "originDeviceId": row.get::<_, String>(10)?,
                    "updatedByStaffId": row.get::<_, Option<String>>(11)?,
                    "deletedAt": row.get::<_, Option<String>>(12)?,
                }))
            },
        ),
        "companies" => dump_simple(
            conn,
            table,
            "SELECT id, legal_name, trade_name, tax_id, address, phone, email, website,
                    logo_path, logo_content_hash, is_default, created_at, updated_at,
                    archived_at, hlc_wall_ms, hlc_counter, origin_device_id,
                    updated_by_staff_id, deleted_at
             FROM companies",
            |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "legalName": row.get::<_, String>(1)?,
                    "tradeName": row.get::<_, Option<String>>(2)?,
                    "taxId": row.get::<_, Option<String>>(3)?,
                    "address": row.get::<_, Option<String>>(4)?,
                    "phone": row.get::<_, Option<String>>(5)?,
                    "email": row.get::<_, Option<String>>(6)?,
                    "website": row.get::<_, Option<String>>(7)?,
                    "logoPath": row.get::<_, Option<String>>(8)?,
                    "logoContentHash": row.get::<_, Option<String>>(9)?,
                    "isDefault": row.get::<_, i64>(10)? != 0,
                    "createdAt": row.get::<_, String>(11)?,
                    "updatedAt": row.get::<_, String>(12)?,
                    "archivedAt": row.get::<_, Option<String>>(13)?,
                    "hlcWallMs": row.get::<_, i64>(14)?,
                    "hlcCounter": row.get::<_, i64>(15)?,
                    "originDeviceId": row.get::<_, String>(16)?,
                    "updatedByStaffId": row.get::<_, Option<String>>(17)?,
                    "deletedAt": row.get::<_, Option<String>>(18)?,
                }))
            },
        ),
        "customers" => dump_simple(
            conn,
            table,
            "SELECT id, name, phone, email, address, notes, created_at, updated_at,
                    archived_at, hlc_wall_ms, hlc_counter, origin_device_id,
                    updated_by_staff_id, deleted_at
             FROM customers",
            |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "phone": row.get::<_, Option<String>>(2)?,
                    "email": row.get::<_, Option<String>>(3)?,
                    "address": row.get::<_, Option<String>>(4)?,
                    "notes": row.get::<_, Option<String>>(5)?,
                    "createdAt": row.get::<_, String>(6)?,
                    "updatedAt": row.get::<_, String>(7)?,
                    "archivedAt": row.get::<_, Option<String>>(8)?,
                    "hlcWallMs": row.get::<_, i64>(9)?,
                    "hlcCounter": row.get::<_, i64>(10)?,
                    "originDeviceId": row.get::<_, String>(11)?,
                    "updatedByStaffId": row.get::<_, Option<String>>(12)?,
                    "deletedAt": row.get::<_, Option<String>>(13)?,
                }))
            },
        ),
        "devices" => dump_simple(
            conn,
            table,
            "SELECT id, customer_id, device_type, manufacturer, model, serial_number,
                    accessories, notes, created_at, updated_at, archived_at,
                    hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
             FROM devices",
            |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "customerId": row.get::<_, String>(1)?,
                    "deviceType": row.get::<_, Option<String>>(2)?,
                    "manufacturer": row.get::<_, Option<String>>(3)?,
                    "model": row.get::<_, Option<String>>(4)?,
                    "serialNumber": row.get::<_, Option<String>>(5)?,
                    "accessories": row.get::<_, Option<String>>(6)?,
                    "notes": row.get::<_, Option<String>>(7)?,
                    "createdAt": row.get::<_, String>(8)?,
                    "updatedAt": row.get::<_, String>(9)?,
                    "archivedAt": row.get::<_, Option<String>>(10)?,
                    "hlcWallMs": row.get::<_, i64>(11)?,
                    "hlcCounter": row.get::<_, i64>(12)?,
                    "originDeviceId": row.get::<_, String>(13)?,
                    "updatedByStaffId": row.get::<_, Option<String>>(14)?,
                    "deletedAt": row.get::<_, Option<String>>(15)?,
                }))
            },
        ),
        "diagnosis_templates" => dump_simple(
            conn,
            table,
            "SELECT id, name, body_json, created_at, updated_at, hlc_wall_ms, hlc_counter,
                    origin_device_id, updated_by_staff_id, deleted_at
             FROM diagnosis_templates",
            |row| {
                let body_json: String = row.get(2)?;
                let body: Value = serde_json::from_str(&body_json).unwrap_or(Value::Null);
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "body": body,
                    "createdAt": row.get::<_, String>(3)?,
                    "updatedAt": row.get::<_, String>(4)?,
                    "hlcWallMs": row.get::<_, i64>(5)?,
                    "hlcCounter": row.get::<_, i64>(6)?,
                    "originDeviceId": row.get::<_, String>(7)?,
                    "updatedByStaffId": row.get::<_, Option<String>>(8)?,
                    "deletedAt": row.get::<_, Option<String>>(9)?,
                }))
            },
        ),
        "repairs" => dump_simple(
            conn,
            table,
            "SELECT id, repair_number, customer_id, device_id, company_id, assigned_to_staff_id,
                    status, received_at, reported_problem, accessories_received, device_condition,
                    diagnosis_notes, work_performed, notes, expected_pickup_at,
                    estimate_list_cents, estimate_discount_bps,
                    estimate_base_cents, estimate_tax_rate_bps, estimate_tax_cents,
                    estimate_gross_cents, ready_at, collected_at, warranty_years, created_at, updated_at,
                    archived_at, hlc_wall_ms, hlc_counter, origin_device_id,
                    updated_by_staff_id, deleted_at
             FROM repairs",
            |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "repairNumber": row.get::<_, String>(1)?,
                    "customerId": row.get::<_, String>(2)?,
                    "deviceId": row.get::<_, String>(3)?,
                    "companyId": row.get::<_, Option<String>>(4)?,
                    "assignedToStaffId": row.get::<_, Option<String>>(5)?,
                    "status": row.get::<_, String>(6)?,
                    "receivedAt": row.get::<_, String>(7)?,
                    "reportedProblem": row.get::<_, Option<String>>(8)?,
                    "accessoriesReceived": row.get::<_, Option<String>>(9)?,
                    "deviceCondition": row.get::<_, Option<String>>(10)?,
                    "diagnosisNotes": row.get::<_, Option<String>>(11)?,
                    "workPerformed": row.get::<_, Option<String>>(12)?,
                    "notes": row.get::<_, Option<String>>(13)?,
                    "expectedPickupAt": row.get::<_, Option<String>>(14)?,
                    "estimateListCents": row.get::<_, Option<i64>>(15)?,
                    "estimateDiscountBps": row.get::<_, Option<i64>>(16)?,
                    "estimateBaseCents": row.get::<_, Option<i64>>(17)?,
                    "estimateTaxRateBps": row.get::<_, Option<i64>>(18)?,
                    "estimateTaxCents": row.get::<_, Option<i64>>(19)?,
                    "estimateGrossCents": row.get::<_, Option<i64>>(20)?,
                    "readyAt": row.get::<_, Option<String>>(21)?,
                    "collectedAt": row.get::<_, Option<String>>(22)?,
                    "warrantyYears": row.get::<_, Option<i64>>(23)?,
                    "createdAt": row.get::<_, String>(24)?,
                    "updatedAt": row.get::<_, String>(25)?,
                    "archivedAt": row.get::<_, Option<String>>(26)?,
                    "hlcWallMs": row.get::<_, i64>(27)?,
                    "hlcCounter": row.get::<_, i64>(28)?,
                    "originDeviceId": row.get::<_, String>(29)?,
                    "updatedByStaffId": row.get::<_, Option<String>>(30)?,
                    "deletedAt": row.get::<_, Option<String>>(31)?,
                }))
            },
        ),
        "repair_diagnosis" => dump_simple(
            conn,
            table,
            "SELECT id, repair_id, template_id, result_json, created_at, updated_at,
                    hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
             FROM repair_diagnosis",
            |row| {
                let result_json: String = row.get(3)?;
                let result: Value = serde_json::from_str(&result_json).unwrap_or(Value::Null);
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "repairId": row.get::<_, String>(1)?,
                    "templateId": row.get::<_, Option<String>>(2)?,
                    "result": result,
                    "createdAt": row.get::<_, String>(4)?,
                    "updatedAt": row.get::<_, String>(5)?,
                    "hlcWallMs": row.get::<_, i64>(6)?,
                    "hlcCounter": row.get::<_, i64>(7)?,
                    "originDeviceId": row.get::<_, String>(8)?,
                    "updatedByStaffId": row.get::<_, Option<String>>(9)?,
                    "deletedAt": row.get::<_, Option<String>>(10)?,
                }))
            },
        ),
        "repair_images" => dump_simple(
            conn,
            table,
            "SELECT id, repair_id, original_path, thumb_path, caption, sort_order, created_at,
                    content_hash, updated_at, hlc_wall_ms, hlc_counter, origin_device_id,
                    updated_by_staff_id, deleted_at
             FROM repair_images",
            |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "repairId": row.get::<_, String>(1)?,
                    "originalPath": row.get::<_, String>(2)?,
                    "thumbPath": row.get::<_, Option<String>>(3)?,
                    "caption": row.get::<_, Option<String>>(4)?,
                    "sortOrder": row.get::<_, i64>(5)?,
                    "createdAt": row.get::<_, String>(6)?,
                    "contentHash": row.get::<_, String>(7)?,
                    "updatedAt": row.get::<_, String>(8)?,
                    "hlcWallMs": row.get::<_, i64>(9)?,
                    "hlcCounter": row.get::<_, i64>(10)?,
                    "originDeviceId": row.get::<_, String>(11)?,
                    "updatedByStaffId": row.get::<_, Option<String>>(12)?,
                    "deletedAt": row.get::<_, Option<String>>(13)?,
                }))
            },
        ),
        "repair_documents" => dump_simple(
            conn,
            table,
            "SELECT id, repair_id, document_type, file_path, original_filename, content_hash,
                    created_at, updated_at, hlc_wall_ms, hlc_counter, origin_device_id,
                    updated_by_staff_id, deleted_at
             FROM repair_documents",
            |row| {
                Ok(json!({
                    "id": row.get::<_, String>(0)?,
                    "repairId": row.get::<_, String>(1)?,
                    "documentType": row.get::<_, String>(2)?,
                    "filePath": row.get::<_, String>(3)?,
                    "originalFilename": row.get::<_, String>(4)?,
                    "contentHash": row.get::<_, String>(5)?,
                    "createdAt": row.get::<_, String>(6)?,
                    "updatedAt": row.get::<_, String>(7)?,
                    "hlcWallMs": row.get::<_, i64>(8)?,
                    "hlcCounter": row.get::<_, i64>(9)?,
                    "originDeviceId": row.get::<_, String>(10)?,
                    "updatedByStaffId": row.get::<_, Option<String>>(11)?,
                    "deletedAt": row.get::<_, Option<String>>(12)?,
                }))
            },
        ),
        "content_blobs" => dump_simple(
            conn,
            table,
            "SELECT content_hash, byte_size, kind, created_at FROM content_blobs",
            |row| {
                let hash: String = row.get(0)?;
                Ok(json!({
                    "id": hash.clone(),
                    "contentHash": hash,
                    "byteSize": row.get::<_, i64>(1)?,
                    "kind": row.get::<_, String>(2)?,
                    "createdAt": row.get::<_, String>(3)?,
                }))
            },
        ),
        "shop_settings" => {
            let settings = get_shop_settings(conn)?;
            Ok(vec![shop_settings_row(&settings)])
        }
        _ => Ok(Vec::new()),
    }
}

pub fn shop_settings_row(settings: &ShopSettings) -> SnapshotRow {
    SnapshotRow {
        table: "shop_settings".into(),
        payload: json!({
            "id": SHOP_SETTINGS_ID,
            "taxRatePercent": settings.tax_rate_percent,
            "currency": settings.currency,
        }),
    }
}

pub fn list_blob_hashes(conn: &Connection) -> Result<Vec<String>, AppError> {
    let mut stmt = conn.prepare("SELECT content_hash FROM content_blobs")?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn dump_simple<F>(
    conn: &Connection,
    table: &str,
    sql: &str,
    map: F,
) -> Result<Vec<SnapshotRow>, AppError>
where
    F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<Value>,
{
    let mut stmt = conn.prepare(sql)?;
    let mapped = stmt.query_map([], map)?.collect::<Result<Vec<_>, _>>()?;
    Ok(mapped
        .into_iter()
        .map(|payload| SnapshotRow {
            table: table.to_string(),
            payload,
        })
        .collect())
}
