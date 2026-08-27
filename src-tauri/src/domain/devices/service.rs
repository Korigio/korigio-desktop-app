use rusqlite::Connection;

use crate::db::repository::now_utc_rfc3339;
use crate::domain::customers;
use crate::domain::devices::constants::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};
use crate::domain::devices::repository;
use crate::domain::devices::types::{Device, DeviceInput, DeviceListQuery, DeviceListResult};
use crate::domain::devices::validation::validate_device_input;
use crate::error::AppError;

pub fn create_device(conn: &Connection, input: DeviceInput) -> Result<Device, AppError> {
    let validated = validate_device_input(&input)?;
    ensure_customer_accepts_device(conn, validated.customer_id)?;
    let now = now_utc_rfc3339()?;
    repository::insert_device(conn, &validated, &now)
}

pub fn update_device(
    conn: &Connection,
    id: i64,
    input: DeviceInput,
) -> Result<Device, AppError> {
    let existing = get_device(conn, id)?;
    let mut validated = validate_device_input(&input)?;
    // Do not reassign ownership in Phase 4.
    validated.customer_id = existing.customer_id;
    let now = now_utc_rfc3339()?;
    repository::update_device(conn, id, &validated, &now)
}

pub fn get_device(conn: &Connection, id: i64) -> Result<Device, AppError> {
    repository::get_device_by_id(conn, id)?.ok_or(AppError::NotFound)
}

pub fn list_devices(
    conn: &Connection,
    query: DeviceListQuery,
) -> Result<DeviceListResult, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query
        .page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let include_archived = query.include_archived.unwrap_or(false);
    let offset = (page - 1).saturating_mul(page_size);

    let (items, total) = repository::list_devices(
        conn,
        query.query.as_deref(),
        query.customer_id,
        include_archived,
        page_size,
        offset,
    )?;

    Ok(DeviceListResult {
        items,
        total,
        page,
        page_size,
    })
}

pub fn archive_device(conn: &Connection, id: i64) -> Result<Device, AppError> {
    let existing = get_device(conn, id)?;
    if existing.archived_at.is_some() {
        return Ok(existing);
    }
    let now = now_utc_rfc3339()?;
    repository::set_archived_at(conn, id, Some(&now), &now)
}

pub fn unarchive_device(conn: &Connection, id: i64) -> Result<Device, AppError> {
    let existing = get_device(conn, id)?;
    if existing.archived_at.is_none() {
        return Ok(existing);
    }
    let now = now_utc_rfc3339()?;
    repository::set_archived_at(conn, id, None, &now)
}

fn ensure_customer_accepts_device(conn: &Connection, customer_id: i64) -> Result<(), AppError> {
    let customer = customers::get_customer(conn, customer_id)?;
    if customer.archived_at.is_some() {
        return Err(AppError::Validation {
            field: Some("customerId".into()),
            message: "Cannot add a device to an archived customer. Unarchive the customer first."
                .into(),
        });
    }
    Ok(())
}
