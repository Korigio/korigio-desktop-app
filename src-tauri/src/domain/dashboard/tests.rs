//! Home dashboard integration tests (in-memory DB only).

use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

use crate::db::Db;
use crate::domain::companies::{CompanyInput, create_company};
use crate::domain::customers::{CustomerInput, create_customer};
use crate::domain::dashboard::constants::STALE_AFTER_DAYS;
use crate::domain::dashboard::get_home_dashboard;
use crate::domain::devices::{DeviceInput, create_device};
use crate::domain::repairs::constants::REPAIR_STATUSES;
use crate::domain::repairs::{RepairInput, create_repair};

fn customer(db: &Db, name: &str, phone: Option<&str>) -> i64 {
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

fn device(db: &Db, customer_id: i64, serial: &str) -> i64 {
    create_device(
        db.conn(),
        DeviceInput {
            customer_id,
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


fn company(db: &Db) -> i64 {
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

fn sample_repair(customer_id: i64, device_id: i64, company_id: i64) -> RepairInput {
    RepairInput {
        customer_id,
        device_id,
        company_id,
        status: None,
        reported_problem: Some("Screen cracked".into()),
        accessories_received: None,
        device_condition: None,
        diagnosis_notes: None,
        work_performed: None,
        notes: None,
        expected_pickup_at: None,
    }
}

fn set_status(db: &Db, id: i64, status: &str) {
    let now = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("now");
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
}

#[test]
fn status_counts_and_today_received() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Alice", Some("555-0100"));
    let device_id = device(&db, customer_id, "SN-1");

    let received = create_repair(db.conn(), sample_repair(customer_id, device_id, company_id)).expect("r1");
    let diagnosis = create_repair(db.conn(), sample_repair(customer_id, device_id, company_id)).expect("r2");
    set_status(&db, diagnosis.id, "diagnosis");

    let ready = create_repair(db.conn(), sample_repair(customer_id, device_id, company_id)).expect("r3");
    set_status(&db, ready.id, "ready");

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
    assert_eq!(dash.ready_for_pickup.len(), 1);
    assert_eq!(dash.ready_for_pickup[0].id, ready.id);
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
    let device_id = device(&db, customer_id, "SN-2");

    let first = create_repair(db.conn(), sample_repair(customer_id, device_id, company_id)).expect("first");
    set_status(&db, first.id, "ready");

    let second = create_repair(db.conn(), sample_repair(customer_id, device_id, company_id)).expect("second");
    set_status(&db, second.id, "ready");

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
            rusqlite::params![newer.as_str(), first.id],
        )
        .expect("bump first");
    db.conn()
        .execute(
            "UPDATE repairs SET ready_at = ?1 WHERE id = ?2",
            rusqlite::params![older.as_str(), second.id],
        )
        .expect("bump second");

    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    assert_eq!(dash.ready_for_pickup.len(), 2);
    assert_eq!(dash.ready_for_pickup[0].id, second.id);
    assert_eq!(dash.ready_for_pickup[1].id, first.id);
    assert!(dash.ready_for_pickup[0].days_in_status >= 1);
}

#[test]
fn stale_list_includes_old_open_repairs_excludes_fresh_and_terminal() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Carol", None);
    let device_id = device(&db, customer_id, "SN-3");

    let stale = create_repair(db.conn(), sample_repair(customer_id, device_id, company_id)).expect("stale");
    set_status(&db, stale.id, "in_repair");

    let fresh = create_repair(db.conn(), sample_repair(customer_id, device_id, company_id)).expect("fresh");
    set_status(&db, fresh.id, "diagnosis");

    let collected = create_repair(db.conn(), sample_repair(customer_id, device_id, company_id)).expect("col");
    set_status(&db, collected.id, "collected");

    let cancelled = create_repair(db.conn(), sample_repair(customer_id, device_id, company_id)).expect("can");
    set_status(&db, cancelled.id, "cancelled");

    let old = (OffsetDateTime::now_utc() - Duration::days(i64::from(STALE_AFTER_DAYS) + 2))
        .format(&Rfc3339)
        .expect("fmt");
    let recent = (OffsetDateTime::now_utc() - Duration::days(1))
        .format(&Rfc3339)
        .expect("fmt");

    db.conn()
        .execute(
            "UPDATE repairs SET updated_at = ?1 WHERE id = ?2",
            rusqlite::params![old.as_str(), stale.id],
        )
        .expect("age stale");
    db.conn()
        .execute(
            "UPDATE repairs SET updated_at = ?1 WHERE id = ?2",
            rusqlite::params![recent.as_str(), fresh.id],
        )
        .expect("age fresh");
    db.conn()
        .execute(
            "UPDATE repairs SET updated_at = ?1 WHERE id = ?2",
            rusqlite::params![old.as_str(), collected.id],
        )
        .expect("age collected");
    db.conn()
        .execute(
            "UPDATE repairs SET updated_at = ?1 WHERE id = ?2",
            rusqlite::params![old.as_str(), cancelled.id],
        )
        .expect("age cancelled");

    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    let stale_ids: Vec<i64> = dash.stale_repairs.iter().map(|r| r.id).collect();
    assert_eq!(stale_ids, vec![stale.id]);
    assert!(dash.stale_repairs[0].days_in_status >= i64::from(STALE_AFTER_DAYS));
}

#[test]
fn archived_repairs_excluded_from_all_sections() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Dan", None);
    let device_id = device(&db, customer_id, "SN-4");

    let repair = create_repair(db.conn(), sample_repair(customer_id, device_id, company_id)).expect("repair");
    set_status(&db, repair.id, "ready");

    let old = (OffsetDateTime::now_utc() - Duration::days(i64::from(STALE_AFTER_DAYS) + 3))
        .format(&Rfc3339)
        .expect("fmt");
    db.conn()
        .execute(
            "UPDATE repairs SET updated_at = ?1, archived_at = ?2 WHERE id = ?3",
            rusqlite::params![old.as_str(), old.as_str(), repair.id],
        )
        .expect("archive");

    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    assert!(dash.ready_for_pickup.is_empty());
    assert!(dash.stale_repairs.is_empty());
    assert_eq!(dash.today.received, 0);
    for entry in &dash.status_counts {
        assert_eq!(entry.count, 0, "status={}", entry.status);
    }
}

#[test]
fn today_collected_uses_collected_at_date_prefix() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Eve", None);
    let device_id = device(&db, customer_id, "SN-5");

    let repair = create_repair(db.conn(), sample_repair(customer_id, device_id, company_id)).expect("repair");
    set_status(&db, repair.id, "collected");

    let dash = get_home_dashboard(db.conn()).expect("dashboard");
    assert_eq!(dash.today.collected, 1);
    assert_eq!(dash.today.received, 1);
}
