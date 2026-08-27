//! Synthetic seed integration tests (temporary / in-memory DB only).

use crate::db::Db;
use crate::domain::seed::repository::count_rows;
use crate::domain::seed::types::SeedSyntheticDataInput;
use crate::domain::seed::seed_synthetic_data;
use crate::error::AppError;

#[test]
fn seed_rejects_without_confirm() {
    let db = Db::open_in_memory().expect("db");
    let err = seed_synthetic_data(
        db.conn(),
        SeedSyntheticDataInput {
            customers: Some(5),
            devices: Some(10),
            repairs: Some(15),
            confirm: false,
        },
    )
    .expect_err("confirm");
    assert!(matches!(
        err,
        AppError::Validation {
            field: Some(ref f),
            ..
        } if f == "confirm"
    ));
}

#[test]
fn seed_inserts_tiny_counts() {
    let db = Db::open_in_memory().expect("db");

    let before_c = count_rows(db.conn(), "customers").expect("c0");
    let before_d = count_rows(db.conn(), "devices").expect("d0");
    let before_r = count_rows(db.conn(), "repairs").expect("r0");

    let result = seed_synthetic_data(
        db.conn(),
        SeedSyntheticDataInput {
            customers: Some(5),
            devices: Some(10),
            repairs: Some(15),
            confirm: true,
        },
    )
    .expect("seed");

    assert_eq!(result.customers, 5);
    assert_eq!(result.devices, 10);
    assert_eq!(result.repairs, 15);

    let after_c = count_rows(db.conn(), "customers").expect("c1");
    let after_d = count_rows(db.conn(), "devices").expect("d1");
    let after_r = count_rows(db.conn(), "repairs").expect("r1");

    assert_eq!(after_c - before_c, 5);
    assert_eq!(after_d - before_d, 10);
    assert_eq!(after_r - before_r, 15);
}

#[test]
fn seed_rejects_over_cap() {
    let db = Db::open_in_memory().expect("db");
    let err = seed_synthetic_data(
        db.conn(),
        SeedSyntheticDataInput {
            customers: Some(20_001),
            devices: Some(0),
            repairs: Some(0),
            confirm: true,
        },
    )
    .expect_err("cap");
    assert!(matches!(
        err,
        AppError::Validation {
            field: Some(ref f),
            ..
        } if f == "customers"
    ));
}

#[test]
fn seed_rejects_devices_without_customers() {
    let db = Db::open_in_memory().expect("db");
    let err = seed_synthetic_data(
        db.conn(),
        SeedSyntheticDataInput {
            customers: Some(0),
            devices: Some(5),
            repairs: Some(0),
            confirm: true,
        },
    )
    .expect_err("devices need customers");
    assert!(matches!(err, AppError::Validation { .. }));
}


#[test]
fn search_timing_on_medium_seed() {
    use crate::domain::search::{global_search, GlobalSearchQuery};
    use std::time::Instant;

    let db = Db::open_in_memory().expect("db");
    let seeded = seed_synthetic_data(
        db.conn(),
        SeedSyntheticDataInput {
            customers: Some(1_000),
            devices: Some(2_000),
            repairs: Some(5_000),
            confirm: true,
        },
    )
    .expect("seed");
    eprintln!(
        "perf_seed customers={} devices={} repairs={} elapsed_ms={}",
        seeded.customers, seeded.devices, seeded.repairs, seeded.elapsed_ms
    );

    let start = Instant::now();
    let result = global_search(
        db.conn(),
        GlobalSearchQuery {
            query: "Customer 500".into(),
            limit_per_type: Some(10),
        },
    )
    .expect("search");
    let ms = start.elapsed().as_millis();
    eprintln!(
        "perf_search_ms={} customers={} devices={} repairs={}",
        ms,
        result.customers.len(),
        result.devices.len(),
        result.repairs.len()
    );
    // Target ≪ 500 ms; allow headroom on CI hosts.
    assert!(
        ms < 1_000,
        "global_search took {ms} ms on medium seed (want < 1000)"
    );
}
