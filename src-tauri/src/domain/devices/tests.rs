//! Device domain integration tests (temporary / in-memory DB only).

use crate::db::Db;
use crate::domain::customers::{create_customer, CustomerInput};
use crate::domain::devices::{
    archive_device, create_device, get_device, list_devices, unarchive_device, update_device,
    DeviceInput, DeviceListQuery,
};
use crate::error::AppError;

fn customer(db: &Db, name: &str) -> String {
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

fn sample(customer_id: &str, serial: &str) -> DeviceInput {
    DeviceInput {
        customer_id: customer_id.to_string(),
        device_type: Some("Phone".into()),
        manufacturer: Some("Acme".into()),
        model: Some("X1".into()),
        serial_number: Some(serial.into()),
        accessories: Some("Charger".into()),
        notes: None,
    }
}

#[test]
fn create_list_get_update_archive_flow() {
    let db = Db::open_in_memory().expect("db");
    let customer_id = customer(&db, "Owner");

    let created = create_device(db.conn(), sample(&customer_id, "SN-100")).expect("create");
    assert!(!created.id.clone().is_empty());
    assert_eq!(created.customer_id, customer_id);

    let listed = list_devices(
        db.conn(),
        DeviceListQuery {
            query: Some("SN-100".into()),
            customer_id: None,
            include_archived: Some(false),
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("list");
    assert_eq!(listed.total, 1);

    let by_customer = list_devices(
        db.conn(),
        DeviceListQuery {
            query: None,
            customer_id: Some(customer_id.clone()),
            include_archived: Some(false),
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("by customer");
    assert_eq!(by_customer.total, 1);

    let updated = update_device(
        db.conn(),
        created.id.clone(),
        DeviceInput {
            customer_id: customer_id.to_string(),
            device_type: Some("Tablet".into()),
            manufacturer: Some("Acme".into()),
            model: Some("X2".into()),
            serial_number: Some("SN-100".into()),
            accessories: None,
            notes: Some("screen crack".into()),
        },
    )
    .expect("update");
    assert_eq!(updated.device_type.as_deref(), Some("Tablet"));
    assert_eq!(updated.notes.as_deref(), Some("screen crack"));

    let archived = archive_device(db.conn(), created.id.clone()).expect("archive");
    assert!(archived.archived_at.is_some());

    let active = list_devices(
        db.conn(),
        DeviceListQuery {
            query: None,
            customer_id: None,
            include_archived: Some(false),
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("active");
    assert_eq!(active.total, 0);

    let restored = unarchive_device(db.conn(), created.id.clone()).expect("unarchive");
    assert!(restored.archived_at.is_none());
    assert_eq!(
        get_device(db.conn(), created.id.clone())
            .expect("get")
            .model
            .as_deref(),
        Some("X2")
    );
}

#[test]
fn rejects_device_for_archived_customer() {
    let db = Db::open_in_memory().expect("db");
    let customer_id = customer(&db, "Gone");
    crate::domain::customers::archive_customer(db.conn(), customer_id.clone())
        .expect("archive cust");

    let err = create_device(db.conn(), sample(&customer_id, "SN-9")).expect_err("blocked");
    assert_eq!(err.code(), "validation");
}

#[test]
fn rejects_empty_device_identity() {
    let db = Db::open_in_memory().expect("db");
    let customer_id = customer(&db, "Owner");
    let err = create_device(
        db.conn(),
        DeviceInput {
            customer_id: customer_id.to_string(),
            device_type: None,
            manufacturer: None,
            model: None,
            serial_number: None,
            accessories: Some("bag".into()),
            notes: None,
        },
    )
    .expect_err("empty");
    assert!(matches!(err, AppError::Validation { .. }));
}
