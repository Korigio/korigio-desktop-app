//! Home dashboard SQL queries.

use std::collections::HashMap;

use rusqlite::{Connection, params};

use crate::domain::dashboard::constants::DASHBOARD_LIST_LIMIT;
use crate::domain::dashboard::types::{StatusCount, TodayCounts};
use crate::domain::repairs::constants::REPAIR_STATUSES;
use crate::error::AppError;

/// Raw row before `days_in_status` is computed in the service layer.
pub struct DashboardRepairRaw {
    pub id: i64,
    pub repair_number: String,
    pub status: String,
    pub customer_name: String,
    pub customer_phone: Option<String>,
    pub updated_at: String,
    pub ready_at: Option<String>,
}

pub fn count_by_status(conn: &Connection) -> Result<Vec<StatusCount>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT status, COUNT(*)
         FROM repairs
         WHERE archived_at IS NULL
         GROUP BY status",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;

    let mut by_status: HashMap<String, i64> = HashMap::new();
    for row in rows {
        let (status, count) = row?;
        by_status.insert(status, count);
    }

    Ok(REPAIR_STATUSES
        .iter()
        .map(|status| StatusCount {
            status: (*status).to_string(),
            count: by_status.get(*status).copied().unwrap_or(0),
        })
        .collect())
}

pub fn list_ready_for_pickup(conn: &Connection) -> Result<Vec<DashboardRepairRaw>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT repairs.id, repairs.repair_number, repairs.status,
                customers.name, customers.phone,
                repairs.updated_at, repairs.ready_at
         FROM repairs
         INNER JOIN customers ON customers.id = repairs.customer_id
         WHERE repairs.archived_at IS NULL
           AND repairs.status = 'ready'
         ORDER BY repairs.ready_at IS NULL, repairs.ready_at ASC, repairs.updated_at ASC
         LIMIT ?1",
    )?;

    let mapped = stmt.query_map(params![DASHBOARD_LIST_LIMIT], map_raw)?;
    let mut out = Vec::new();
    for row in mapped {
        out.push(row?);
    }
    Ok(out)
}

pub fn list_stale_repairs(
    conn: &Connection,
    updated_before: &str,
) -> Result<Vec<DashboardRepairRaw>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT repairs.id, repairs.repair_number, repairs.status,
                customers.name, customers.phone,
                repairs.updated_at, repairs.ready_at
         FROM repairs
         INNER JOIN customers ON customers.id = repairs.customer_id
         WHERE repairs.archived_at IS NULL
           AND repairs.status NOT IN ('collected', 'cancelled')
           AND repairs.updated_at < ?1
         ORDER BY repairs.updated_at ASC
         LIMIT ?2",
    )?;

    let mapped = stmt.query_map(params![updated_before, DASHBOARD_LIST_LIMIT], map_raw)?;
    let mut out = Vec::new();
    for row in mapped {
        out.push(row?);
    }
    Ok(out)
}

pub fn today_counts(conn: &Connection, local_day: &str) -> Result<TodayCounts, AppError> {
    let received: i64 = conn.query_row(
        "SELECT COUNT(*)
         FROM repairs
         WHERE archived_at IS NULL
           AND substr(received_at, 1, 10) = ?1",
        params![local_day],
        |row| row.get(0),
    )?;

    let collected: i64 = conn.query_row(
        "SELECT COUNT(*)
         FROM repairs
         WHERE archived_at IS NULL
           AND collected_at IS NOT NULL
           AND substr(collected_at, 1, 10) = ?1",
        params![local_day],
        |row| row.get(0),
    )?;

    Ok(TodayCounts {
        received,
        collected,
    })
}

fn map_raw(row: &rusqlite::Row<'_>) -> rusqlite::Result<DashboardRepairRaw> {
    Ok(DashboardRepairRaw {
        id: row.get(0)?,
        repair_number: row.get(1)?,
        status: row.get(2)?,
        customer_name: row.get(3)?,
        customer_phone: row.get(4)?,
        updated_at: row.get(5)?,
        ready_at: row.get(6)?,
    })
}
