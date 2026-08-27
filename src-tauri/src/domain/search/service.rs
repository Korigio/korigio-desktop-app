use rusqlite::Connection;

use crate::domain::customers::{self, CustomerListQuery};
use crate::domain::devices::{self, DeviceListQuery};
use crate::domain::repairs::{self, RepairListQuery};
use crate::domain::search::constants::{DEFAULT_LIMIT_PER_TYPE, MAX_LIMIT_PER_TYPE};
use crate::domain::search::types::{GlobalSearchQuery, GlobalSearchResult};
use crate::error::AppError;

pub fn global_search(
    conn: &Connection,
    query: GlobalSearchQuery,
) -> Result<GlobalSearchResult, AppError> {
    let trimmed = query.query.trim();
    if trimmed.is_empty() {
        return Ok(GlobalSearchResult {
            customers: Vec::new(),
            devices: Vec::new(),
            repairs: Vec::new(),
        });
    }

    let limit = query
        .limit_per_type
        .unwrap_or(DEFAULT_LIMIT_PER_TYPE)
        .clamp(1, MAX_LIMIT_PER_TYPE);

    let customers = customers::list_customers(
        conn,
        CustomerListQuery {
            query: Some(trimmed.to_string()),
            include_archived: Some(false),
            page: Some(1),
            page_size: Some(limit),
        },
    )?
    .items;

    let devices = devices::list_devices(
        conn,
        DeviceListQuery {
            query: Some(trimmed.to_string()),
            customer_id: None,
            include_archived: Some(false),
            page: Some(1),
            page_size: Some(limit),
        },
    )?
    .items;

    let repairs = repairs::list_repairs(
        conn,
        RepairListQuery {
            query: Some(trimmed.to_string()),
            customer_id: None,
            device_id: None,
            status: None,
            page: Some(1),
            page_size: Some(limit),
        },
    )?
    .items;

    Ok(GlobalSearchResult {
        customers,
        devices,
        repairs,
    })
}
