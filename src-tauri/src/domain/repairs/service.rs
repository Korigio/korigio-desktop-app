use rusqlite::Connection;
use time::OffsetDateTime;

use crate::db::repository::now_utc_rfc3339;
use crate::db::Db;
use crate::domain::companies;
use crate::domain::customers;
use crate::domain::devices;
use crate::domain::repairs::constants::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};
use crate::domain::repairs::repository;
use crate::domain::repairs::types::{
    CompleteRepairDiagnosisInput, CompleteRepairDiagnosisResult, Repair, RepairInput,
    RepairListQuery, RepairListResult,
};
use crate::domain::repairs::validation::{
    validate_collected_at_date, validate_complete_diagnosis, validate_create_input,
    validate_update_input, validate_work_performed, validate_workflow_status,
};
use crate::domain::settings;
use crate::error::AppError;

pub fn create_repair(conn: &Connection, input: RepairInput) -> Result<Repair, AppError> {
    let validated = validate_create_input(&input)?;
    ensure_customer_accepts_repair(conn, validated.customer_id)?;
    ensure_device_belongs_to_customer(conn, validated.device_id, validated.customer_id)?;
    ensure_company_accepts_repair(conn, validated.company_id)?;

    let now = now_utc_rfc3339()?;
    let year = local_calendar_year()?;

    let tx = conn.unchecked_transaction()?;
    let repair_number = repository::allocate_repair_number(&tx, year)?;
    let repair = repository::insert_repair(&tx, &repair_number, &validated, &now, &now)?;
    tx.commit()?;
    Ok(repair)
}

pub fn update_repair(
    conn: &Connection,
    id: i64,
    input: RepairInput,
) -> Result<Repair, AppError> {
    let existing = get_repair(conn, id)?;
    let validated = validate_update_input(&input, &existing.status)?;

    let now = now_utc_rfc3339()?;
    let ready_at = resolve_ready_at(&existing, &validated.status, &now);
    let collected_at = resolve_collected_at(&existing, &validated.status, &now);

    repository::update_repair(
        conn,
        id,
        &validated,
        ready_at.as_deref(),
        collected_at.as_deref(),
        &now,
    )
}

pub fn get_repair(conn: &Connection, id: i64) -> Result<Repair, AppError> {
    repository::get_repair_by_id(conn, id)?.ok_or(AppError::NotFound)
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

    let (items, total) = repository::list_repairs(
        conn,
        query.query.as_deref(),
        query.customer_id,
        query.device_id,
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

/// Save diagnosis notes / pickup / estimate and apply draft or finalize status rules.
/// Does not upsert `repair_diagnosis` checklist rows (notes-primary).
pub fn complete_repair_diagnosis(
    conn: &Connection,
    input: CompleteRepairDiagnosisInput,
) -> Result<CompleteRepairDiagnosisResult, AppError> {
    let existing = get_repair(conn, input.repair_id)?;
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

    let (base, rate_bps, tax, gross) = match validated.estimate {
        Some(est) => (
            Some(est.base_cents),
            Some(est.tax_rate_bps),
            Some(est.tax_cents),
            Some(est.gross_cents),
        ),
        None => (None, None, None, None),
    };

    let repair = repository::update_repair_diagnosis(
        &tx,
        input.repair_id,
        &validated.next_status,
        validated.diagnosis_notes.as_deref(),
        validated.expected_pickup_at.as_deref(),
        base,
        rate_bps,
        tax,
        gross,
        &now,
    )?;
    tx.commit()?;

    Ok(CompleteRepairDiagnosisResult { repair })
}

pub fn confirm_repair_intake(db: &Db, repair_id: i64) -> Result<Repair, AppError> {
    if repair_id <= 0 {
        return Err(AppError::Validation {
            field: Some("repairId".into()),
            message: "Repair is required.".into(),
        });
    }

    let existing = get_repair(db.conn(), repair_id)?;
    if existing.archived_at.is_some() {
        return Err(AppError::Validation {
            field: None,
            message: "Archived repairs cannot be edited.".into(),
        });
    }
    validate_workflow_status(&existing.status, "received", "confirm intake")?;

    let now = now_utc_rfc3339()?;
    repository::update_repair_status(db.conn(), repair_id, "diagnosis", &now)
}

pub fn confirm_customer_approval(db: &Db, repair_id: i64) -> Result<Repair, AppError> {
    if repair_id <= 0 {
        return Err(AppError::Validation {
            field: Some("repairId".into()),
            message: "Repair is required.".into(),
        });
    }

    let existing = get_repair(db.conn(), repair_id)?;
    if existing.archived_at.is_some() {
        return Err(AppError::Validation {
            field: None,
            message: "Archived repairs cannot be edited.".into(),
        });
    }
    validate_workflow_status(&existing.status, "waiting_customer", "confirm customer approval")?;

    let now = now_utc_rfc3339()?;
    repository::update_repair_status(db.conn(), repair_id, "waiting_part", &now)
}

pub fn confirm_repair_parts_received(db: &Db, repair_id: i64) -> Result<Repair, AppError> {
    if repair_id <= 0 {
        return Err(AppError::Validation {
            field: Some("repairId".into()),
            message: "Repair is required.".into(),
        });
    }

    let existing = get_repair(db.conn(), repair_id)?;
    if existing.archived_at.is_some() {
        return Err(AppError::Validation {
            field: None,
            message: "Archived repairs cannot be edited.".into(),
        });
    }
    validate_workflow_status(&existing.status, "waiting_part", "confirm parts received")?;

    let now = now_utc_rfc3339()?;
    repository::update_repair_status(db.conn(), repair_id, "in_repair", &now)
}

pub fn complete_repair_protocol(
    db: &Db,
    repair_id: i64,
    work_performed: String,
) -> Result<Repair, AppError> {
    if repair_id <= 0 {
        return Err(AppError::Validation {
            field: Some("repairId".into()),
            message: "Repair is required.".into(),
        });
    }

    let work = validate_work_performed(&work_performed)?;
    let existing = get_repair(db.conn(), repair_id)?;
    if existing.archived_at.is_some() {
        return Err(AppError::Validation {
            field: None,
            message: "Archived repairs cannot be edited.".into(),
        });
    }
    validate_workflow_status(&existing.status, "in_repair", "complete the repair protocol")?;

    let now = now_utc_rfc3339()?;
    let ready_at = resolve_ready_at(&existing, "ready", &now);
    repository::update_repair_protocol_complete(
        db.conn(),
        repair_id,
        &work,
        ready_at.as_deref(),
        &now,
    )
}

pub fn confirm_repair_summary(db: &Db, repair_id: i64) -> Result<Repair, AppError> {
    if repair_id <= 0 {
        return Err(AppError::Validation {
            field: Some("repairId".into()),
            message: "Repair is required.".into(),
        });
    }

    let existing = get_repair(db.conn(), repair_id)?;
    if existing.archived_at.is_some() {
        return Err(AppError::Validation {
            field: None,
            message: "Archived repairs cannot be edited.".into(),
        });
    }
    validate_workflow_status(&existing.status, "ready", "confirm summary")?;

    let now = now_utc_rfc3339()?;
    repository::update_repair_status(db.conn(), repair_id, "awaiting_pickup", &now)
}

pub fn complete_repair_pickup(
    db: &Db,
    repair_id: i64,
    collected_at: String,
) -> Result<Repair, AppError> {
    if repair_id <= 0 {
        return Err(AppError::Validation {
            field: Some("repairId".into()),
            message: "Repair is required.".into(),
        });
    }

    let collected_at_rfc3339 = validate_collected_at_date(&collected_at)?;
    let existing = get_repair(db.conn(), repair_id)?;
    if existing.archived_at.is_some() {
        return Err(AppError::Validation {
            field: None,
            message: "Archived repairs cannot be edited.".into(),
        });
    }
    validate_workflow_status(&existing.status, "awaiting_pickup", "complete pickup")?;

    let now = now_utc_rfc3339()?;
    repository::update_repair_pickup_complete(
        db.conn(),
        repair_id,
        &collected_at_rfc3339,
        &now,
    )
}

fn local_calendar_year() -> Result<i32, AppError> {
    OffsetDateTime::now_local()
        .map(|dt| dt.year())
        .map_err(|_| AppError::Internal {
            message: "local calendar year unavailable".into(),
        })
}

fn ensure_customer_accepts_repair(conn: &Connection, customer_id: i64) -> Result<(), AppError> {
    let customer = customers::get_customer(conn, customer_id)?;
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
    device_id: i64,
    customer_id: i64,
) -> Result<(), AppError> {
    let device = devices::get_device(conn, device_id)?;
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

fn ensure_company_accepts_repair(conn: &Connection, company_id: i64) -> Result<(), AppError> {
    let company = companies::get_company(conn, company_id)?;
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
