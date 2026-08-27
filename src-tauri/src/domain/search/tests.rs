//! Global search integration tests (temporary / in-memory DB only).

use crate::db::Db;
use crate::domain::companies::{CompanyInput, create_company};
use crate::domain::customers::{CustomerInput, archive_customer, create_customer};
use crate::domain::devices::{DeviceInput, archive_device, create_device};
use crate::domain::repairs::{RepairInput, create_repair};
use crate::domain::search::{GlobalSearchQuery, global_search};

fn customer_with_phone(db: &Db, name: &str, phone: &str) -> i64 {
    create_customer(
        db.conn(),
        CustomerInput {
            name: name.into(),
            phone: Some(phone.into()),
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

#[test]
fn empty_or_whitespace_query_returns_empty_arrays() {
    let db = Db::open_in_memory().expect("db");
    let _ = customer_with_phone(&db, "Alice", "555-0100");

    for query in ["", "   ", "\t\n"] {
        let result = global_search(
            db.conn(),
            GlobalSearchQuery {
                query: query.into(),
                limit_per_type: None,
            },
        )
        .expect("search");
        assert!(result.customers.is_empty(), "query={query:?}");
        assert!(result.devices.is_empty(), "query={query:?}");
        assert!(result.repairs.is_empty(), "query={query:?}");
    }
}

#[test]
fn mixed_hits_across_customers_devices_and_repairs() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer_with_phone(&db, "Maria Lopez", "612345678");
    let device_id = device(&db, customer_id, "SN-MIXED-99");
    let repair = create_repair(
        db.conn(),
        RepairInput {
            customer_id,
            device_id,
            company_id,
            status: None,
            reported_problem: Some("Battery drain".into()),
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: None,
        },
    )
    .expect("repair");

    let by_phone = global_search(
        db.conn(),
        GlobalSearchQuery {
            query: "612345678".into(),
            limit_per_type: Some(10),
        },
    )
    .expect("by phone");
    assert_eq!(by_phone.customers.len(), 1);
    assert_eq!(by_phone.customers[0].id, customer_id);
    assert_eq!(by_phone.repairs.len(), 1);
    assert_eq!(by_phone.repairs[0].id, repair.id);

    let by_serial = global_search(
        db.conn(),
        GlobalSearchQuery {
            query: "SN-MIXED".into(),
            limit_per_type: None,
        },
    )
    .expect("by serial");
    assert_eq!(by_serial.devices.len(), 1);
    assert_eq!(by_serial.devices[0].id, device_id);
    assert_eq!(by_serial.repairs.len(), 1);

    let by_problem = global_search(
        db.conn(),
        GlobalSearchQuery {
            query: "Battery".into(),
            limit_per_type: None,
        },
    )
    .expect("by problem");
    assert_eq!(by_problem.repairs.len(), 1);
    assert_eq!(by_problem.repairs[0].id, repair.id);
}

#[test]
fn archived_entities_are_excluded() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let active_id = customer_with_phone(&db, "Active User", "111-2222");
    let archived_customer_id = customer_with_phone(&db, "Archived User", "111-9999");
    archive_customer(db.conn(), archived_customer_id).expect("archive customer");

    let active_device = device(&db, active_id, "ACTIVE-SN");
    let archived_device = device(&db, active_id, "ARCHIVED-SN");
    archive_device(db.conn(), archived_device).expect("archive device");

    let active_repair = create_repair(
        db.conn(),
        RepairInput {
            customer_id: active_id,
            device_id: active_device,
            company_id,
            status: None,
            reported_problem: Some("UniqueProblemXYZ".into()),
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: None,
        },
    )
    .expect("active repair");

    let archived_repair = create_repair(
        db.conn(),
        RepairInput {
            customer_id: active_id,
            device_id: active_device,
            company_id,
            status: None,
            reported_problem: Some("UniqueProblemXYZ archived".into()),
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: None,
        },
    )
    .expect("to-archive repair");

    db.conn()
        .execute(
            "UPDATE repairs SET archived_at = '2026-01-01T00:00:00Z' WHERE id = ?1",
            [archived_repair.id],
        )
        .expect("archive repair");

    let customers = global_search(
        db.conn(),
        GlobalSearchQuery {
            query: "111-".into(),
            limit_per_type: None,
        },
    )
    .expect("customers");
    assert_eq!(customers.customers.len(), 1);
    assert_eq!(customers.customers[0].id, active_id);

    let devices = global_search(
        db.conn(),
        GlobalSearchQuery {
            query: "SN".into(),
            limit_per_type: None,
        },
    )
    .expect("devices");
    assert!(devices.devices.iter().all(|d| d.id == active_device));
    assert_eq!(devices.devices.len(), 1);
    assert!(!devices.devices.iter().any(|d| d.id == archived_device));

    let repairs = global_search(
        db.conn(),
        GlobalSearchQuery {
            query: "UniqueProblemXYZ".into(),
            limit_per_type: None,
        },
    )
    .expect("repairs");
    assert_eq!(repairs.repairs.len(), 1);
    assert_eq!(repairs.repairs[0].id, active_repair.id);
}

#[test]
fn limit_per_type_is_clamped() {
    let db = Db::open_in_memory().expect("db");
    for i in 0..30 {
        let _ = customer_with_phone(&db, &format!("Person {i:02}"), &format!("900-{i:04}"));
    }

    let result = global_search(
        db.conn(),
        GlobalSearchQuery {
            query: "Person".into(),
            limit_per_type: Some(100),
        },
    )
    .expect("clamped");
    assert_eq!(result.customers.len(), 25);

    let defaulted = global_search(
        db.conn(),
        GlobalSearchQuery {
            query: "Person".into(),
            limit_per_type: None,
        },
    )
    .expect("default");
    assert_eq!(defaulted.customers.len(), 10);
}
