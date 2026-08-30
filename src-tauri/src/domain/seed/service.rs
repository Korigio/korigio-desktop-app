use std::time::Instant;

use rusqlite::Connection;

use crate::domain::seed::constants::{
    DEFAULT_CUSTOMERS, DEFAULT_DEVICES, DEFAULT_REPAIRS, MAX_CUSTOMERS, MAX_DEVICES, MAX_REPAIRS,
};
use crate::domain::seed::repository;
use crate::domain::seed::types::{SeedSyntheticDataInput, SeedSyntheticDataResult};
use crate::error::AppError;

/// Insert synthetic customers / devices / repairs for local performance measurement.
///
/// Requires `confirm: true`. Defaults are small; pass explicit counts for full measure
/// (e.g. 10000 / 20000 / 50000). Caps: customers ≤ 20000, devices ≤ 40000, repairs ≤ 100000.
pub fn seed_synthetic_data(
    conn: &Connection,
    input: SeedSyntheticDataInput,
) -> Result<SeedSyntheticDataResult, AppError> {
    if !input.confirm {
        return Err(AppError::Validation {
            field: Some("confirm".into()),
            message: "Synthetic seed requires confirm: true.".into(),
        });
    }

    let customers = resolve_count(
        input.customers,
        DEFAULT_CUSTOMERS,
        MAX_CUSTOMERS,
        "customers",
    )?;
    let devices = resolve_count(input.devices, DEFAULT_DEVICES, MAX_DEVICES, "devices")?;
    let repairs = resolve_count(input.repairs, DEFAULT_REPAIRS, MAX_REPAIRS, "repairs")?;

    if devices > 0 && customers == 0 {
        return Err(AppError::Validation {
            field: Some("customers".into()),
            message: "Cannot seed devices without customers.".into(),
        });
    }
    if repairs > 0 && devices == 0 {
        return Err(AppError::Validation {
            field: Some("devices".into()),
            message: "Cannot seed repairs without devices.".into(),
        });
    }

    let started = Instant::now();
    let now = repository::timestamp_now()?;
    let year = repository::local_calendar_year();

    let tx = conn.unchecked_transaction()?;
    let customer_ids = repository::insert_customers(&tx, customers, &now)?;
    let device_pairs = repository::insert_devices(&tx, devices, &customer_ids, &now)?;
    let repair_numbers = repository::allocate_repair_numbers(&tx, year, repairs)?;
    let company_id = if repairs > 0 {
        repository::ensure_seed_company(&tx, &now)?
    } else {
        String::new()
    };
    repository::insert_repairs(
        &tx,
        repairs,
        &device_pairs,
        &repair_numbers,
        &company_id,
        &now,
    )?;
    tx.commit()?;

    let elapsed_ms = started.elapsed().as_millis() as u64;

    Ok(SeedSyntheticDataResult {
        customers,
        devices,
        repairs,
        elapsed_ms,
    })
}

fn resolve_count(value: Option<u32>, default: u32, max: u32, field: &str) -> Result<u32, AppError> {
    let count = value.unwrap_or(default);
    if count > max {
        return Err(AppError::Validation {
            field: Some(field.into()),
            message: format!("{field} must be at most {max}."),
        });
    }
    Ok(count)
}
