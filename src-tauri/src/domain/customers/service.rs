use rusqlite::Connection;

use crate::db::repository::now_utc_rfc3339;
use crate::domain::customers::constants::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};
use crate::domain::customers::repository;
use crate::domain::customers::types::{
    Customer, CustomerInput, CustomerListQuery, CustomerListResult,
};
use crate::domain::customers::validation::validate_customer_input;
use crate::error::AppError;

pub fn create_customer(conn: &Connection, input: CustomerInput) -> Result<Customer, AppError> {
    let validated = validate_customer_input(&input)?;
    let now = now_utc_rfc3339()?;
    repository::insert_customer(conn, &validated, &now)
}

pub fn update_customer(
    conn: &Connection,
    id: i64,
    input: CustomerInput,
) -> Result<Customer, AppError> {
    let validated = validate_customer_input(&input)?;
    let now = now_utc_rfc3339()?;
    repository::update_customer(conn, id, &validated, &now)
}

pub fn get_customer(conn: &Connection, id: i64) -> Result<Customer, AppError> {
    repository::get_customer_by_id(conn, id)?.ok_or(AppError::NotFound)
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

pub fn archive_customer(conn: &Connection, id: i64) -> Result<Customer, AppError> {
    let existing = get_customer(conn, id)?;
    if existing.archived_at.is_some() {
        return Ok(existing);
    }
    let now = now_utc_rfc3339()?;
    repository::set_archived_at(conn, id, Some(&now), &now)
}

pub fn unarchive_customer(conn: &Connection, id: i64) -> Result<Customer, AppError> {
    let existing = get_customer(conn, id)?;
    if existing.archived_at.is_none() {
        return Ok(existing);
    }
    let now = now_utc_rfc3339()?;
    repository::set_archived_at(conn, id, None, &now)
}
