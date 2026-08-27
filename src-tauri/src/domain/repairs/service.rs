use rusqlite::Connection;
use time::OffsetDateTime;

use crate::db::repository::now_utc_rfc3339;
use crate::domain::customers;
use crate::domain::devices;
use crate::domain::repairs::constants::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};
use crate::domain::repairs::repository;
use crate::domain::repairs::types::{Repair, RepairInput, RepairListQuery, RepairListResult};
use crate::domain::repairs::validation::{validate_create_input, validate_update_input};
use crate::error::AppError;

pub fn create_repair(conn: &Connection, input: RepairInput) -> Result<Repair, AppError> {
    let validated = validate_create_input(&input)?;
    ensure_customer_accepts_repair(conn, validated.customer_id)?;
    ensure_device_belongs_to_customer(conn, validated.device_id, validated.customer_id)?;

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
