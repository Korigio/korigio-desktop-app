use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::db::repository::like_pattern;
use crate::domain::repairs::types::Repair;
use crate::domain::repairs::validation::{ValidatedCreateRepairInput, ValidatedUpdateRepairInput};
use crate::error::AppError;

pub fn allocate_repair_number(
    tx: &Transaction<'_>,
    year: i32,
) -> Result<String, AppError> {
    tx.execute(
        "INSERT INTO repair_number_sequences (year, last_value) VALUES (?1, 0)
         ON CONFLICT(year) DO NOTHING",
        params![year],
    )?;
    tx.execute(
        "UPDATE repair_number_sequences SET last_value = last_value + 1 WHERE year = ?1",
        params![year],
    )?;
    let seq: i64 = tx.query_row(
        "SELECT last_value FROM repair_number_sequences WHERE year = ?1",
        params![year],
        |row| row.get(0),
    )?;
    Ok(format!("{year}-{seq:06}"))
}

pub fn insert_repair(
    tx: &Transaction<'_>,
    repair_number: &str,
    input: &ValidatedCreateRepairInput,
    received_at: &str,
    now: &str,
) -> Result<Repair, AppError> {
    tx.execute(
        "INSERT INTO repairs (
            repair_number, customer_id, device_id, status, received_at,
            reported_problem, accessories_received, device_condition,
            diagnosis_notes, work_performed, notes, expected_pickup_at,
            ready_at, collected_at, created_at, updated_at, archived_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, NULL, NULL, ?13, ?14, NULL)",
        params![
            repair_number,
            input.customer_id,
            input.device_id,
            input.status,
            received_at,
            input.reported_problem,
            input.accessories_received,
            input.device_condition,
            input.diagnosis_notes,
            input.work_performed,
            input.notes,
            input.expected_pickup_at,
            now,
            now,
        ],
    )?;
    let id = tx.last_insert_rowid();
    get_repair_by_id(tx, id)?.ok_or(AppError::Internal {
        message: "repair missing after insert".into(),
    })
}

pub fn update_repair(
    conn: &Connection,
    id: i64,
    input: &ValidatedUpdateRepairInput,
    ready_at: Option<&str>,
    collected_at: Option<&str>,
    now: &str,
) -> Result<Repair, AppError> {
    let updated = conn.execute(
        "UPDATE repairs SET
            status = ?1,
            reported_problem = ?2,
            accessories_received = ?3,
            device_condition = ?4,
            diagnosis_notes = ?5,
            work_performed = ?6,
            notes = ?7,
            expected_pickup_at = ?8,
            ready_at = ?9,
            collected_at = ?10,
            updated_at = ?11
         WHERE id = ?12 AND archived_at IS NULL",
        params![
            input.status,
            input.reported_problem,
            input.accessories_received,
            input.device_condition,
            input.diagnosis_notes,
            input.work_performed,
            input.notes,
            input.expected_pickup_at,
            ready_at,
            collected_at,
            now,
            id,
        ],
    )?;
    if updated == 0 {
        return match get_repair_by_id(conn, id)? {
            Some(_) => Err(AppError::Validation {
                field: None,
                message: "Archived repairs cannot be edited.".into(),
            }),
            None => Err(AppError::NotFound),
        };
    }
    get_repair_by_id(conn, id)?.ok_or(AppError::NotFound)
}

pub fn get_repair_by_id(conn: &Connection, id: i64) -> Result<Option<Repair>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, repair_number, customer_id, device_id, status, received_at,
                reported_problem, accessories_received, device_condition,
                diagnosis_notes, work_performed, notes, expected_pickup_at,
                ready_at, collected_at, created_at, updated_at, archived_at
         FROM repairs WHERE id = ?1",
    )?;
    let repair = stmt.query_row(params![id], map_repair).optional()?;
    Ok(repair)
}

const REPAIR_SELECT_COLS: &str = "repairs.id, repairs.repair_number, repairs.customer_id,
        repairs.device_id, repairs.status, repairs.received_at,
        repairs.reported_problem, repairs.accessories_received, repairs.device_condition,
        repairs.diagnosis_notes, repairs.work_performed, repairs.notes,
        repairs.expected_pickup_at, repairs.ready_at, repairs.collected_at,
        repairs.created_at, repairs.updated_at, repairs.archived_at";

const SEARCH_MATCH_SQL: &str = "(
            repairs.repair_number LIKE {ph} ESCAPE '\\'
            OR IFNULL(repairs.reported_problem, '') LIKE {ph} ESCAPE '\\'
            OR customers.name LIKE {ph} ESCAPE '\\'
            OR IFNULL(customers.phone, '') LIKE {ph} ESCAPE '\\'
            OR IFNULL(devices.serial_number, '') LIKE {ph} ESCAPE '\\'
            OR IFNULL(devices.manufacturer, '') LIKE {ph} ESCAPE '\\'
            OR IFNULL(devices.model, '') LIKE {ph} ESCAPE '\\'
        )";

pub fn list_repairs(
    conn: &Connection,
    search: Option<&str>,
    customer_id: Option<i64>,
    device_id: Option<i64>,
    status: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<(Vec<Repair>, i64), AppError> {
    let pattern = like_pattern(search);
    let needs_join = pattern.is_some();

    let mut where_parts = vec!["repairs.archived_at IS NULL".to_string()];
    if customer_id.is_some() {
        where_parts.push("repairs.customer_id = ?1".into());
    }
    if device_id.is_some() {
        let idx = 1 + usize::from(customer_id.is_some());
        where_parts.push(format!("repairs.device_id = ?{idx}"));
    }
    if status.is_some() {
        let idx = 1
            + usize::from(customer_id.is_some())
            + usize::from(device_id.is_some());
        where_parts.push(format!("repairs.status = ?{idx}"));
    }
    if pattern.is_some() {
        let idx = 1
            + usize::from(customer_id.is_some())
            + usize::from(device_id.is_some())
            + usize::from(status.is_some());
        let ph = format!("?{idx}");
        where_parts.push(SEARCH_MATCH_SQL.replace("{ph}", &ph));
    }

    let where_sql = format!("WHERE {}", where_parts.join(" AND "));
    let from_sql = if needs_join {
        "FROM repairs
         INNER JOIN customers ON customers.id = repairs.customer_id
         INNER JOIN devices ON devices.id = repairs.device_id"
    } else {
        "FROM repairs"
    };

    // Without a join, column names need no table prefix in SELECT.
    let select_cols = if needs_join {
        REPAIR_SELECT_COLS
    } else {
        "id, repair_number, customer_id, device_id, status, received_at,
                reported_problem, accessories_received, device_condition,
                diagnosis_notes, work_performed, notes, expected_pickup_at,
                ready_at, collected_at, created_at, updated_at, archived_at"
    };

    // When not joining, drop the `repairs.` prefix from WHERE for clarity/consistency
    // with the unprefixed FROM — rewrite archived/customer/device/status filters.
    let where_sql = if needs_join {
        where_sql
    } else {
        where_sql.replace("repairs.", "")
    };

    let param_count = usize::from(customer_id.is_some())
        + usize::from(device_id.is_some())
        + usize::from(status.is_some())
        + usize::from(pattern.is_some());
    let limit_ph = format!("?{}", param_count + 1);
    let offset_ph = format!("?{}", param_count + 2);

    let count_sql = format!("SELECT COUNT(*) {from_sql} {where_sql}");
    let list_sql = format!(
        "SELECT {select_cols}
         {from_sql}
         {where_sql}
         ORDER BY received_at DESC, id DESC
         LIMIT {limit_ph} OFFSET {offset_ph}"
    );

    // Unprefixed ORDER BY is fine; with join use repairs.received_at / repairs.id.
    let list_sql = if needs_join {
        list_sql
            .replace("ORDER BY received_at DESC, id DESC", "ORDER BY repairs.received_at DESC, repairs.id DESC")
    } else {
        list_sql
    };

    let total = query_count(conn, &count_sql, customer_id, device_id, status, pattern.as_deref())?;

    let mut stmt = conn.prepare(&list_sql)?;
    let rows = query_rows(
        &mut stmt,
        customer_id,
        device_id,
        status,
        pattern.as_deref(),
        limit,
        offset,
    )?;

    Ok((rows, total))
}

fn query_count(
    conn: &Connection,
    sql: &str,
    customer_id: Option<i64>,
    device_id: Option<i64>,
    status: Option<&str>,
    pattern: Option<&str>,
) -> Result<i64, AppError> {
    match (customer_id, device_id, status, pattern) {
        (None, None, None, None) => conn.query_row(sql, [], |row| row.get(0)),
        (Some(c), None, None, None) => conn.query_row(sql, params![c], |row| row.get(0)),
        (None, Some(d), None, None) => conn.query_row(sql, params![d], |row| row.get(0)),
        (None, None, Some(s), None) => conn.query_row(sql, params![s], |row| row.get(0)),
        (None, None, None, Some(q)) => conn.query_row(sql, params![q], |row| row.get(0)),
        (Some(c), Some(d), None, None) => conn.query_row(sql, params![c, d], |row| row.get(0)),
        (Some(c), None, Some(s), None) => conn.query_row(sql, params![c, s], |row| row.get(0)),
        (Some(c), None, None, Some(q)) => conn.query_row(sql, params![c, q], |row| row.get(0)),
        (None, Some(d), Some(s), None) => conn.query_row(sql, params![d, s], |row| row.get(0)),
        (None, Some(d), None, Some(q)) => conn.query_row(sql, params![d, q], |row| row.get(0)),
        (None, None, Some(s), Some(q)) => conn.query_row(sql, params![s, q], |row| row.get(0)),
        (Some(c), Some(d), Some(s), None) => {
            conn.query_row(sql, params![c, d, s], |row| row.get(0))
        }
        (Some(c), Some(d), None, Some(q)) => {
            conn.query_row(sql, params![c, d, q], |row| row.get(0))
        }
        (Some(c), None, Some(s), Some(q)) => {
            conn.query_row(sql, params![c, s, q], |row| row.get(0))
        }
        (None, Some(d), Some(s), Some(q)) => {
            conn.query_row(sql, params![d, s, q], |row| row.get(0))
        }
        (Some(c), Some(d), Some(s), Some(q)) => {
            conn.query_row(sql, params![c, d, s, q], |row| row.get(0))
        }
    }
    .map_err(AppError::from)
}

fn query_rows(
    stmt: &mut rusqlite::Statement<'_>,
    customer_id: Option<i64>,
    device_id: Option<i64>,
    status: Option<&str>,
    pattern: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<Vec<Repair>, AppError> {
    let rows = match (customer_id, device_id, status, pattern) {
        (None, None, None, None) => stmt
            .query_map(params![limit, offset], map_repair)?
            .collect::<Result<Vec<_>, _>>()?,
        (Some(c), None, None, None) => stmt
            .query_map(params![c, limit, offset], map_repair)?
            .collect::<Result<Vec<_>, _>>()?,
        (None, Some(d), None, None) => stmt
            .query_map(params![d, limit, offset], map_repair)?
            .collect::<Result<Vec<_>, _>>()?,
        (None, None, Some(s), None) => stmt
            .query_map(params![s, limit, offset], map_repair)?
            .collect::<Result<Vec<_>, _>>()?,
        (None, None, None, Some(q)) => stmt
            .query_map(params![q, limit, offset], map_repair)?
            .collect::<Result<Vec<_>, _>>()?,
        (Some(c), Some(d), None, None) => stmt
            .query_map(params![c, d, limit, offset], map_repair)?
            .collect::<Result<Vec<_>, _>>()?,
        (Some(c), None, Some(s), None) => stmt
            .query_map(params![c, s, limit, offset], map_repair)?
            .collect::<Result<Vec<_>, _>>()?,
        (Some(c), None, None, Some(q)) => stmt
            .query_map(params![c, q, limit, offset], map_repair)?
            .collect::<Result<Vec<_>, _>>()?,
        (None, Some(d), Some(s), None) => stmt
            .query_map(params![d, s, limit, offset], map_repair)?
            .collect::<Result<Vec<_>, _>>()?,
        (None, Some(d), None, Some(q)) => stmt
            .query_map(params![d, q, limit, offset], map_repair)?
            .collect::<Result<Vec<_>, _>>()?,
        (None, None, Some(s), Some(q)) => stmt
            .query_map(params![s, q, limit, offset], map_repair)?
            .collect::<Result<Vec<_>, _>>()?,
        (Some(c), Some(d), Some(s), None) => stmt
            .query_map(params![c, d, s, limit, offset], map_repair)?
            .collect::<Result<Vec<_>, _>>()?,
        (Some(c), Some(d), None, Some(q)) => stmt
            .query_map(params![c, d, q, limit, offset], map_repair)?
            .collect::<Result<Vec<_>, _>>()?,
        (Some(c), None, Some(s), Some(q)) => stmt
            .query_map(params![c, s, q, limit, offset], map_repair)?
            .collect::<Result<Vec<_>, _>>()?,
        (None, Some(d), Some(s), Some(q)) => stmt
            .query_map(params![d, s, q, limit, offset], map_repair)?
            .collect::<Result<Vec<_>, _>>()?,
        (Some(c), Some(d), Some(s), Some(q)) => stmt
            .query_map(params![c, d, s, q, limit, offset], map_repair)?
            .collect::<Result<Vec<_>, _>>()?,
    };
    Ok(rows)
}

fn map_repair(row: &rusqlite::Row<'_>) -> rusqlite::Result<Repair> {
    Ok(Repair {
        id: row.get(0)?,
        repair_number: row.get(1)?,
        customer_id: row.get(2)?,
        device_id: row.get(3)?,
        status: row.get(4)?,
        received_at: row.get(5)?,
        reported_problem: row.get(6)?,
        accessories_received: row.get(7)?,
        device_condition: row.get(8)?,
        diagnosis_notes: row.get(9)?,
        work_performed: row.get(10)?,
        notes: row.get(11)?,
        expected_pickup_at: row.get(12)?,
        ready_at: row.get(13)?,
        collected_at: row.get(14)?,
        created_at: row.get(15)?,
        updated_at: row.get(16)?,
        archived_at: row.get(17)?,
    })
}
