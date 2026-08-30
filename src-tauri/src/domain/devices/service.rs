use rusqlite::Connection;

use crate::db::repository::now_utc_rfc3339;
use crate::domain::customers;
use crate::domain::devices::constants::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};
use crate::domain::devices::repository;
use crate::domain::devices::types::{Device, DeviceInput, DeviceListQuery, DeviceListResult};
use crate::domain::devices::validation::validate_device_input;
use crate::domain::ids::{new_entity_id, parse_entity_id};
use crate::domain::sync::{self, begin_write};
use crate::error::AppError;

fn record(conn: &Connection, device: &Device, ctx: &sync::WriteContext) -> Result<(), AppError> {
    let payload = serde_json::to_value(device).map_err(|err| AppError::Internal {
        message: format!("serialize device: {err}"),
    })?;
    sync::record_upsert(conn, "devices", &device.id, payload, ctx)?;
    Ok(())
}

pub fn create_device(conn: &Connection, input: DeviceInput) -> Result<Device, AppError> {
    let validated = validate_device_input(&input)?;
    ensure_customer_accepts_device(conn, &validated.customer_id)?;
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let id = new_entity_id();
    let device = repository::insert_device(conn, &id, &validated, &now, &ctx)?;
    record(conn, &device, &ctx)?;
    Ok(device)
}

pub fn update_device(
    conn: &Connection,
    id: String,
    input: DeviceInput,
) -> Result<Device, AppError> {
    let id = parse_entity_id(&id)?;
    let existing = get_device(conn, id.clone())?;
    let mut validated = validate_device_input(&input)?;
    validated.customer_id = existing.customer_id;
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let device = repository::update_device(conn, &id, &validated, &now, &ctx)?;
    record(conn, &device, &ctx)?;
    Ok(device)
}

pub fn get_device(conn: &Connection, id: String) -> Result<Device, AppError> {
    let id = parse_entity_id(&id)?;
    repository::get_device_by_id(conn, &id)?.ok_or(AppError::NotFound)
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
    let customer_id = match query.customer_id {
        Some(raw) => Some(parse_entity_id_field_opt(&raw, "customerId")?),
        None => None,
    };

    let (items, total) = repository::list_devices(
        conn,
        query.query.as_deref(),
        customer_id.as_deref(),
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

pub fn archive_device(conn: &Connection, id: String) -> Result<Device, AppError> {
    let existing = get_device(conn, id.clone())?;
    if existing.archived_at.is_some() {
        return Ok(existing);
    }
    let id = parse_entity_id(&id)?;
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let device = repository::set_archived_at(conn, &id, Some(&now), &now, &ctx)?;
    record(conn, &device, &ctx)?;
    Ok(device)
}

pub fn unarchive_device(conn: &Connection, id: String) -> Result<Device, AppError> {
    let existing = get_device(conn, id.clone())?;
    if existing.archived_at.is_none() {
        return Ok(existing);
    }
    let id = parse_entity_id(&id)?;
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let device = repository::set_archived_at(conn, &id, None, &now, &ctx)?;
    record(conn, &device, &ctx)?;
    Ok(device)
}

fn ensure_customer_accepts_device(conn: &Connection, customer_id: &str) -> Result<(), AppError> {
    let customer = customers::get_customer(conn, customer_id.to_string())?;
    if customer.archived_at.is_some() {
        return Err(AppError::Validation {
            field: Some("customerId".into()),
            message: "Cannot add a device to an archived customer. Unarchive the customer first."
                .into(),
        });
    }
    Ok(())
}

fn parse_entity_id_field_opt(s: &str, field: &str) -> Result<String, AppError> {
    crate::domain::ids::parse_entity_id_field(s, field)
}
