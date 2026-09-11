use rusqlite::{params, Connection};
use serde_json::Value;

use super::{i64_opt, s, s_req};
use crate::domain::sync::hlc::Hlc;
use crate::error::AppError;

pub(super) fn upsert_customer(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
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

pub(super) fn upsert_device(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
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

pub(super) fn upsert_repair(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO repairs (
            id, repair_number, customer_id, device_id, company_id, assigned_to_staff_id,
            status, received_at, reported_problem, accessories_received, device_condition,
            diagnosis_notes, work_performed, notes, expected_pickup_at,
            estimate_list_cents, estimate_discount_bps,
            estimate_base_cents, estimate_tax_rate_bps, estimate_tax_cents, estimate_gross_cents,
            ready_at, collected_at, warranty_years, created_at, updated_at, archived_at,
            hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
            ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32
         )
         ON CONFLICT(id) DO UPDATE SET
            assigned_to_staff_id = excluded.assigned_to_staff_id,
            status = excluded.status, reported_problem = excluded.reported_problem,
            accessories_received = excluded.accessories_received,
            device_condition = excluded.device_condition, diagnosis_notes = excluded.diagnosis_notes,
            work_performed = excluded.work_performed, notes = excluded.notes,
            expected_pickup_at = excluded.expected_pickup_at,
            estimate_list_cents = excluded.estimate_list_cents,
            estimate_discount_bps = excluded.estimate_discount_bps,
            estimate_base_cents = excluded.estimate_base_cents,
            estimate_tax_rate_bps = excluded.estimate_tax_rate_bps,
            estimate_tax_cents = excluded.estimate_tax_cents,
            estimate_gross_cents = excluded.estimate_gross_cents,
            ready_at = excluded.ready_at, collected_at = excluded.collected_at,
            warranty_years = excluded.warranty_years,
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
            i64_opt(v, "estimateListCents"),
            i64_opt(v, "estimateDiscountBps"),
            i64_opt(v, "estimateBaseCents"),
            i64_opt(v, "estimateTaxRateBps"),
            i64_opt(v, "estimateTaxCents"),
            i64_opt(v, "estimateGrossCents"),
            s(v, "readyAt"),
            s(v, "collectedAt"),
            i64_opt(v, "warrantyYears"),
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
