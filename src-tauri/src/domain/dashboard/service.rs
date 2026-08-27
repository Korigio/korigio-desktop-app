//! Home dashboard aggregation.

use rusqlite::Connection;
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

use crate::domain::dashboard::constants::STALE_AFTER_DAYS;
use crate::domain::dashboard::repository::{self, DashboardRepairRaw};
use crate::domain::dashboard::types::{DashboardRepairRow, HomeDashboard};
use crate::error::AppError;

pub fn get_home_dashboard(conn: &Connection) -> Result<HomeDashboard, AppError> {
    let now = OffsetDateTime::now_utc();
    let local_day = local_calendar_day()?;
    let stale_cutoff = format_rfc3339(now - Duration::days(i64::from(STALE_AFTER_DAYS)))?;

    let status_counts = repository::count_by_status(conn)?;
    let ready_raw = repository::list_ready_for_pickup(conn)?;
    let stale_raw = repository::list_stale_repairs(conn, &stale_cutoff)?;
    let today = repository::today_counts(conn, &local_day)?;

    Ok(HomeDashboard {
        status_counts,
        ready_for_pickup: ready_raw
            .into_iter()
            .map(|row| to_row(row, now, true))
            .collect(),
        stale_repairs: stale_raw
            .into_iter()
            .map(|row| to_row(row, now, false))
            .collect(),
        today,
        stale_after_days: STALE_AFTER_DAYS,
    })
}

fn to_row(raw: DashboardRepairRaw, now: OffsetDateTime, use_ready_anchor: bool) -> DashboardRepairRow {
    let anchor = if use_ready_anchor {
        raw.ready_at.as_deref().unwrap_or(raw.updated_at.as_str())
    } else {
        raw.updated_at.as_str()
    };
    let days_in_status = days_since(anchor, now);

    DashboardRepairRow {
        id: raw.id,
        repair_number: raw.repair_number,
        status: raw.status,
        customer_name: raw.customer_name,
        customer_phone: raw.customer_phone,
        updated_at: raw.updated_at,
        ready_at: raw.ready_at,
        days_in_status,
    }
}

fn days_since(iso: &str, now: OffsetDateTime) -> i64 {
    match OffsetDateTime::parse(iso, &Rfc3339) {
        Ok(then) => {
            let secs = (now - then).whole_seconds();
            if secs <= 0 {
                0
            } else {
                secs / 86_400
            }
        }
        Err(_) => 0,
    }
}

fn local_calendar_day() -> Result<String, AppError> {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    let date = now.date();
    Ok(format!(
        "{:04}-{:02}-{:02}",
        date.year(),
        u8::from(date.month()),
        date.day()
    ))
}

fn format_rfc3339(dt: OffsetDateTime) -> Result<String, AppError> {
    dt.format(&Rfc3339).map_err(|err| AppError::Internal {
        message: format!("timestamp format failed: {err}"),
    })
}
