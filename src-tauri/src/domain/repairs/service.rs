use rusqlite::Connection;
use time::OffsetDateTime;

use crate::db::repository::now_utc_rfc3339;
use crate::db::Db;
use crate::domain::companies;
use crate::domain::customers;
use crate::domain::devices;
use crate::domain::identity;
use crate::domain::ids::{new_entity_id, parse_entity_id, parse_entity_id_field};
use crate::domain::repairs::constants::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};
use crate::domain::repairs::repository;
use crate::domain::repairs::types::{
    CompleteRepairDiagnosisInput, CompleteRepairDiagnosisResult, Repair, RepairInput,
    RepairListQuery, RepairListResult,
};
use crate::domain::repairs::validation::{
    validate_collected_at_date, validate_complete_diagnosis, validate_create_input,
    validate_update_input, validate_warranty_years, validate_work_performed,
    validate_workflow_status,
};
use crate::domain::settings;
use crate::domain::staff;
use crate::domain::sync::{self, begin_write};
use crate::error::AppError;

fn record(conn: &Connection, repair: &Repair, ctx: &sync::WriteContext) -> Result<(), AppError> {
    let payload = serde_json::to_value(repair).map_err(|err| AppError::Internal {
        message: format!("serialize repair: {err}"),
    })?;
    sync::record_upsert(conn, "repairs", &repair.id, payload, ctx)?;
    Ok(())
}

pub fn create_repair(conn: &Connection, input: RepairInput) -> Result<Repair, AppError> {
    let shop = settings::get_shop_settings(conn)?;
    let validated = validate_create_input(&input, &shop.tax_rate_percent)?;
    ensure_customer_accepts_repair(conn, &validated.customer_id)?;
    ensure_device_belongs_to_customer(conn, &validated.device_id, &validated.customer_id)?;
    ensure_company_accepts_repair(conn, &validated.company_id)?;

    let now = now_utc_rfc3339()?;
    let year = local_calendar_year()?;
    let device_code = identity::repair_device_code(conn)?;

    let tx = conn.unchecked_transaction()?;
    let ctx = begin_write(&tx)?;
    let repair_number = repository::allocate_repair_number(&tx, year, &device_code)?;
    let id = new_entity_id();
    let repair = repository::insert_repair(&tx, &id, &repair_number, &validated, &now, &now, &ctx)?;
    record(&tx, &repair, &ctx)?;
    tx.commit()?;
    Ok(repair)
}

pub fn update_repair(
    conn: &Connection,
    id: String,
    input: RepairInput,
) -> Result<Repair, AppError> {
    let id = parse_entity_id(&id)?;
    let existing = get_repair(conn, id.clone())?;
    let validated = validate_update_input(&input, &existing.status)?;

    let now = now_utc_rfc3339()?;
    let ready_at = resolve_ready_at(&existing, &validated.status, &now);
    let collected_at = resolve_collected_at(&existing, &validated.status, &now);
    let ctx = begin_write(conn)?;
    let repair = repository::update_repair(
        conn,
        &id,
        &validated,
        ready_at.as_deref(),
        collected_at.as_deref(),
        &now,
        &ctx,
    )?;
    record(conn, &repair, &ctx)?;
    Ok(repair)
}

pub fn get_repair(conn: &Connection, id: String) -> Result<Repair, AppError> {
    let id = parse_entity_id(&id)?;
    repository::get_repair_by_id(conn, &id)?.ok_or(AppError::NotFound)
}

pub fn list_repairs(
    conn: &Connection,
    query: RepairListQuery,
) -> Result<RepairListResult, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query
        .page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let offset = (page - 1).saturating_mul(page_size);

    let status = query
        .status
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let customer_id = match query.customer_id {
        Some(raw) => Some(parse_entity_id_field(&raw, "customerId")?),
        None => None,
    };
    let device_id = match query.device_id {
        Some(raw) => Some(parse_entity_id_field(&raw, "deviceId")?),
        None => None,
    };
    let company_id = match query.company_id {
        Some(raw) => Some(parse_entity_id_field(&raw, "companyId")?),
        None => None,
    };

    let (items, total) = repository::list_repairs(
        conn,
        query.query.as_deref(),
        customer_id.as_deref(),
        device_id.as_deref(),
        company_id.as_deref(),
        status,
        page_size,
        offset,
    )?;

    Ok(RepairListResult {
        items,
        total,
        page,
        page_size,
    })
}

pub fn complete_repair_diagnosis(
    conn: &Connection,
    input: CompleteRepairDiagnosisInput,
) -> Result<CompleteRepairDiagnosisResult, AppError> {
    let repair_id = parse_entity_id_field(&input.repair_id, "repairId")?;
    let existing = get_repair(conn, repair_id.clone())?;
    if existing.archived_at.is_some() {
        return Err(AppError::Validation {
            field: None,
            message: "Archived repairs cannot be diagnosed.".into(),
        });
    }

    let shop = settings::get_shop_settings(conn)?;
    let validated = validate_complete_diagnosis(
        &input,
        &existing.status,
        existing.estimate_base_cents,
        &shop.tax_rate_percent,
    )?;

    let now = now_utc_rfc3339()?;
    let tx = conn.unchecked_transaction()?;
    let ctx = begin_write(&tx)?;

    let (list, discount, base, rate_bps, tax, gross) = match validated.estimate {
        Some(est) => (
            est.list_cents,
            est.discount_bps,
            Some(est.base_cents),
            Some(est.tax_rate_bps),
            Some(est.tax_cents),
            Some(est.gross_cents),
        ),
        None => (None, None, None, None, None, None),
    };

    let repair = repository::update_repair_diagnosis(
        &tx,
        &repair_id,
        &validated.next_status,
        validated.diagnosis_notes.as_deref(),
        validated.expected_pickup_at.as_deref(),
        list,
        discount,
        base,
        rate_bps,
        tax,
        gross,
        &now,
        &ctx,
    )?;
    record(&tx, &repair, &ctx)?;
    tx.commit()?;

    Ok(CompleteRepairDiagnosisResult { repair })
}

pub fn confirm_repair_intake(db: &Db, repair_id: String) -> Result<Repair, AppError> {
    advance_status(db, &repair_id, "received", "diagnosis", "confirm intake")
}

pub fn confirm_customer_approval(db: &Db, repair_id: String) -> Result<Repair, AppError> {
    advance_status(
        db,
        &repair_id,
        "waiting_customer",
        "waiting_part",
        "confirm customer approval",
    )
}

pub fn confirm_repair_parts_received(db: &Db, repair_id: String) -> Result<Repair, AppError> {
    advance_status(
        db,
        &repair_id,
        "waiting_part",
        "in_repair",
        "confirm parts received",
    )
}

pub fn complete_repair_protocol(
    db: &Db,
    repair_id: String,
    work_performed: String,
) -> Result<Repair, AppError> {
    let repair_id = parse_entity_id_field(&repair_id, "repairId")?;
    let work = validate_work_performed(&work_performed)?;
    let existing = get_repair(db.conn(), repair_id.clone())?;
    ensure_editable(&existing)?;
    validate_workflow_status(
        &existing.status,
        "in_repair",
        "complete the repair protocol",
    )?;

    let now = now_utc_rfc3339()?;
    let ready_at = resolve_ready_at(&existing, "ready", &now);
    let ctx = begin_write(db.conn())?;
    let repair = repository::update_repair_protocol_complete(
        db.conn(),
        &repair_id,
        &work,
        ready_at.as_deref(),
        &now,
        &ctx,
    )?;
    record(db.conn(), &repair, &ctx)?;
    Ok(repair)
}

pub fn confirm_repair_summary(db: &Db, repair_id: String) -> Result<Repair, AppError> {
    advance_status(
        db,
        &repair_id,
        "ready",
        "awaiting_pickup",
        "confirm summary",
    )
}

/// Records collection date and warranty years while status remains `ready` (before summary print / confirm).
/// Allows updating again while still `ready`.
pub fn record_repair_summary_handover(
    db: &Db,
    repair_id: String,
    collected_at: String,
    warranty_years: i64,
) -> Result<Repair, AppError> {
    let repair_id = parse_entity_id_field(&repair_id, "repairId")?;
    let collected_at_rfc3339 = validate_collected_at_date(&collected_at)?;
    let warranty_years = validate_warranty_years(warranty_years)?;
    let existing = get_repair(db.conn(), repair_id.clone())?;
    ensure_editable(&existing)?;
    validate_workflow_status(&existing.status, "ready", "record summary handover")?;

    let now = now_utc_rfc3339()?;
    let ctx = begin_write(db.conn())?;
    let repair = repository::update_repair_summary_handover(
        db.conn(),
        &repair_id,
        &collected_at_rfc3339,
        warranty_years,
        &now,
        &ctx,
    )?;
    record(db.conn(), &repair, &ctx)?;
    Ok(repair)
}

pub fn complete_repair_pickup(
    db: &Db,
    repair_id: String,
    collected_at: String,
) -> Result<Repair, AppError> {
    let repair_id = parse_entity_id_field(&repair_id, "repairId")?;
    let collected_at_rfc3339 = validate_collected_at_date(&collected_at)?;
    let existing = get_repair(db.conn(), repair_id.clone())?;
    ensure_editable(&existing)?;
    validate_workflow_status(&existing.status, "awaiting_pickup", "complete pickup")?;

    let now = now_utc_rfc3339()?;
    let ctx = begin_write(db.conn())?;
    let repair = repository::update_repair_pickup_complete(
        db.conn(),
        &repair_id,
        &collected_at_rfc3339,
        &now,
        &ctx,
    )?;
    record(db.conn(), &repair, &ctx)?;
    Ok(repair)
}

pub fn assign_repair(
    conn: &Connection,
    repair_id: String,
    staff_id: String,
) -> Result<Repair, AppError> {
    let repair_id = parse_entity_id_field(&repair_id, "repairId")?;
    let staff_id = parse_entity_id_field(&staff_id, "staffId")?;
    identity::require_write_access(conn)?;
    let session = staff::require_session(conn)?;
    let _ = session;
    let assignee = staff::get_staff(conn, &staff_id)?;
    if assignee.deactivated_at.is_some() {
        return Err(AppError::Validation {
            field: Some("staffId".into()),
            message: "Cannot assign a deactivated staff member.".into(),
        });
    }
    let existing = get_repair(conn, repair_id.clone())?;
    ensure_editable(&existing)?;
    let now = now_utc_rfc3339()?;
    let ctx = begin_write(conn)?;
    let repair = repository::set_assigned_staff(conn, &repair_id, &staff_id, &now, &ctx)?;
    record(conn, &repair, &ctx)?;
    Ok(repair)
}

pub fn take_over_repair(conn: &Connection, repair_id: String) -> Result<Repair, AppError> {
    let session = staff::require_session(conn)?;
    assign_repair(conn, repair_id, session.staff.id)
}

fn advance_status(
    db: &Db,
    repair_id: &str,
    required: &str,
    next: &str,
    action: &str,
) -> Result<Repair, AppError> {
    let repair_id = parse_entity_id_field(repair_id, "repairId")?;
    let existing = get_repair(db.conn(), repair_id.clone())?;
    ensure_editable(&existing)?;
    validate_workflow_status(&existing.status, required, action)?;
    let now = now_utc_rfc3339()?;
    let ctx = begin_write(db.conn())?;
    let repair = repository::update_repair_status(db.conn(), &repair_id, next, &now, &ctx)?;
    record(db.conn(), &repair, &ctx)?;
    Ok(repair)
}

fn ensure_editable(existing: &Repair) -> Result<(), AppError> {
    if existing.archived_at.is_some() {
        return Err(AppError::Validation {
            field: None,
            message: "Archived repairs cannot be edited.".into(),
        });
    }
    Ok(())
}

fn local_calendar_year() -> Result<i32, AppError> {
    OffsetDateTime::now_local()
        .map(|dt| dt.year())
        .map_err(|_| AppError::Internal {
            message: "local calendar year unavailable".into(),
        })
}

fn ensure_customer_accepts_repair(conn: &Connection, customer_id: &str) -> Result<(), AppError> {
    let customer = customers::get_customer(conn, customer_id.to_string())?;
    if customer.archived_at.is_some() {
        return Err(AppError::Validation {
            field: Some("customerId".into()),
            message: "Cannot create a repair for an archived customer.".into(),
        });
    }
    Ok(())
}

fn ensure_device_belongs_to_customer(
    conn: &Connection,
    device_id: &str,
    customer_id: &str,
) -> Result<(), AppError> {
    let device = devices::get_device(conn, device_id.to_string())?;
    if device.archived_at.is_some() {
        return Err(AppError::Validation {
            field: Some("deviceId".into()),
            message: "Cannot create a repair for an archived device.".into(),
        });
    }
    if device.customer_id != customer_id {
        return Err(AppError::Validation {
            field: Some("deviceId".into()),
            message: "Device does not belong to the selected customer.".into(),
        });
    }
    Ok(())
}

fn ensure_company_accepts_repair(conn: &Connection, company_id: &str) -> Result<(), AppError> {
    let company = companies::get_company(conn, company_id.to_string())?;
    if company.archived_at.is_some() {
        return Err(AppError::Validation {
            field: Some("companyId".into()),
            message: "Cannot create a repair for an archived company.".into(),
        });
    }
    Ok(())
}

fn resolve_ready_at(existing: &Repair, new_status: &str, now: &str) -> Option<String> {
    if new_status == "ready" && existing.ready_at.is_none() {
        Some(now.to_string())
    } else {
        existing.ready_at.clone()
    }
}

fn resolve_collected_at(existing: &Repair, new_status: &str, now: &str) -> Option<String> {
    if new_status == "collected" && existing.collected_at.is_none() {
        Some(now.to_string())
    } else {
        existing.collected_at.clone()
    }
}
