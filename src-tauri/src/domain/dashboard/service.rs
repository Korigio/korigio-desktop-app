//! Home dashboard aggregation.

use rusqlite::Connection;
use time::format_description::well_known::Rfc3339;
use time::{Date, Duration, OffsetDateTime};

use crate::domain::dashboard::constants::{INTAKE_SERIES_DAYS, STALE_AFTER_DAYS};
use crate::domain::dashboard::repository::{self, DashboardRepairRaw};
use crate::domain::dashboard::types::{
    DashboardRepairRow, HomeDashboard, IntakeDayCounts, RevenueDayTotals, RevenueTotals,
};
use crate::domain::identity;
use crate::domain::settings;
use crate::domain::staff::{self, StaffRole};
use crate::error::AppError;

pub fn get_home_dashboard(conn: &Connection) -> Result<HomeDashboard, AppError> {
    let now = OffsetDateTime::now_utc();
    let local_date = local_calendar_date();
    let local_day = format_calendar_day(local_date);
    let stale_cutoff = format_rfc3339(now - Duration::days(i64::from(STALE_AFTER_DAYS)))?;

    let status_counts = repository::count_by_status(conn)?;
    let ready_raw = repository::list_ready_for_pickup(conn)?;
    let stale_raw = repository::list_stale_repairs(conn, &stale_cutoff)?;
    let stale_count = repository::count_stale_repairs(conn, &stale_cutoff)?;
    let today = repository::today_counts(conn, &local_day)?;
    let intake_by_day = intake_series(conn, local_date)?;
    let (revenue, revenue_by_day) = if revenue_visible(conn)? {
        let series = revenue_series(conn, local_date)?;
        let totals = revenue_totals(&series, conn)?;
        (Some(totals), Some(series))
    } else {
        (None, None)
    };
    let currency = settings::get_shop_settings(conn)?.currency;

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
        intake_by_day,
        stale_count,
        stale_after_days: STALE_AFTER_DAYS,
        currency,
        revenue,
        revenue_by_day,
    })
}

fn revenue_visible(conn: &Connection) -> Result<bool, AppError> {
    let identity = identity::require_local_identity(conn)?;
    if identity.team_id.is_none() {
        return Ok(true);
    }
    match staff::get_current_session(conn)? {
        Some(session) if session.staff.role == StaffRole::Admin => Ok(true),
        _ => Ok(false),
    }
}

fn intake_series(conn: &Connection, today: Date) -> Result<Vec<IntakeDayCounts>, AppError> {
    let dates = intake_series_dates(today)?;
    let from_day = dates
        .first()
        .cloned()
        .unwrap_or_else(|| format_calendar_day(today));
    let to_day = dates
        .last()
        .cloned()
        .unwrap_or_else(|| format_calendar_day(today));
    let received = repository::count_received_by_day(conn, &from_day, &to_day)?;
    let collected = repository::count_collected_by_day(conn, &from_day, &to_day)?;

    Ok(dates
        .into_iter()
        .map(|date| IntakeDayCounts {
            received: received.get(&date).copied().unwrap_or(0),
            collected: collected.get(&date).copied().unwrap_or(0),
            date,
        })
        .collect())
}

fn revenue_series(conn: &Connection, today: Date) -> Result<Vec<RevenueDayTotals>, AppError> {
    let dates = intake_series_dates(today)?;
    let from_day = dates
        .first()
        .cloned()
        .unwrap_or_else(|| format_calendar_day(today));
    let to_day = dates
        .last()
        .cloned()
        .unwrap_or_else(|| format_calendar_day(today));
    let collected = repository::sum_collected_gross_by_day(conn, &from_day, &to_day)?;

    Ok(dates
        .into_iter()
        .map(|date| RevenueDayTotals {
            collected_gross_cents: collected.get(&date).copied().unwrap_or(0),
            date,
        })
        .collect())
}

fn revenue_totals(
    series: &[RevenueDayTotals],
    conn: &Connection,
) -> Result<RevenueTotals, AppError> {
    let collected_gross_cents_today = series
        .last()
        .map(|day| day.collected_gross_cents)
        .unwrap_or(0);
    let collected_gross_cents_week: i64 = series.iter().map(|day| day.collected_gross_cents).sum();
    let open_estimate_gross_cents = repository::sum_open_estimate_gross(conn)?;

    Ok(RevenueTotals {
        collected_gross_cents_today,
        collected_gross_cents_week,
        open_estimate_gross_cents,
    })
}

fn intake_series_dates(today: Date) -> Result<Vec<String>, AppError> {
    let count = i64::from(INTAKE_SERIES_DAYS);
    let mut dates = Vec::with_capacity(INTAKE_SERIES_DAYS as usize);
    for offset in (0..count).rev() {
        let day = today
            .checked_sub(Duration::days(offset))
            .ok_or_else(|| AppError::Internal {
                message: "intake series date underflow".into(),
            })?;
        dates.push(format_calendar_day(day));
    }
    Ok(dates)
}

fn to_row(
    raw: DashboardRepairRaw,
    now: OffsetDateTime,
    use_ready_anchor: bool,
) -> DashboardRepairRow {
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

pub(crate) fn local_calendar_date() -> Date {
    OffsetDateTime::now_local()
        .unwrap_or_else(|_| OffsetDateTime::now_utc())
        .date()
}

pub(crate) fn local_calendar_day() -> Result<String, AppError> {
    Ok(format_calendar_day(local_calendar_date()))
}

pub(crate) fn format_calendar_day(date: Date) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        date.year(),
        u8::from(date.month()),
        date.day()
    )
}

fn format_rfc3339(dt: OffsetDateTime) -> Result<String, AppError> {
    dt.format(&Rfc3339).map_err(|err| AppError::Internal {
        message: format!("timestamp format failed: {err}"),
    })
}
