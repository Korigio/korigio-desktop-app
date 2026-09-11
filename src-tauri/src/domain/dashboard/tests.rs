//! Home dashboard integration tests (in-memory DB only).

use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

use crate::db::Db;
use crate::domain::companies::{create_company, CompanyInput};
use crate::domain::customers::{create_customer, CustomerInput};
use crate::domain::dashboard::constants::{
    DASHBOARD_LIST_LIMIT, INTAKE_SERIES_DAYS, STALE_AFTER_DAYS,
};
use crate::domain::dashboard::get_home_dashboard;
use crate::domain::dashboard::service::{
    format_calendar_day, local_calendar_date, local_calendar_day,
};
use crate::domain::devices::{create_device, DeviceInput};
use crate::domain::repairs::constants::REPAIR_STATUSES;
use crate::domain::repairs::{create_repair, RepairInput};
use crate::domain::settings::{set_shop_settings, ShopSettingsInput};
use crate::domain::staff::{create_staff, sign_in_staff, sign_out_staff, StaffInput, StaffRole};
use crate::domain::team::{create_team, CreateTeamInput};

fn customer(db: &Db, name: &str, phone: Option<&str>) -> String {
    create_customer(
        db.conn(),
        CustomerInput {
            name: name.into(),
            phone: phone.map(str::to_string),
            email: None,
            address: None,
            notes: None,
        },
    )
    .expect("customer")
    .id
}

fn device(db: &Db, customer_id: &str, serial: &str) -> String {
    create_device(
        db.conn(),
        DeviceInput {
            customer_id: customer_id.to_string(),
            device_type: Some("Phone".into()),
            manufacturer: Some("Acme".into()),
            model: Some("X1".into()),
            serial_number: Some(serial.into()),
            accessories: None,
            notes: None,
        },
    )
    .expect("device")
    .id
}

fn company(db: &Db) -> String {
    create_company(
        db.conn(),
        CompanyInput {
            legal_name: "Test Company".into(),
            trade_name: None,
            tax_id: None,
            address: None,
            phone: None,
            email: None,
            website: None,
        },
    )
    .expect("company")
    .id
}

fn sample_repair(customer_id: &str, device_id: &str, company_id: &str) -> RepairInput {
    RepairInput {
        customer_id: customer_id.to_string(),
        device_id: device_id.to_string(),
        company_id: company_id.to_string(),
        status: None,
        reported_problem: Some("Screen cracked".into()),
        accessories_received: None,
        device_condition: None,
        diagnosis_notes: None,
        work_performed: None,
        notes: None,
        expected_pickup_at: None,
        estimate_base_cents: None,
        estimate_discount_bps: None,
    }
}

fn set_status(db: &Db, id: &str, status: &str) {
    let now = OffsetDateTime::now_utc().format(&Rfc3339).expect("now");
    match status {
        "ready" => {
            db.conn()
                .execute(
                    "UPDATE repairs SET status = ?1, ready_at = ?2, updated_at = ?2 WHERE id = ?3",
                    rusqlite::params![status, now.as_str(), id],
                )
                .expect("status");
        }
        "collected" => {
            db.conn()
                .execute(
                    "UPDATE repairs SET status = ?1, collected_at = ?2, updated_at = ?2 WHERE id = ?3",
                    rusqlite::params![status, now.as_str(), id],
                )
                .expect("status");
        }
        _ => {
            db.conn()
                .execute(
                    "UPDATE repairs SET status = ?1, updated_at = ?2 WHERE id = ?3",
                    rusqlite::params![status, now.as_str(), id],
                )
                .expect("status");
        }
    }
}

fn count_for(status_counts: &[(String, i64)], status: &str) -> i64 {
    status_counts
        .iter()
        .find(|(s, _)| s == status)
        .map(|(_, c)| *c)
        .unwrap_or(-1)
}

#[test]
fn empty_dashboard_returns_zero_counts_for_all_statuses() {
    let db = Db::open_in_memory().expect("db");
    let dash = get_home_dashboard(db.conn()).expect("dashboard");

    assert_eq!(dash.stale_after_days, STALE_AFTER_DAYS);
    assert_eq!(dash.status_counts.len(), REPAIR_STATUSES.len());
    for (entry, expected) in dash.status_counts.iter().zip(REPAIR_STATUSES.iter()) {
        assert_eq!(entry.status, *expected);
        assert_eq!(entry.count, 0);
    }
    assert!(dash.ready_for_pickup.is_empty());
    assert!(dash.stale_repairs.is_empty());
    assert_eq!(dash.today.received, 0);
    assert_eq!(dash.today.collected, 0);
    assert_eq!(dash.intake_by_day.len(), INTAKE_SERIES_DAYS as usize);
    for day in &dash.intake_by_day {
        assert_eq!(day.received, 0);
        assert_eq!(day.collected, 0);
    }
    let today = local_calendar_day().expect("local today");
    assert_eq!(
        dash.intake_by_day.last().map(|d| d.date.as_str()),
        Some(today.as_str())
    );
    assert_eq!(dash.stale_count, 0);
    assert_eq!(dash.currency, "EUR");
    let revenue = dash.revenue.as_ref().expect("solo revenue");
    assert_eq!(revenue.collected_gross_cents_today, 0);
    assert_eq!(revenue.collected_gross_cents_week, 0);
    assert_eq!(revenue.open_estimate_gross_cents, 0);
    let revenue_by_day = dash.revenue_by_day.as_ref().expect("solo revenue series");
    assert_eq!(revenue_by_day.len(), INTAKE_SERIES_DAYS as usize);
    for day in revenue_by_day {
        assert_eq!(day.collected_gross_cents, 0);
    }
    assert_revenue_invariants(&dash);
}

#[test]
fn status_counts_and_today_received() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Alice", Some("555-0100"));
    let device_id = device(&db, &customer_id, "SN-1");

    let received = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("r1");
    let diagnosis = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("r2");
    set_status(&db, &diagnosis.id.clone(), "diagnosis");

    let ready = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("r3");
    set_status(&db, &ready.id.clone(), "ready");
    stamp_on_local_today(&db, &received.id, false);
    stamp_on_local_today(&db, &diagnosis.id, false);
    stamp_on_local_today(&db, &ready.id, false);

    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    let counts: Vec<(String, i64)> = dash
        .status_counts
        .iter()
        .map(|s| (s.status.clone(), s.count))
        .collect();

    assert_eq!(count_for(&counts, "received"), 1);
    assert_eq!(count_for(&counts, "diagnosis"), 1);
    assert_eq!(count_for(&counts, "ready"), 1);
    assert_eq!(count_for(&counts, "collected"), 0);
    assert_eq!(dash.today.received, 3);
    assert_eq!(dash.today.collected, 0);
    assert_today_matches_series(&dash);
    assert_eq!(dash.ready_for_pickup.len(), 1);
    assert_eq!(dash.ready_for_pickup[0].id, ready.id.clone());
    assert_eq!(dash.ready_for_pickup[0].customer_name, "Alice");
    assert_eq!(
        dash.ready_for_pickup[0].customer_phone.as_deref(),
        Some("555-0100")
    );
    assert!(dash.ready_for_pickup[0].ready_at.is_some());
    assert_eq!(received.status, "received");
}

#[test]
fn ready_list_orders_by_ready_at_ascending() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Bob", None);
    let device_id = device(&db, &customer_id, "SN-2");

    let first = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("first");
    set_status(&db, &first.id.clone(), "ready");

    let second = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("second");
    set_status(&db, &second.id.clone(), "ready");

    // Force older ready_at on the second repair so order is deterministic.
    let older = (OffsetDateTime::now_utc() - Duration::days(2))
        .format(&Rfc3339)
        .expect("fmt");
    let newer = (OffsetDateTime::now_utc() - Duration::days(1))
        .format(&Rfc3339)
        .expect("fmt");
    db.conn()
        .execute(
            "UPDATE repairs SET ready_at = ?1 WHERE id = ?2",
            rusqlite::params![newer.as_str(), first.id.clone()],
        )
        .expect("bump first");
    db.conn()
        .execute(
            "UPDATE repairs SET ready_at = ?1 WHERE id = ?2",
            rusqlite::params![older.as_str(), second.id.clone()],
        )
        .expect("bump second");

    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    assert_eq!(dash.ready_for_pickup.len(), 2);
    assert_eq!(dash.ready_for_pickup[0].id, second.id.clone());
    assert_eq!(dash.ready_for_pickup[1].id, first.id.clone());
    assert!(dash.ready_for_pickup[0].days_in_status >= 1);
}

#[test]
fn stale_list_includes_old_open_repairs_excludes_fresh_and_terminal() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Carol", None);
    let device_id = device(&db, &customer_id, "SN-3");

    let stale = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("stale");
    set_status(&db, &stale.id.clone(), "in_repair");

    let fresh = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("fresh");
    set_status(&db, &fresh.id.clone(), "diagnosis");

    let collected = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("col");
    set_status(&db, &collected.id.clone(), "collected");

    let cancelled = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("can");
    set_status(&db, &cancelled.id.clone(), "cancelled");

    let old = (OffsetDateTime::now_utc() - Duration::days(i64::from(STALE_AFTER_DAYS) + 2))
        .format(&Rfc3339)
        .expect("fmt");
    let recent = (OffsetDateTime::now_utc() - Duration::days(1))
        .format(&Rfc3339)
        .expect("fmt");

    db.conn()
        .execute(
            "UPDATE repairs SET updated_at = ?1 WHERE id = ?2",
            rusqlite::params![old.as_str(), stale.id.clone()],
        )
        .expect("age stale");
    db.conn()
        .execute(
            "UPDATE repairs SET updated_at = ?1 WHERE id = ?2",
            rusqlite::params![recent.as_str(), fresh.id.clone()],
        )
        .expect("age fresh");
    db.conn()
        .execute(
            "UPDATE repairs SET updated_at = ?1 WHERE id = ?2",
            rusqlite::params![old.as_str(), collected.id.clone()],
        )
        .expect("age collected");
    db.conn()
        .execute(
            "UPDATE repairs SET updated_at = ?1 WHERE id = ?2",
            rusqlite::params![old.as_str(), cancelled.id.clone()],
        )
        .expect("age cancelled");

    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    let stale_ids: Vec<String> = dash.stale_repairs.iter().map(|r| r.id.clone()).collect();
    assert_eq!(stale_ids, vec![stale.id.clone()]);
    assert_eq!(dash.stale_count, 1);
    assert!(dash.stale_repairs[0].days_in_status >= i64::from(STALE_AFTER_DAYS));
}

#[test]
fn archived_repairs_excluded_from_all_sections() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Dan", None);
    let device_id = device(&db, &customer_id, "SN-4");

    let repair = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("repair");
    set_status(&db, &repair.id.clone(), "ready");

    let old = (OffsetDateTime::now_utc() - Duration::days(i64::from(STALE_AFTER_DAYS) + 3))
        .format(&Rfc3339)
        .expect("fmt");
    db.conn()
        .execute(
            "UPDATE repairs SET updated_at = ?1, archived_at = ?2 WHERE id = ?3",
            rusqlite::params![old.as_str(), old.as_str(), repair.id.clone()],
        )
        .expect("archive");

    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    assert!(dash.ready_for_pickup.is_empty());
    assert!(dash.stale_repairs.is_empty());
    assert_eq!(dash.today.received, 0);
    assert_eq!(dash.stale_count, 0);
    assert_eq!(dash.intake_by_day.len(), INTAKE_SERIES_DAYS as usize);
    for day in &dash.intake_by_day {
        assert_eq!(day.received, 0, "date={}", day.date);
        assert_eq!(day.collected, 0, "date={}", day.date);
    }
    for entry in &dash.status_counts {
        assert_eq!(entry.count, 0, "status={}", entry.status);
    }
}

#[test]
fn today_collected_uses_collected_at_date_prefix() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Eve", None);
    let device_id = device(&db, &customer_id, "SN-5");

    let repair = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("repair");
    set_status(&db, &repair.id.clone(), "collected");
    stamp_on_local_today(&db, &repair.id, true);

    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    assert_eq!(dash.today.collected, 1);
    assert_eq!(dash.today.received, 1);
    assert_today_matches_series(&dash);
}

fn assert_today_matches_series(dash: &crate::domain::dashboard::HomeDashboard) {
    let last = dash
        .intake_by_day
        .last()
        .expect("intake series must include today");
    assert_eq!(dash.today.received, last.received);
    assert_eq!(dash.today.collected, last.collected);
}

fn assert_revenue_invariants(dash: &crate::domain::dashboard::HomeDashboard) {
    let revenue = dash.revenue.as_ref().expect("revenue visible");
    let revenue_by_day = dash
        .revenue_by_day
        .as_ref()
        .expect("revenue series visible");
    assert_eq!(revenue_by_day.len(), INTAKE_SERIES_DAYS as usize);
    let last = revenue_by_day
        .last()
        .expect("revenue series must include today");
    assert_eq!(
        revenue.collected_gross_cents_today,
        last.collected_gross_cents
    );
    let week: i64 = revenue_by_day
        .iter()
        .map(|day| day.collected_gross_cents)
        .sum();
    assert_eq!(revenue.collected_gross_cents_week, week);
    for (intake, day) in dash.intake_by_day.iter().zip(revenue_by_day.iter()) {
        assert_eq!(intake.date, day.date);
    }
}

fn set_estimate_cents(db: &Db, id: &str, cents: i64) {
    db.conn()
        .execute(
            "UPDATE repairs SET estimate_gross_cents = ?1 WHERE id = ?2",
            rusqlite::params![cents, id],
        )
        .expect("estimate cents");
}

fn rfc3339_on_local_day(day: &str) -> String {
    format!("{day}T12:00:00Z")
}

fn stamp_on_local_today(db: &Db, id: &str, also_collected: bool) {
    let today = local_calendar_day().expect("local today");
    let stamp = rfc3339_on_local_day(&today);
    if also_collected {
        db.conn()
            .execute(
                "UPDATE repairs SET received_at = ?1, collected_at = ?1 WHERE id = ?2",
                rusqlite::params![stamp.as_str(), id],
            )
            .expect("stamp collected today");
    } else {
        db.conn()
            .execute(
                "UPDATE repairs SET received_at = ?1 WHERE id = ?2",
                rusqlite::params![stamp.as_str(), id],
            )
            .expect("stamp received today");
    }
}

fn local_day_offset(days_ago: i64) -> String {
    let day = local_calendar_date()
        .checked_sub(Duration::days(days_ago))
        .expect("date underflow");
    format_calendar_day(day)
}

#[test]
fn intake_series_buckets_received_and_collected_on_target_day() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Fay", None);
    let device_id = device(&db, &customer_id, "SN-6");

    let received = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("received");
    let collected = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("collected");
    set_status(&db, &collected.id.clone(), "collected");

    let target = local_day_offset(3);
    let stamp = rfc3339_on_local_day(&target);
    db.conn()
        .execute(
            "UPDATE repairs SET received_at = ?1 WHERE id = ?2",
            rusqlite::params![stamp.as_str(), received.id.clone()],
        )
        .expect("date received");
    db.conn()
        .execute(
            "UPDATE repairs SET received_at = ?1, collected_at = ?1 WHERE id = ?2",
            rusqlite::params![stamp.as_str(), collected.id.clone()],
        )
        .expect("date collected");

    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    assert_eq!(dash.intake_by_day.len(), INTAKE_SERIES_DAYS as usize);
    assert_today_matches_series(&dash);

    for (index, day) in dash.intake_by_day.iter().enumerate() {
        if day.date == target {
            assert_eq!(day.received, 2, "target slot {index}");
            assert_eq!(day.collected, 1, "target slot {index}");
        } else {
            assert_eq!(day.received, 0, "other day {}", day.date);
            assert_eq!(day.collected, 0, "other day {}", day.date);
        }
    }
}

#[test]
fn intake_series_excludes_archived_repairs() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Gus", None);
    let device_id = device(&db, &customer_id, "SN-7");

    let repair = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("repair");
    set_status(&db, &repair.id.clone(), "collected");

    let target = local_day_offset(3);
    let stamp = rfc3339_on_local_day(&target);
    let archived = OffsetDateTime::now_utc().format(&Rfc3339).expect("now");
    db.conn()
        .execute(
            "UPDATE repairs SET received_at = ?1, collected_at = ?1, archived_at = ?2 WHERE id = ?3",
            rusqlite::params![stamp.as_str(), archived.as_str(), repair.id.clone()],
        )
        .expect("archive");

    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    assert_eq!(dash.intake_by_day.len(), INTAKE_SERIES_DAYS as usize);
    for day in &dash.intake_by_day {
        assert_eq!(day.received, 0, "date={}", day.date);
        assert_eq!(day.collected, 0, "date={}", day.date);
    }
    assert_eq!(dash.today.received, 0);
    assert_eq!(dash.today.collected, 0);
}

#[test]
fn stale_count_includes_rows_beyond_list_limit() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Hal", None);
    let device_id = device(&db, &customer_id, "SN-8");

    let extra = i64::from(DASHBOARD_LIST_LIMIT) + 1;
    let mut ids = Vec::new();
    for _ in 0..extra {
        let repair = create_repair(
            db.conn(),
            sample_repair(&customer_id, &device_id, &company_id),
        )
        .expect("repair");
        set_status(&db, &repair.id.clone(), "in_repair");
        ids.push(repair.id);
    }

    let old = (OffsetDateTime::now_utc() - Duration::days(i64::from(STALE_AFTER_DAYS) + 2))
        .format(&Rfc3339)
        .expect("fmt");
    for id in &ids {
        db.conn()
            .execute(
                "UPDATE repairs SET updated_at = ?1 WHERE id = ?2",
                rusqlite::params![old.as_str(), id],
            )
            .expect("age");
    }

    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    assert_eq!(dash.stale_repairs.len(), DASHBOARD_LIST_LIMIT as usize);
    assert_eq!(dash.stale_count, extra);
}

#[test]
fn revenue_collected_today_fills_today_last_and_week() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Ivy", None);
    let device_id = device(&db, &customer_id, "SN-9");

    let repair = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("repair");
    set_status(&db, &repair.id.clone(), "collected");
    set_estimate_cents(&db, &repair.id, 11_900);
    stamp_on_local_today(&db, &repair.id, true);

    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    assert_revenue_invariants(&dash);
    let revenue = dash.revenue.as_ref().expect("revenue visible");
    assert_eq!(revenue.collected_gross_cents_today, 11_900);
    assert_eq!(revenue.collected_gross_cents_week, 11_900);
    assert_eq!(
        dash.revenue_by_day
            .as_ref()
            .and_then(|days| days.last().map(|d| d.collected_gross_cents)),
        Some(11_900)
    );
    for day in dash
        .revenue_by_day
        .as_ref()
        .expect("revenue series visible")
        .iter()
        .rev()
        .skip(1)
    {
        assert_eq!(day.collected_gross_cents, 0, "date={}", day.date);
    }
}

#[test]
fn revenue_collected_on_local_day_minus_three_only_fills_that_slot() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Jade", None);
    let device_id = device(&db, &customer_id, "SN-10");

    let repair = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("repair");
    set_status(&db, &repair.id.clone(), "collected");
    set_estimate_cents(&db, &repair.id, 5_500);

    let target = local_day_offset(3);
    let stamp = rfc3339_on_local_day(&target);
    db.conn()
        .execute(
            "UPDATE repairs SET collected_at = ?1 WHERE id = ?2",
            rusqlite::params![stamp.as_str(), repair.id.clone()],
        )
        .expect("date collected");

    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    assert_revenue_invariants(&dash);
    let revenue = dash.revenue.as_ref().expect("revenue visible");
    assert_eq!(revenue.collected_gross_cents_today, 0);
    assert_eq!(revenue.collected_gross_cents_week, 5_500);
    for day in dash
        .revenue_by_day
        .as_ref()
        .expect("revenue series visible")
    {
        if day.date == target {
            assert_eq!(day.collected_gross_cents, 5_500);
        } else {
            assert_eq!(day.collected_gross_cents, 0, "date={}", day.date);
        }
    }
}

#[test]
fn revenue_excludes_archived_collected_estimate() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Kim", None);
    let device_id = device(&db, &customer_id, "SN-11");

    let repair = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("repair");
    set_status(&db, &repair.id.clone(), "collected");
    set_estimate_cents(&db, &repair.id, 11_900);

    let archived = OffsetDateTime::now_utc().format(&Rfc3339).expect("now");
    db.conn()
        .execute(
            "UPDATE repairs SET archived_at = ?1 WHERE id = ?2",
            rusqlite::params![archived.as_str(), repair.id.clone()],
        )
        .expect("archive");

    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    assert_revenue_invariants(&dash);
    let revenue = dash.revenue.as_ref().expect("revenue visible");
    assert_eq!(revenue.collected_gross_cents_today, 0);
    assert_eq!(revenue.collected_gross_cents_week, 0);
    assert_eq!(revenue.open_estimate_gross_cents, 0);
    for day in dash
        .revenue_by_day
        .as_ref()
        .expect("revenue series visible")
    {
        assert_eq!(day.collected_gross_cents, 0, "date={}", day.date);
    }
}

#[test]
fn revenue_cancelled_or_open_estimate_not_in_collected_totals() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Lee", None);
    let device_id = device(&db, &customer_id, "SN-12");

    let cancelled = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("cancelled");
    set_status(&db, &cancelled.id.clone(), "cancelled");
    set_estimate_cents(&db, &cancelled.id, 8_000);

    let open = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("open");
    set_estimate_cents(&db, &open.id, 4_200);

    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    assert_revenue_invariants(&dash);
    let revenue = dash.revenue.as_ref().expect("revenue visible");
    assert_eq!(revenue.collected_gross_cents_today, 0);
    assert_eq!(revenue.collected_gross_cents_week, 0);
    for day in dash
        .revenue_by_day
        .as_ref()
        .expect("revenue series visible")
    {
        assert_eq!(day.collected_gross_cents, 0, "date={}", day.date);
    }
}

#[test]
fn revenue_open_estimate_includes_in_repair_excludes_collected() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Mia", None);
    let device_id = device(&db, &customer_id, "SN-13");

    let open = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("open");
    set_status(&db, &open.id.clone(), "in_repair");
    set_estimate_cents(&db, &open.id, 7_500);

    let collected = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("collected");
    set_status(&db, &collected.id.clone(), "collected");
    set_estimate_cents(&db, &collected.id, 3_300);
    stamp_on_local_today(&db, &collected.id, true);

    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    assert_revenue_invariants(&dash);
    let revenue = dash.revenue.as_ref().expect("revenue visible");
    assert_eq!(revenue.open_estimate_gross_cents, 7_500);
    assert_eq!(revenue.collected_gross_cents_today, 3_300);
    assert_eq!(revenue.collected_gross_cents_week, 3_300);
}

#[test]
fn revenue_currency_follows_shop_settings() {
    let db = Db::open_in_memory().expect("db");
    set_shop_settings(
        db.conn(),
        ShopSettingsInput {
            tax_rate_percent: None,
            currency: Some("usd".into()),
        },
    )
    .expect("currency");

    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    assert_eq!(dash.currency, "USD");
    assert_revenue_invariants(&dash);
}

fn shop_team(db: &Db) {
    create_team(
        db.conn(),
        CreateTeamInput {
            name: "Shop".into(),
            member_name: "Ada".into(),
        },
    )
    .expect("team");
}

#[test]
fn team_admin_sees_revenue() {
    let db = Db::open_in_memory().expect("db");
    shop_team(&db);
    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    assert!(dash.revenue.is_some());
    assert!(dash.revenue_by_day.is_some());
    assert_revenue_invariants(&dash);
}

#[test]
fn team_staff_hides_revenue() {
    let db = Db::open_in_memory().expect("db");
    shop_team(&db);
    let staff = create_staff(
        db.conn(),
        StaffInput {
            name: "Bob".into(),
            pin: "5678".into(),
            role: Some(StaffRole::Staff),
        },
    )
    .expect("staff");
    sign_in_staff(db.conn(), &staff.id, "5678").expect("sign in");
    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    assert!(dash.revenue.is_none());
    assert!(dash.revenue_by_day.is_none());
}

#[test]
fn team_no_session_hides_revenue() {
    let db = Db::open_in_memory().expect("db");
    shop_team(&db);
    sign_out_staff(db.conn()).expect("sign out");
    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    assert!(dash.revenue.is_none());
    assert!(dash.revenue_by_day.is_none());
}
