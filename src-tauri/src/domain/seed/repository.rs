#[cfg(test)]
use rusqlite::Connection;
use rusqlite::{params, OptionalExtension, Transaction};
use time::OffsetDateTime;

use crate::db::repository::now_utc_rfc3339;
use crate::domain::ids::new_entity_id;
use crate::domain::repairs::constants::{DEFAULT_STATUS, REPAIR_STATUSES};
use crate::domain::sync::hlc::tick_hlc;
use crate::error::AppError;

const SEED_DEVICE_CODE: &str = "AA";

/// Insert `count` synthetic customers; returns their new ids in insert order.
pub fn insert_customers(
    tx: &Transaction<'_>,
    count: u32,
    now: &str,
) -> Result<Vec<String>, AppError> {
    if count == 0 {
        return Ok(Vec::new());
    }

    let (wall, counter, origin) = tick_hlc(tx)?;
    let mut stmt = tx.prepare(
        "INSERT INTO customers (
            id, name, phone, email, address, notes, created_at, updated_at, archived_at,
            hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7, NULL, ?8, ?9, ?10, NULL, NULL)",
    )?;

    let mut ids = Vec::with_capacity(count as usize);
    for i in 1..=count {
        let id = new_entity_id();
        let name = format!("Seed Customer {i}");
        let phone = format!("+49000{i:08}");
        let email = format!("seed{i}@example.test");
        let address = format!("Seed Street {i}");
        stmt.execute(params![
            id, name, phone, email, address, now, now, wall, counter, origin
        ])?;
        ids.push(id);
    }
    Ok(ids)
}

/// Insert `count` synthetic devices round-robin across `customer_ids`.
/// Returns `(device_id, customer_id)` pairs.
pub fn insert_devices(
    tx: &Transaction<'_>,
    count: u32,
    customer_ids: &[String],
    now: &str,
) -> Result<Vec<(String, String)>, AppError> {
    if count == 0 {
        return Ok(Vec::new());
    }
    if customer_ids.is_empty() {
        return Err(AppError::Internal {
            message: "seed devices requires customer ids".into(),
        });
    }

    let (wall, counter, origin) = tick_hlc(tx)?;
    let mut stmt = tx.prepare(
        "INSERT INTO devices (
            id, customer_id, device_type, manufacturer, model, serial_number,
            accessories, notes, created_at, updated_at, archived_at,
            hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL, ?7, ?8, NULL, ?9, ?10, ?11, NULL, NULL)",
    )?;

    let mut out = Vec::with_capacity(count as usize);
    let n_customers = customer_ids.len();
    for i in 1..=count {
        let id = new_entity_id();
        let customer_id = customer_ids[((i as usize) - 1) % n_customers].clone();
        let device_type = "Phone";
        let manufacturer = "SeedCo";
        let model = format!("Model-{}", ((i - 1) % 20) + 1);
        let serial = format!("SEED-SN-{i:08}");
        stmt.execute(params![
            id,
            customer_id,
            device_type,
            manufacturer,
            model,
            serial,
            now,
            now,
            wall,
            counter,
            origin
        ])?;
        out.push((id, customer_id));
    }
    Ok(out)
}

/// Reserve `count` repair numbers for `year` + device code `AA` (`2026-AA-000001`).
pub fn allocate_repair_numbers(
    tx: &Transaction<'_>,
    year: i32,
    count: u32,
) -> Result<Vec<String>, AppError> {
    if count == 0 {
        return Ok(Vec::new());
    }

    tx.execute(
        "INSERT INTO repair_number_sequences (year, device_code, last_value) VALUES (?1, ?2, 0)
         ON CONFLICT(year, device_code) DO NOTHING",
        params![year, SEED_DEVICE_CODE],
    )?;

    let start: i64 = tx.query_row(
        "SELECT last_value FROM repair_number_sequences WHERE year = ?1 AND device_code = ?2",
        params![year, SEED_DEVICE_CODE],
        |row| row.get(0),
    )?;

    let end = start + i64::from(count);
    tx.execute(
        "UPDATE repair_number_sequences SET last_value = ?1 WHERE year = ?2 AND device_code = ?3",
        params![end, year, SEED_DEVICE_CODE],
    )?;

    let mut numbers = Vec::with_capacity(count as usize);
    for seq in (start + 1)..=end {
        numbers.push(format!("{year}-{SEED_DEVICE_CODE}-{seq:06}"));
    }
    Ok(numbers)
}

/// Insert synthetic repairs linked to devices (and their customers).
pub fn insert_repairs(
    tx: &Transaction<'_>,
    count: u32,
    devices: &[(String, String)],
    repair_numbers: &[String],
    company_id: &str,
    now: &str,
) -> Result<(), AppError> {
    if count == 0 {
        return Ok(());
    }
    if devices.is_empty() {
        return Err(AppError::Internal {
            message: "seed repairs requires devices".into(),
        });
    }
    if repair_numbers.len() != count as usize {
        return Err(AppError::Internal {
            message: "seed repair number count mismatch".into(),
        });
    }

    let (wall, counter, origin) = tick_hlc(tx)?;
    let mut stmt = tx.prepare(
        "INSERT INTO repairs (
            id, repair_number, customer_id, device_id, company_id, assigned_to_staff_id,
            status, received_at, reported_problem, accessories_received, device_condition,
            diagnosis_notes, work_performed, notes, expected_pickup_at,
            estimate_base_cents, estimate_tax_rate_bps, estimate_tax_cents, estimate_gross_cents,
            ready_at, collected_at, created_at, updated_at, archived_at,
            hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7, ?8, NULL, NULL, NULL, NULL, NULL, NULL,
            NULL, NULL, NULL, NULL, NULL, NULL, ?9, ?10, NULL, ?11, ?12, ?13, NULL, NULL
         )",
    )?;

    let n_devices = devices.len();
    let n_statuses = REPAIR_STATUSES.len().max(1);

    for i in 0..count as usize {
        let (device_id, customer_id) = &devices[i % n_devices];
        let status = if i == 0 {
            DEFAULT_STATUS
        } else {
            REPAIR_STATUSES[i % n_statuses]
        };
        let problem = format!("Seed problem {}", i + 1);
        let id = new_entity_id();
        stmt.execute(params![
            id,
            &repair_numbers[i],
            customer_id,
            device_id,
            company_id,
            status,
            now,
            problem,
            now,
            now,
            wall,
            counter,
            origin
        ])?;
    }
    Ok(())
}

/// Ensure at least one company exists for seeded repairs; returns its id.
pub fn ensure_seed_company(tx: &Transaction<'_>, now: &str) -> Result<String, AppError> {
    let existing: Option<String> = tx
        .query_row(
            "SELECT id FROM companies WHERE archived_at IS NULL ORDER BY is_default DESC, id ASC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(id) = existing {
        return Ok(id);
    }

    let id = new_entity_id();
    let (wall, counter, origin) = tick_hlc(tx)?;
    tx.execute(
        "INSERT INTO companies (
            id, legal_name, trade_name, tax_id, address, phone, email, website,
            logo_path, logo_content_hash, is_default, created_at, updated_at, archived_at,
            hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
         ) VALUES (?1, 'Seed Company', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 1, ?2, ?3, NULL, ?4, ?5, ?6, NULL, NULL)",
        params![id, now, now, wall, counter, origin],
    )?;
    Ok(id)
}

pub fn local_calendar_year() -> i32 {
    OffsetDateTime::now_local()
        .map(|dt| dt.year())
        .unwrap_or_else(|_| OffsetDateTime::now_utc().year())
}

/// Count rows in a table (active + archived). Used by tests / verification.
#[cfg(test)]
pub fn count_rows(conn: &Connection, table: &str) -> Result<i64, AppError> {
    // Whitelist table names — never interpolate user input.
    let sql = match table {
        "customers" => "SELECT COUNT(*) FROM customers",
        "devices" => "SELECT COUNT(*) FROM devices",
        "repairs" => "SELECT COUNT(*) FROM repairs",
        _ => {
            return Err(AppError::Internal {
                message: format!("unsupported count table: {table}"),
            });
        }
    };
    Ok(conn.query_row(sql, [], |row| row.get(0))?)
}

pub fn timestamp_now() -> Result<String, AppError> {
    now_utc_rfc3339()
}
