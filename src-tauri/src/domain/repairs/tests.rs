//! Repair domain integration tests (temporary / in-memory DB only).

use time::OffsetDateTime;

use crate::db::Db;
use crate::domain::companies::{CompanyInput, create_company};
use crate::domain::customers::{CustomerInput, archive_customer, create_customer};
use crate::domain::devices::{DeviceInput, create_device};
use crate::domain::repairs::repository;
use crate::domain::repairs::{
    RepairInput, RepairListQuery, create_repair, get_repair, list_repairs, update_repair,
};
use crate::error::AppError;

fn local_year() -> i32 {
    OffsetDateTime::now_local()
        .map(|dt| dt.year())
        .unwrap_or_else(|_| OffsetDateTime::now_utc().year())
}

fn customer(db: &Db, name: &str) -> i64 {
    create_customer(
        db.conn(),
        CustomerInput {
            name: name.into(),
            phone: None,
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
        accessories_received: Some("Charger".into()),
        device_condition: Some("Scratched back".into()),
        diagnosis_notes: None,
        work_performed: None,
        notes: None,
        expected_pickup_at: None,
    }
}

#[test]
fn create_allocates_year_prefixed_repair_number() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, customer_id, "SN-1");

    let created = create_repair(db.conn(), sample_repair(customer_id, device_id, company_id)).expect("create");
    let year = local_year();
    assert_eq!(created.repair_number, format!("{year}-000001"));
    assert_eq!(created.status, "received");
    assert_eq!(created.company_id, Some(company_id));
    assert!(!created.received_at.is_empty());
}

#[test]
fn second_create_increments_sequence() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, customer_id, "SN-1");

    let first = create_repair(db.conn(), sample_repair(customer_id, device_id, company_id)).expect("first");
    let second = create_repair(db.conn(), sample_repair(customer_id, device_id, company_id)).expect("second");

    let year = local_year();
    assert_eq!(first.repair_number, format!("{year}-000001"));
    assert_eq!(second.repair_number, format!("{year}-000002"));
}

#[test]
fn repair_number_sequence_is_per_year() {
    let db = Db::open_in_memory().expect("db");
    let conn = db.conn();
    let tx = conn.unchecked_transaction().expect("tx");

    let n1 = repository::allocate_repair_number(&tx, 2030).expect("n1");
    let n2 = repository::allocate_repair_number(&tx, 2030).expect("n2");
    let n3 = repository::allocate_repair_number(&tx, 2031).expect("n3");
    tx.commit().expect("commit");

    assert_eq!(n1, "2030-000001");
    assert_eq!(n2, "2030-000002");
    assert_eq!(n3, "2031-000001");
}

#[test]
fn rejects_wrong_device_ownership() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let owner_a = customer(&db, "A");
    let owner_b = customer(&db, "B");
    let device_b = device(&db, owner_b, "SN-B");

    let err = create_repair(
        db.conn(),
        RepairInput {
            customer_id: owner_a,
            device_id: device_b,
            company_id,
            status: None,
            reported_problem: None,
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: None,
        },
    )
    .expect_err("ownership");
    assert_eq!(err.code(), "validation");
}

#[test]
fn rejects_archived_customer() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Archived");
    let device_id = device(&db, customer_id, "SN-1");
    archive_customer(db.conn(), customer_id).expect("archive");

    let err = create_repair(db.conn(), sample_repair(customer_id, device_id, company_id)).expect_err("blocked");
    assert_eq!(err.code(), "validation");
}

#[test]
fn list_filters_by_customer_device_status_and_query() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, customer_id, "SN-1");

    let repair = create_repair(db.conn(), sample_repair(customer_id, device_id, company_id)).expect("create");

    let by_customer = list_repairs(
        db.conn(),
        RepairListQuery {
            query: None,
            customer_id: Some(customer_id),
            device_id: None,
            status: None,
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("by customer");
    assert_eq!(by_customer.total, 1);

    let by_device = list_repairs(
        db.conn(),
        RepairListQuery {
            query: None,
            customer_id: None,
            device_id: Some(device_id),
            status: None,
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("by device");
    assert_eq!(by_device.total, 1);

    let by_status = list_repairs(
        db.conn(),
        RepairListQuery {
            query: None,
            customer_id: None,
            device_id: None,
            status: Some("received".into()),
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("by status");
    assert_eq!(by_status.total, 1);

    let by_query = list_repairs(
        db.conn(),
        RepairListQuery {
            query: Some(repair.repair_number.clone()),
            customer_id: None,
            device_id: None,
            status: None,
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("by query");
    assert_eq!(by_query.total, 1);
    assert_eq!(by_query.items[0].id, repair.id);

    let empty = list_repairs(
        db.conn(),
        RepairListQuery {
            query: None,
            customer_id: None,
            device_id: None,
            status: Some("ready".into()),
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("empty");
    assert_eq!(empty.total, 0);
}

#[test]
fn list_repairs_finds_by_phone_serial_and_reported_problem() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = create_customer(
        db.conn(),
        CustomerInput {
            name: "Phone Owner".into(),
            phone: Some("+34 600 111 222".into()),
            email: None,
            address: None,
            notes: None,
        },
    )
    .expect("customer")
    .id;
    let device_id = create_device(
        db.conn(),
        DeviceInput {
            customer_id,
            device_type: Some("Laptop".into()),
            manufacturer: Some("Lenovo".into()),
            model: Some("T14".into()),
            serial_number: Some("SERIAL-FIND-ME".into()),
            accessories: None,
            notes: None,
        },
    )
    .expect("device")
    .id;

    let repair = create_repair(
        db.conn(),
        RepairInput {
            customer_id,
            device_id,
            company_id,
            status: None,
            reported_problem: Some("Won't boot after update".into()),
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: None,
        },
    )
    .expect("create");

    let by_phone = list_repairs(
        db.conn(),
        RepairListQuery {
            query: Some("600 111".into()),
            customer_id: None,
            device_id: None,
            status: None,
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("by phone");
    assert_eq!(by_phone.total, 1);
    assert_eq!(by_phone.items[0].id, repair.id);

    let by_serial = list_repairs(
        db.conn(),
        RepairListQuery {
            query: Some("SERIAL-FIND".into()),
            customer_id: None,
            device_id: None,
            status: None,
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("by serial");
    assert_eq!(by_serial.total, 1);
    assert_eq!(by_serial.items[0].id, repair.id);

    let by_problem = list_repairs(
        db.conn(),
        RepairListQuery {
            query: Some("won't boot".into()),
            customer_id: None,
            device_id: None,
            status: None,
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("by problem");
    assert_eq!(by_problem.total, 1);
    assert_eq!(by_problem.items[0].id, repair.id);

    let by_manufacturer = list_repairs(
        db.conn(),
        RepairListQuery {
            query: Some("Lenovo".into()),
            customer_id: None,
            device_id: None,
            status: None,
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("by manufacturer");
    assert_eq!(by_manufacturer.total, 1);
}

#[test]
fn update_status_sets_ready_at_once() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, customer_id, "SN-1");
    let created = create_repair(db.conn(), sample_repair(customer_id, device_id, company_id)).expect("create");
    assert!(created.ready_at.is_none());

    let ready = update_repair(
        db.conn(),
        created.id,
        RepairInput {
            customer_id,
            device_id,
            company_id,
            status: Some("ready".into()),
            reported_problem: None,
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: None,
        },
    )
    .expect("ready");
    assert_eq!(ready.status, "ready");
    let ready_at = ready.ready_at.clone().expect("ready_at set");

    let back_to_repair = update_repair(
        db.conn(),
        created.id,
        RepairInput {
            customer_id,
            device_id,
            company_id,
            status: Some("in_repair".into()),
            reported_problem: None,
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: None,
        },
    )
    .expect("in_repair");
    assert_eq!(back_to_repair.ready_at.as_deref(), Some(ready_at.as_str()));

    let ready_again = update_repair(
        db.conn(),
        created.id,
        RepairInput {
            customer_id,
            device_id,
            company_id,
            status: Some("ready".into()),
            reported_problem: None,
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: None,
        },
    )
    .expect("ready again");
    assert_eq!(ready_again.ready_at.as_deref(), Some(ready_at.as_str()));

    let fetched = get_repair(db.conn(), created.id).expect("get");
    assert_eq!(fetched.ready_at.as_deref(), Some(ready_at.as_str()));
}

#[test]
fn update_keeps_customer_and_device_ownership() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let other_customer = customer(&db, "Other");
    let device_id = device(&db, customer_id, "SN-1");
    let other_device = device(&db, other_customer, "SN-2");

    let created = create_repair(db.conn(), sample_repair(customer_id, device_id, company_id)).expect("create");

    let updated = update_repair(
        db.conn(),
        created.id,
        RepairInput {
            customer_id: other_customer,
            device_id: other_device,
            company_id,
            status: None,
            reported_problem: Some("Updated problem".into()),
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: None,
        },
    )
    .expect("update");
    assert_eq!(updated.customer_id, customer_id);
    assert_eq!(updated.device_id, device_id);
    assert_eq!(updated.repair_number, created.repair_number);
    assert_eq!(updated.reported_problem.as_deref(), Some("Updated problem"));
}

#[test]
fn rejects_invalid_status_on_update() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, customer_id, "SN-1");
    let created = create_repair(db.conn(), sample_repair(customer_id, device_id, company_id)).expect("create");

    let err = update_repair(
        db.conn(),
        created.id,
        RepairInput {
            customer_id,
            device_id,
            company_id,
            status: Some("invalid".into()),
            reported_problem: None,
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: None,
        },
    )
    .expect_err("invalid status");
    assert!(matches!(err, AppError::Validation { .. }));
}

#[test]
fn create_and_update_persist_expected_pickup_at() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, customer_id, "SN-1");

    let mut input = sample_repair(customer_id, device_id, company_id);
    input.expected_pickup_at = Some("2026-09-15".into());
    let created = create_repair(db.conn(), input).expect("create");
    assert_eq!(created.expected_pickup_at.as_deref(), Some("2026-09-15"));
    assert!(created.ready_at.is_none());

    let fetched = get_repair(db.conn(), created.id).expect("get");
    assert_eq!(fetched.expected_pickup_at.as_deref(), Some("2026-09-15"));

    let updated = update_repair(
        db.conn(),
        created.id,
        RepairInput {
            customer_id,
            device_id,
            company_id,
            status: None,
            reported_problem: None,
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: Some("2026-10-01".into()),
        },
    )
    .expect("update");
    assert_eq!(updated.expected_pickup_at.as_deref(), Some("2026-10-01"));
    assert!(updated.ready_at.is_none());
}

#[test]
fn omit_expected_pickup_at_stores_null() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, customer_id, "SN-1");

    let created = create_repair(db.conn(), sample_repair(customer_id, device_id, company_id)).expect("create");
    assert!(created.expected_pickup_at.is_none());

    let cleared = update_repair(
        db.conn(),
        created.id,
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
        },
    )
    .expect("clear");
    assert!(cleared.expected_pickup_at.is_none());
}

#[test]
fn rejects_invalid_expected_pickup_at_on_create() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, customer_id, "SN-1");

    let mut input = sample_repair(customer_id, device_id, company_id);
    input.expected_pickup_at = Some("09/15/2026".into());
    let err = create_repair(db.conn(), input).expect_err("invalid date");
    match err {
        AppError::Validation { field, .. } => {
            assert_eq!(field.as_deref(), Some("expectedPickupAt"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn expected_pickup_at_does_not_affect_ready_at() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, customer_id, "SN-1");

    let mut input = sample_repair(customer_id, device_id, company_id);
    input.expected_pickup_at = Some("2026-09-20".into());
    let created = create_repair(db.conn(), input).expect("create");
    assert!(created.ready_at.is_none());
    assert_eq!(created.expected_pickup_at.as_deref(), Some("2026-09-20"));

    let ready = update_repair(
        db.conn(),
        created.id,
        RepairInput {
            customer_id,
            device_id,
            company_id,
            status: Some("ready".into()),
            reported_problem: None,
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: Some("2026-09-20".into()),
        },
    )
    .expect("ready");
    assert!(ready.ready_at.is_some());
    assert_ne!(ready.ready_at.as_deref(), Some("2026-09-20"));
    assert_eq!(ready.expected_pickup_at.as_deref(), Some("2026-09-20"));
}
