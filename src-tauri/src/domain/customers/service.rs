use rusqlite::Connection;

use crate::db::repository::now_utc_rfc3339;
use crate::domain::customers::constants::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};
use crate::domain::customers::repository;
use crate::domain::customers::types::{
    Customer, CustomerInput, CustomerListQuery, CustomerListResult,
};
use crate::domain::customers::validation::validate_customer_input;
use crate::domain::ids::{new_entity_id, parse_entity_id};
use crate::domain::sync::{self, begin_write};
use crate::error::AppError;

fn record(
    conn: &Connection,
    customer: &Customer,
    ctx: &sync::WriteContext,
) -> Result<(), AppError> {
    let payload = serde_json::to_value(customer).map_err(|err| AppError::Internal {
        message: format!("serialize customer: {err}"),
    })?;
    sync::record_upsert(conn, "customers", &customer.id, payload, ctx)?;
    Ok(())
}

pub fn create_customer(conn: &Connection, input: CustomerInput) -> Result<Customer, AppError> {
    let validated = validate_customer_input(&input)?;
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let id = new_entity_id();
    let customer = repository::insert_customer(conn, &id, &validated, &now, &ctx)?;
    record(conn, &customer, &ctx)?;
    Ok(customer)
}

pub fn update_customer(
    conn: &Connection,
    id: String,
    input: CustomerInput,
) -> Result<Customer, AppError> {
    let id = parse_entity_id(&id)?;
    let validated = validate_customer_input(&input)?;
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let customer = repository::update_customer(conn, &id, &validated, &now, &ctx)?;
    record(conn, &customer, &ctx)?;
    Ok(customer)
}

pub fn get_customer(conn: &Connection, id: String) -> Result<Customer, AppError> {
    let id = parse_entity_id(&id)?;
    repository::get_customer_by_id(conn, &id)?.ok_or(AppError::NotFound)
}

pub fn list_customers(
    conn: &Connection,
    query: CustomerListQuery,
) -> Result<CustomerListResult, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query
        .page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let include_archived = query.include_archived.unwrap_or(false);
    let offset = (page - 1).saturating_mul(page_size);

    let (items, total) = repository::list_customers(
        conn,
        query.query.as_deref(),
        include_archived,
        page_size,
        offset,
    )?;

    Ok(CustomerListResult {
        items,
        total,
        page,
        page_size,
    })
}

pub fn archive_customer(conn: &Connection, id: String) -> Result<Customer, AppError> {
    let existing = get_customer(conn, id.clone())?;
    if existing.archived_at.is_some() {
        return Ok(existing);
    }
    let id = parse_entity_id(&id)?;
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let customer = repository::set_archived_at(conn, &id, Some(&now), &now, &ctx)?;
    record(conn, &customer, &ctx)?;
    Ok(customer)
}

pub fn unarchive_customer(conn: &Connection, id: String) -> Result<Customer, AppError> {
    let existing = get_customer(conn, id.clone())?;
    if existing.archived_at.is_none() {
        return Ok(existing);
    }
    let id = parse_entity_id(&id)?;
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let customer = repository::set_archived_at(conn, &id, None, &now, &ctx)?;
    record(conn, &customer, &ctx)?;
    Ok(customer)
}
