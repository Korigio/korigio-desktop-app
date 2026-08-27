use rusqlite::{Connection, Transaction, params};
use time::OffsetDateTime;

use crate::db::repository::now_utc_rfc3339;
use crate::domain::repairs::constants::{DEFAULT_STATUS, REPAIR_STATUSES};
use crate::error::AppError;

/// Insert `count` synthetic customers; returns their new ids in insert order.
pub fn insert_customers(
    tx: &Transaction<'_>,
    count: u32,
    now: &str,
) -> Result<Vec<i64>, AppError> {
    if count == 0 {
        return Ok(Vec::new());
    }

    let mut stmt = tx.prepare(
        "INSERT INTO customers (name, phone, email, address, notes, created_at, updated_at, archived_at)
         VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6, NULL)",
    )?;

    let mut ids = Vec::with_capacity(count as usize);
    for i in 1..=count {
        let name = format!("Seed Customer {i}");
        let phone = format!("+49000{i:08}");
        let email = format!("seed{i}@example.test");
        let address = format!("Seed Street {i}");
        stmt.execute(params![name, phone, email, address, now, now])?;
        ids.push(tx.last_insert_rowid());
    }
    Ok(ids)
}

/// Insert `count` synthetic devices round-robin across `customer_ids`.
/// Returns `(device_id, customer_id)` pairs.
pub fn insert_devices(
    tx: &Transaction<'_>,
    count: u32,
    customer_ids: &[i64],
    now: &str,
) -> Result<Vec<(i64, i64)>, AppError> {
    if count == 0 {
        return Ok(Vec::new());
    }
    if customer_ids.is_empty() {
        return Err(AppError::Internal {
            message: "seed devices requires customer ids".into(),
        });
    }

    let mut stmt = tx.prepare(
        "INSERT INTO devices (
            customer_id, device_type, manufacturer, model, serial_number,
            accessories, notes, created_at, updated_at, archived_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, NULL, NULL, ?6, ?7, NULL)",
    )?;

    let mut out = Vec::with_capacity(count as usize);
    let n_customers = customer_ids.len();
    for i in 1..=count {
        let customer_id = customer_ids[((i as usize) - 1) % n_customers];
        let device_type = "Phone";
        let manufacturer = "SeedCo";
        let model = format!("Model-{}", ((i - 1) % 20) + 1);
        let serial = format!("SEED-SN-{i:08}");
        stmt.execute(params![
            customer_id,
            device_type,
            manufacturer,
            model,
            serial,
            now,
            now
        ])?;
        out.push((tx.last_insert_rowid(), customer_id));
    }
    Ok(out)
}

/// Reserve `count` repair numbers for `year` and return them (e.g. `2026-000001`).
pub fn allocate_repair_numbers(
    tx: &Transaction<'_>,
    year: i32,
    count: u32,
) -> Result<Vec<String>, AppError> {
    if count == 0 {
        return Ok(Vec::new());
    }

    tx.execute(
        "INSERT INTO repair_number_sequences (year, last_value) VALUES (?1, 0)
         ON CONFLICT(year) DO NOTHING",
        params![year],
    )?;

    let start: i64 = tx.query_row(
        "SELECT last_value FROM repair_number_sequences WHERE year = ?1",
        params![year],
        |row| row.get(0),
    )?;

    let end = start + i64::from(count);
    tx.execute(
        "UPDATE repair_number_sequences SET last_value = ?1 WHERE year = ?2",
        params![end, year],
    )?;

    let mut numbers = Vec::with_capacity(count as usize);
    for seq in (start + 1)..=end {
        numbers.push(format!("{year}-{seq:06}"));
    }
    Ok(numbers)
}

/// Insert synthetic repairs linked to devices (and their customers).
pub fn insert_repairs(
    tx: &Transaction<'_>,
    count: u32,
    devices: &[(i64, i64)],
    repair_numbers: &[String],
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

    let mut stmt = tx.prepare(
        "INSERT INTO repairs (
            repair_number, customer_id, device_id, status, received_at,
            reported_problem, accessories_received, device_condition,
            diagnosis_notes, work_performed, notes, ready_at, collected_at,
            created_at, updated_at, archived_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL, NULL, NULL, NULL, NULL, NULL, ?7, ?8, NULL)",
    )?;

    let n_devices = devices.len();
    let n_statuses = REPAIR_STATUSES.len().max(1);

    for i in 0..count as usize {
        let (device_id, customer_id) = devices[i % n_devices];
        let status = if i == 0 {
            DEFAULT_STATUS
        } else {
            REPAIR_STATUSES[i % n_statuses]
        };
        let problem = format!("Seed problem {}", i + 1);
        stmt.execute(params![
            &repair_numbers[i],
            customer_id,
            device_id,
            status,
            now,
            problem,
            now,
            now
        ])?;
    }
    Ok(())
}

pub fn local_calendar_year() -> i32 {
    OffsetDateTime::now_local()
        .map(|dt| dt.year())
        .unwrap_or_else(|_| OffsetDateTime::now_utc().year())
}

/// Count rows in a table (active + archived). Used by tests / verification.
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
