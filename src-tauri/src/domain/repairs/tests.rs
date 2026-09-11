//! Repair domain integration tests (temporary / in-memory DB only).

use time::OffsetDateTime;

use crate::db::Db;
use crate::domain::companies::{create_company, CompanyInput};
use crate::domain::customers::{archive_customer, create_customer, CustomerInput};
use crate::domain::devices::{create_device, DeviceInput};
use crate::domain::repairs::repository;
use crate::domain::repairs::types::CompleteDiagnosisMode;
use crate::domain::repairs::{
    complete_repair_diagnosis, complete_repair_pickup, complete_repair_protocol,
    confirm_customer_approval, confirm_repair_intake, confirm_repair_parts_received,
    confirm_repair_summary, create_repair, delete_repair_document, get_repair,
    list_repair_documents, list_repairs, record_repair_summary_handover,
    resolve_repair_document_absolute, update_repair, upload_repair_document,
    CompleteRepairDiagnosisInput, RepairDocumentType, RepairInput, RepairListQuery,
};
use crate::error::AppError;

fn local_year() -> i32 {
    OffsetDateTime::now_local()
        .map(|dt| dt.year())
        .unwrap_or_else(|_| OffsetDateTime::now_utc().year())
}

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
        accessories_received: Some("Charger".into()),
        device_condition: Some("Scratched back".into()),
        diagnosis_notes: None,
        work_performed: None,
        notes: None,
        expected_pickup_at: None,
        estimate_base_cents: None,
        estimate_discount_bps: None,
    }
}

fn open_temp_db() -> (tempfile::TempDir, Db) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = Db::open_temp(dir.path().to_path_buf()).expect("db");
    (dir, db)
}

fn finalize_diagnosis(db: &Db, repair_id: &str) {
    complete_repair_diagnosis(
        db.conn(),
        CompleteRepairDiagnosisInput {
            repair_id: repair_id.to_string(),
            mode: CompleteDiagnosisMode::Finalize,
            diagnosis_notes: Some("Needs parts".into()),
            expected_pickup_at: Some("2026-09-20".into()),
            estimate_base_cents: Some(5_000),
            estimate_discount_bps: None,
        },
    )
    .expect("finalize");
}

fn write_sample_pdf(path: &std::path::Path) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("parent");
    }
    std::fs::write(path, b"%PDF-1.4 sample").expect("pdf");
}

fn advance_from_waiting_customer_to_in_repair(db: &Db, repair_id: &str) {
    confirm_customer_approval(db, repair_id.to_string()).expect("customer approval");
    let pdf = db.paths().root.join("signed.pdf");
    write_sample_pdf(&pdf);
    upload_repair_document(
        db,
        repair_id.to_string(),
        RepairDocumentType::DiagnosisSigned,
        pdf.to_string_lossy().into_owned(),
    )
    .expect("upload");
    confirm_repair_parts_received(db, repair_id.to_string()).expect("parts");
}

fn advance_to_in_repair(db: &Db, repair_id: &str) {
    confirm_repair_intake(db, repair_id.to_string()).expect("intake");
    finalize_diagnosis(db, repair_id);
    advance_from_waiting_customer_to_in_repair(db, repair_id);
}

#[test]
fn create_allocates_year_prefixed_repair_number() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");

    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");
    let year = local_year();
    assert_eq!(created.repair_number, format!("{year}-AA-000001"));
    assert_eq!(created.status, "received");
    assert_eq!(created.company_id, Some(company_id));
    assert!(created.estimate_list_cents.is_none());
    assert!(created.estimate_discount_bps.is_none());
    assert!(created.estimate_base_cents.is_none());
    assert!(created.estimate_tax_rate_bps.is_none());
    assert!(created.estimate_tax_cents.is_none());
    assert!(created.estimate_gross_cents.is_none());
    assert!(!created.received_at.is_empty());
}

#[test]
fn create_with_estimate_base_only_snapshots_tax() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");

    let mut input = sample_repair(&customer_id, &device_id, &company_id);
    input.estimate_base_cents = Some(10_000);
    let created = create_repair(db.conn(), input).expect("create");

    assert_eq!(created.estimate_list_cents, Some(10_000));
    assert_eq!(created.estimate_discount_bps, Some(0));
    assert_eq!(created.estimate_base_cents, Some(10_000));
    assert_eq!(created.estimate_tax_rate_bps, Some(1_900));
    assert_eq!(created.estimate_tax_cents, Some(1_900));
    assert_eq!(created.estimate_gross_cents, Some(11_900));
}

#[test]
fn create_with_estimate_discount_persists_net_base() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");

    let mut input = sample_repair(&customer_id, &device_id, &company_id);
    input.estimate_base_cents = Some(10_000);
    input.estimate_discount_bps = Some(1_000); // 10% → net 9000
    let created = create_repair(db.conn(), input).expect("create");

    assert_eq!(created.estimate_list_cents, Some(10_000));
    assert_eq!(created.estimate_discount_bps, Some(1_000));
    assert_eq!(created.estimate_base_cents, Some(9_000));
    assert_eq!(created.estimate_tax_rate_bps, Some(1_900));
    assert_eq!(created.estimate_tax_cents, Some(1_710));
    assert_eq!(created.estimate_gross_cents, Some(10_710));
}

#[test]
fn create_rejects_discount_without_estimate_base() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");

    let mut input = sample_repair(&customer_id, &device_id, &company_id);
    input.estimate_discount_bps = Some(500);
    let err = create_repair(db.conn(), input).expect_err("discount needs base");
    match err {
        AppError::Validation { field, .. } => {
            assert_eq!(field.as_deref(), Some("estimateDiscountBps"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn update_repair_ignores_estimate_fields() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");

    let mut input = sample_repair(&customer_id, &device_id, &company_id);
    input.estimate_base_cents = Some(10_000);
    let created = create_repair(db.conn(), input).expect("create");

    let mut update = sample_repair(&customer_id, &device_id, &company_id);
    update.reported_problem = Some("Updated problem".into());
    update.estimate_base_cents = Some(1);
    update.estimate_discount_bps = Some(5_000);
    let updated = update_repair(db.conn(), created.id.clone(), update).expect("update");

    assert_eq!(updated.reported_problem.as_deref(), Some("Updated problem"));
    assert_eq!(updated.estimate_list_cents, Some(10_000));
    assert_eq!(updated.estimate_discount_bps, Some(0));
    assert_eq!(updated.estimate_base_cents, Some(10_000));
    assert_eq!(updated.estimate_tax_cents, Some(1_900));
    assert_eq!(updated.estimate_gross_cents, Some(11_900));
}

#[test]
fn second_create_increments_sequence() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");

    let first = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("first");
    let second = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("second");

    let year = local_year();
    assert_eq!(first.repair_number, format!("{year}-AA-000001"));
    assert_eq!(second.repair_number, format!("{year}-AA-000002"));
}

#[test]
fn repair_number_sequence_is_per_year() {
    let db = Db::open_in_memory().expect("db");
    let conn = db.conn();
    let tx = conn.unchecked_transaction().expect("tx");

    let n1 = repository::allocate_repair_number(&tx, 2030, "AA").expect("n1");
    let n2 = repository::allocate_repair_number(&tx, 2030, "AA").expect("n2");
    let n3 = repository::allocate_repair_number(&tx, 2031, "AA").expect("n3");
    tx.commit().expect("commit");

    assert_eq!(n1, "2030-AA-000001");
    assert_eq!(n2, "2030-AA-000002");
    assert_eq!(n3, "2031-AA-000001");
}

#[test]
fn repair_number_sequence_is_per_device_code() {
    let db = Db::open_in_memory().expect("db");
    let conn = db.conn();
    let tx = conn.unchecked_transaction().expect("tx");

    let aa = repository::allocate_repair_number(&tx, 2026, "AA").expect("aa");
    let ab = repository::allocate_repair_number(&tx, 2026, "AB").expect("ab");
    let aa2 = repository::allocate_repair_number(&tx, 2026, "AA").expect("aa2");
    tx.commit().expect("commit");

    assert_eq!(aa, "2026-AA-000001");
    assert_eq!(ab, "2026-AB-000001");
    assert_eq!(aa2, "2026-AA-000002");
}

#[test]
fn rejects_wrong_device_ownership() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let owner_a = customer(&db, "A");
    let owner_b = customer(&db, "B");
    let device_b = device(&db, &owner_b, "SN-B");

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
            estimate_base_cents: None,
            estimate_discount_bps: None,
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
    let device_id = device(&db, &customer_id, "SN-1");
    archive_customer(db.conn(), customer_id.clone()).expect("archive");

    let err = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect_err("blocked");
    assert_eq!(err.code(), "validation");
}

#[test]
fn list_filters_by_customer_device_status_and_query() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");

    let repair = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    let by_customer = list_repairs(
        db.conn(),
        RepairListQuery {
            query: None,
            customer_id: Some(customer_id),
            device_id: None,
            company_id: None,
            status: None,
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("by customer");
    assert_eq!(by_customer.total, 1);
    assert_eq!(by_customer.items[0].device_name, "Acme · X1 · SN-1");

    let by_device = list_repairs(
        db.conn(),
        RepairListQuery {
            query: None,
            customer_id: None,
            device_id: Some(device_id),
            company_id: None,
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
            company_id: None,
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
            company_id: None,
            status: None,
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("by query");
    assert_eq!(by_query.total, 1);
    assert_eq!(by_query.items[0].id, repair.id.clone());
    assert_eq!(by_query.items[0].customer_name, "Owner");
    assert_eq!(by_query.items[0].device_name, "Acme · X1 · SN-1");

    let empty = list_repairs(
        db.conn(),
        RepairListQuery {
            query: None,
            customer_id: None,
            device_id: None,
            company_id: None,
            status: Some("ready".into()),
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("empty");
    assert_eq!(empty.total, 0);
}

#[test]
fn list_filters_by_company_id() {
    let db = Db::open_in_memory().expect("db");
    let company_a = company(&db);
    let company_b = company(&db);
    let customer_a = customer(&db, "Owner A");
    let customer_b = customer(&db, "Owner B");
    let device_a = device(&db, &customer_a, "SN-A");
    let device_b = device(&db, &customer_b, "SN-B");

    let repair_a = create_repair(db.conn(), sample_repair(&customer_a, &device_a, &company_a))
        .expect("create a");
    let repair_b = create_repair(db.conn(), sample_repair(&customer_b, &device_b, &company_b))
        .expect("create b");

    let by_company_a = list_repairs(
        db.conn(),
        RepairListQuery {
            company_id: Some(company_a.clone()),
            page: Some(1),
            page_size: Some(10),
            ..Default::default()
        },
    )
    .expect("by company a");
    assert_eq!(by_company_a.total, 1);
    assert_eq!(by_company_a.items[0].id, repair_a.id);
    assert!(!by_company_a.items.iter().any(|item| item.id == repair_b.id));

    let by_company_b = list_repairs(
        db.conn(),
        RepairListQuery {
            company_id: Some(company_b.clone()),
            page: Some(1),
            page_size: Some(10),
            ..Default::default()
        },
    )
    .expect("by company b");
    assert_eq!(by_company_b.total, 1);
    assert_eq!(by_company_b.items[0].id, repair_b.id);

    let by_company_and_status = list_repairs(
        db.conn(),
        RepairListQuery {
            company_id: Some(company_a.clone()),
            status: Some("received".into()),
            page: Some(1),
            page_size: Some(10),
            ..Default::default()
        },
    )
    .expect("company + received");
    assert_eq!(by_company_and_status.total, 1);
    assert_eq!(by_company_and_status.items[0].id, repair_a.id);

    let company_and_other_status = list_repairs(
        db.conn(),
        RepairListQuery {
            company_id: Some(company_a),
            status: Some("ready".into()),
            page: Some(1),
            page_size: Some(10),
            ..Default::default()
        },
    )
    .expect("company + ready");
    assert_eq!(company_and_other_status.total, 0);

    let err = list_repairs(
        db.conn(),
        RepairListQuery {
            company_id: Some("not-a-uuid".into()),
            page: Some(1),
            page_size: Some(10),
            ..Default::default()
        },
    )
    .expect_err("invalid companyId");
    match err {
        AppError::Validation { field, .. } => {
            assert_eq!(field.as_deref(), Some("companyId"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
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
            customer_id: customer_id.to_string(),
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
            customer_id: customer_id.to_string(),
            device_id: device_id.to_string(),
            company_id: company_id.to_string(),
            status: None,
            reported_problem: Some("Won't boot after update".into()),
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: None,
            estimate_base_cents: None,
            estimate_discount_bps: None,
        },
    )
    .expect("create");

    let by_phone = list_repairs(
        db.conn(),
        RepairListQuery {
            query: Some("600 111".into()),
            customer_id: None,
            device_id: None,
            company_id: None,
            status: None,
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("by phone");
    assert_eq!(by_phone.total, 1);
    assert_eq!(by_phone.items[0].id, repair.id.clone());

    let by_serial = list_repairs(
        db.conn(),
        RepairListQuery {
            query: Some("SERIAL-FIND".into()),
            customer_id: None,
            device_id: None,
            company_id: None,
            status: None,
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("by serial");
    assert_eq!(by_serial.total, 1);
    assert_eq!(by_serial.items[0].id, repair.id.clone());

    let by_problem = list_repairs(
        db.conn(),
        RepairListQuery {
            query: Some("won't boot".into()),
            customer_id: None,
            device_id: None,
            company_id: None,
            status: None,
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("by problem");
    assert_eq!(by_problem.total, 1);
    assert_eq!(by_problem.items[0].id, repair.id.clone());

    let by_manufacturer = list_repairs(
        db.conn(),
        RepairListQuery {
            query: Some("Lenovo".into()),
            customer_id: None,
            device_id: None,
            company_id: None,
            status: None,
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("by manufacturer");
    assert_eq!(by_manufacturer.total, 1);
}

#[test]
fn complete_protocol_sets_ready_at_once() {
    let (_dir, db) = open_temp_db();
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");
    assert!(created.ready_at.is_none());

    advance_to_in_repair(&db, &created.id.clone());

    let ready =
        complete_repair_protocol(&db, created.id.clone(), "Replaced screen".into()).expect("ready");
    assert_eq!(ready.status, "ready");
    let ready_at = ready.ready_at.clone().expect("ready_at set");
    assert_eq!(ready.work_performed.as_deref(), Some("Replaced screen"));

    let err = complete_repair_protocol(&db, created.id.clone(), "Again".into())
        .expect_err("wrong status");
    assert!(matches!(err, AppError::Validation { .. }));

    let fetched = get_repair(db.conn(), created.id.clone()).expect("get");
    assert_eq!(fetched.ready_at.as_deref(), Some(ready_at.as_str()));
}

#[test]
fn update_keeps_customer_and_device_ownership() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let other_customer = customer(&db, "Other");
    let device_id = device(&db, &customer_id, "SN-1");
    let other_device = device(&db, &other_customer, "SN-2");

    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    let updated = update_repair(
        db.conn(),
        created.id.clone(),
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
            estimate_base_cents: None,
            estimate_discount_bps: None,
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
    let device_id = device(&db, &customer_id, "SN-1");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    let err = update_repair(
        db.conn(),
        created.id.clone(),
        RepairInput {
            customer_id: customer_id.to_string(),
            device_id: device_id.to_string(),
            company_id: company_id.to_string(),
            status: Some("invalid".into()),
            reported_problem: None,
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: None,
            estimate_base_cents: None,
            estimate_discount_bps: None,
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
    let device_id = device(&db, &customer_id, "SN-1");

    let mut input = sample_repair(&customer_id, &device_id, &company_id);
    input.expected_pickup_at = Some("2026-09-15".into());
    let created = create_repair(db.conn(), input).expect("create");
    assert_eq!(created.expected_pickup_at.as_deref(), Some("2026-09-15"));
    assert!(created.ready_at.is_none());

    let fetched = get_repair(db.conn(), created.id.clone()).expect("get");
    assert_eq!(fetched.expected_pickup_at.as_deref(), Some("2026-09-15"));

    let updated = update_repair(
        db.conn(),
        created.id.clone(),
        RepairInput {
            customer_id: customer_id.to_string(),
            device_id: device_id.to_string(),
            company_id: company_id.to_string(),
            status: None,
            reported_problem: None,
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: Some("2026-10-01".into()),
            estimate_base_cents: None,
            estimate_discount_bps: None,
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
    let device_id = device(&db, &customer_id, "SN-1");

    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");
    assert!(created.expected_pickup_at.is_none());

    let cleared = update_repair(
        db.conn(),
        created.id.clone(),
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
    let device_id = device(&db, &customer_id, "SN-1");

    let mut input = sample_repair(&customer_id, &device_id, &company_id);
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
    let (_dir, db) = open_temp_db();
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");

    let mut input = sample_repair(&customer_id, &device_id, &company_id);
    input.expected_pickup_at = Some("2026-09-20".into());
    let created = create_repair(db.conn(), input).expect("create");
    assert!(created.ready_at.is_none());
    assert_eq!(created.expected_pickup_at.as_deref(), Some("2026-09-20"));

    advance_to_in_repair(&db, &created.id.clone());

    let ready = complete_repair_protocol(&db, created.id.clone(), "Fixed".into()).expect("ready");
    assert!(ready.ready_at.is_some());
    assert_ne!(ready.ready_at.as_deref(), Some("2026-09-20"));
    assert_eq!(ready.expected_pickup_at.as_deref(), Some("2026-09-20"));
}

#[test]
fn draft_sets_diagnosis_status_and_may_omit_notes() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    let result = complete_repair_diagnosis(
        db.conn(),
        CompleteRepairDiagnosisInput {
            repair_id: created.id.clone(),
            mode: CompleteDiagnosisMode::Draft,
            diagnosis_notes: None,
            expected_pickup_at: Some("2026-09-20".into()),
            estimate_base_cents: Some(10_000),
            estimate_discount_bps: None,
        },
    )
    .expect("draft");

    assert_eq!(result.repair.status, "diagnosis");
    assert!(result.repair.diagnosis_notes.is_none());
    assert_eq!(
        result.repair.expected_pickup_at.as_deref(),
        Some("2026-09-20")
    );
    assert_eq!(result.repair.estimate_list_cents, Some(10_000));
    assert_eq!(result.repair.estimate_discount_bps, Some(0));
    assert_eq!(result.repair.estimate_base_cents, Some(10_000));
    assert_eq!(result.repair.estimate_tax_rate_bps, Some(1_900));
    assert_eq!(result.repair.estimate_tax_cents, Some(1_900));
    assert_eq!(result.repair.estimate_gross_cents, Some(11_900));
}

#[test]
fn diagnosis_estimate_updates_list_and_discount() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");

    let mut input = sample_repair(&customer_id, &device_id, &company_id);
    input.estimate_base_cents = Some(10_000);
    input.estimate_discount_bps = Some(1_000);
    let created = create_repair(db.conn(), input).expect("create");
    assert_eq!(created.estimate_list_cents, Some(10_000));
    assert_eq!(created.estimate_discount_bps, Some(1_000));
    assert_eq!(created.estimate_base_cents, Some(9_000));

    let result = complete_repair_diagnosis(
        db.conn(),
        CompleteRepairDiagnosisInput {
            repair_id: created.id.clone(),
            mode: CompleteDiagnosisMode::Draft,
            diagnosis_notes: None,
            expected_pickup_at: None,
            estimate_base_cents: Some(12_000),
            estimate_discount_bps: None,
        },
    )
    .expect("diagnose");

    // List persisted; omitted discount defaults to 0 (same as create_repair).
    assert_eq!(result.repair.estimate_list_cents, Some(12_000));
    assert_eq!(result.repair.estimate_discount_bps, Some(0));
    assert_eq!(result.repair.estimate_base_cents, Some(12_000));
    assert_eq!(result.repair.estimate_tax_cents, Some(2_280));
    assert_eq!(result.repair.estimate_gross_cents, Some(14_280));
}

#[test]
fn diagnosis_estimate_applies_discount_then_tax() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    let result = complete_repair_diagnosis(
        db.conn(),
        CompleteRepairDiagnosisInput {
            repair_id: created.id.clone(),
            mode: CompleteDiagnosisMode::Draft,
            diagnosis_notes: None,
            expected_pickup_at: None,
            estimate_base_cents: Some(10_000),
            estimate_discount_bps: Some(1_000), // 10% → net 9000
        },
    )
    .expect("diagnose");

    assert_eq!(result.repair.estimate_list_cents, Some(10_000));
    assert_eq!(result.repair.estimate_discount_bps, Some(1_000));
    assert_eq!(result.repair.estimate_base_cents, Some(9_000));
    assert_eq!(result.repair.estimate_tax_rate_bps, Some(1_900));
    assert_eq!(result.repair.estimate_tax_cents, Some(1_710));
    assert_eq!(result.repair.estimate_gross_cents, Some(10_710));
}

#[test]
fn diagnosis_rejects_clearing_estimate_when_intake_set() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");

    let mut input = sample_repair(&customer_id, &device_id, &company_id);
    input.estimate_base_cents = Some(10_000);
    input.estimate_discount_bps = Some(500);
    let created = create_repair(db.conn(), input).expect("create");
    assert_eq!(created.estimate_base_cents, Some(9_500));

    let err = complete_repair_diagnosis(
        db.conn(),
        CompleteRepairDiagnosisInput {
            repair_id: created.id.clone(),
            mode: CompleteDiagnosisMode::Draft,
            diagnosis_notes: Some("WIP".into()),
            expected_pickup_at: None,
            estimate_base_cents: None,
            estimate_discount_bps: None,
        },
    )
    .expect_err("cannot clear");
    match err {
        AppError::Validation { field, .. } => {
            assert_eq!(field.as_deref(), Some("estimateBaseCents"));
        }
        other => panic!("unexpected: {other:?}"),
    }

    // List/discount still intact after rejected clear.
    let still = get_repair(db.conn(), created.id.clone()).expect("get");
    assert_eq!(still.estimate_list_cents, Some(10_000));
    assert_eq!(still.estimate_discount_bps, Some(500));
    assert_eq!(still.estimate_base_cents, Some(9_500));
}

#[test]
fn finalize_requires_notes_and_sets_waiting_customer() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    let err = complete_repair_diagnosis(
        db.conn(),
        CompleteRepairDiagnosisInput {
            repair_id: created.id.clone(),
            mode: CompleteDiagnosisMode::Finalize,
            diagnosis_notes: Some("  ".into()),
            expected_pickup_at: None,
            estimate_base_cents: Some(5_000),
            estimate_discount_bps: None,
        },
    )
    .expect_err("notes required");
    match err {
        AppError::Validation { field, .. } => {
            assert_eq!(field.as_deref(), Some("diagnosisNotes"));
        }
        other => panic!("unexpected: {other:?}"),
    }

    let result = complete_repair_diagnosis(
        db.conn(),
        CompleteRepairDiagnosisInput {
            repair_id: created.id.clone(),
            mode: CompleteDiagnosisMode::Finalize,
            diagnosis_notes: Some("Board short".into()),
            expected_pickup_at: None,
            estimate_base_cents: Some(5_000),
            estimate_discount_bps: None,
        },
    )
    .expect("finalize");
    assert_eq!(result.repair.status, "waiting_customer");
    assert_eq!(
        result.repair.diagnosis_notes.as_deref(),
        Some("Board short")
    );
    assert_eq!(result.repair.estimate_base_cents, Some(5_000));
    assert_eq!(result.repair.estimate_tax_cents, Some(950));
    assert_eq!(result.repair.estimate_gross_cents, Some(5_950));
}

#[test]
fn rejects_clearing_estimate_once_set() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    complete_repair_diagnosis(
        db.conn(),
        CompleteRepairDiagnosisInput {
            repair_id: created.id.clone(),
            mode: CompleteDiagnosisMode::Draft,
            diagnosis_notes: Some("WIP".into()),
            expected_pickup_at: None,
            estimate_base_cents: Some(1_000),
            estimate_discount_bps: None,
        },
    )
    .expect("set estimate");

    let err = complete_repair_diagnosis(
        db.conn(),
        CompleteRepairDiagnosisInput {
            repair_id: created.id.clone(),
            mode: CompleteDiagnosisMode::Draft,
            diagnosis_notes: Some("WIP".into()),
            expected_pickup_at: None,
            estimate_base_cents: None,
            estimate_discount_bps: None,
        },
    )
    .expect_err("no clear");
    match err {
        AppError::Validation { field, .. } => {
            assert_eq!(field.as_deref(), Some("estimateBaseCents"));
        }
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn draft_does_not_downgrade_past_diagnosis() {
    let (_dir, db) = open_temp_db();
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    complete_repair_diagnosis(
        db.conn(),
        CompleteRepairDiagnosisInput {
            repair_id: created.id.clone(),
            mode: CompleteDiagnosisMode::Finalize,
            diagnosis_notes: Some("Done".into()),
            expected_pickup_at: None,
            estimate_base_cents: Some(2_000),
            estimate_discount_bps: None,
        },
    )
    .expect("finalize");

    advance_from_waiting_customer_to_in_repair(&db, &created.id.clone());

    let drafted = complete_repair_diagnosis(
        db.conn(),
        CompleteRepairDiagnosisInput {
            repair_id: created.id.clone(),
            mode: CompleteDiagnosisMode::Draft,
            diagnosis_notes: Some("Still Done".into()),
            expected_pickup_at: None,
            estimate_base_cents: Some(2_500),
            estimate_discount_bps: None,
        },
    )
    .expect("draft after advance");
    assert_eq!(drafted.repair.status, "in_repair");
    assert_eq!(drafted.repair.estimate_base_cents, Some(2_500));
}

#[test]
fn rejects_diagnosis_on_cancelled_repair() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    update_repair(
        db.conn(),
        created.id.clone(),
        RepairInput {
            customer_id: customer_id.to_string(),
            device_id: device_id.to_string(),
            company_id: company_id.to_string(),
            status: Some("cancelled".into()),
            reported_problem: None,
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: None,
            estimate_base_cents: None,
            estimate_discount_bps: None,
        },
    )
    .expect("cancel");

    let err = complete_repair_diagnosis(
        db.conn(),
        CompleteRepairDiagnosisInput {
            repair_id: created.id.clone(),
            mode: CompleteDiagnosisMode::Draft,
            diagnosis_notes: Some("Nope".into()),
            expected_pickup_at: None,
            estimate_base_cents: Some(100),
            estimate_discount_bps: None,
        },
    )
    .expect_err("cancelled");
    assert!(matches!(err, AppError::Validation { .. }));
}

#[test]
fn workflow_advances_through_all_statuses() {
    let (_dir, db) = open_temp_db();
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-WF");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    advance_to_in_repair(&db, &created.id.clone());
    let in_repair = get_repair(db.conn(), created.id.clone()).expect("get");
    assert_eq!(in_repair.status, "in_repair");

    use crate::domain::repairs::list_repair_documents;

    let docs = list_repair_documents(db.conn(), created.id.clone()).expect("list docs");
    assert!(docs.iter().any(|d| d.document_type == "diagnosisSigned"));

    let ready = complete_repair_protocol(&db, created.id.clone(), "Replaced board".into())
        .expect("protocol");
    assert_eq!(ready.status, "ready");
    assert_eq!(ready.work_performed.as_deref(), Some("Replaced board"));

    let awaiting = confirm_repair_summary(&db, ready.id.clone()).expect("summary");
    assert_eq!(awaiting.status, "awaiting_pickup");

    let collected =
        complete_repair_pickup(&db, created.id.clone(), "2026-08-28".into()).expect("pickup");
    assert_eq!(collected.status, "collected");
    assert_eq!(
        collected.collected_at.as_deref(),
        Some("2026-08-28T00:00:00Z")
    );
}

#[test]
fn blocks_manual_status_change_except_cancel() {
    let db = Db::open_in_memory().expect("db");
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    let err = update_repair(
        db.conn(),
        created.id.clone(),
        RepairInput {
            customer_id: customer_id.to_string(),
            device_id: device_id.to_string(),
            company_id: company_id.to_string(),
            status: Some("ready".into()),
            reported_problem: None,
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: None,
            estimate_base_cents: None,
            estimate_discount_bps: None,
        },
    )
    .expect_err("manual status");
    match err {
        AppError::Validation { field, message } => {
            assert_eq!(field.as_deref(), Some("status"));
            assert!(message.contains("workflow"));
        }
        other => panic!("unexpected: {other:?}"),
    }

    let cancelled = update_repair(
        db.conn(),
        created.id.clone(),
        RepairInput {
            customer_id: customer_id.to_string(),
            device_id: device_id.to_string(),
            company_id: company_id.to_string(),
            status: Some("cancelled".into()),
            reported_problem: None,
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: None,
            estimate_base_cents: None,
            estimate_discount_bps: None,
        },
    )
    .expect("cancel allowed");
    assert_eq!(cancelled.status, "cancelled");
}

#[test]
fn upload_repair_document_rejects_wrong_workflow_for_customer_approval_only() {
    let (_dir, db) = open_temp_db();
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    let pdf = db.paths().root.join("signed.pdf");
    write_sample_pdf(&pdf);

    let doc = upload_repair_document(
        &db,
        created.id.clone(),
        RepairDocumentType::EntranceSigned,
        pdf.to_string_lossy().into_owned(),
    )
    .expect("upload entrance anytime");
    assert_eq!(doc.document_type, "entranceSigned");
    assert_eq!(created.status, "received");
}

#[test]
fn confirm_repair_intake_advances_to_diagnosis() {
    let (_dir, db) = open_temp_db();
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-INT");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    let updated = confirm_repair_intake(&db, created.id.clone()).expect("intake");
    assert_eq!(updated.status, "diagnosis");

    let err = confirm_repair_intake(&db, created.id.clone()).expect_err("already past received");
    assert!(matches!(err, AppError::Validation { .. }));
}

#[test]
fn confirm_repair_summary_advances_to_awaiting_pickup() {
    let (_dir, db) = open_temp_db();
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-SUM");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    advance_to_in_repair(&db, &created.id.clone());
    let ready = complete_repair_protocol(&db, created.id.clone(), "Done".into()).expect("protocol");
    assert_eq!(ready.status, "ready");

    let awaiting = confirm_repair_summary(&db, ready.id.clone()).expect("summary");
    assert_eq!(awaiting.status, "awaiting_pickup");

    let err = confirm_repair_summary(&db, created.id.clone()).expect_err("wrong status");
    assert!(matches!(err, AppError::Validation { .. }));
}

#[test]
fn record_repair_summary_handover_sets_collected_at_while_ready() {
    let (_dir, db) = open_temp_db();
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-HO");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    advance_to_in_repair(&db, &created.id.clone());
    let ready = complete_repair_protocol(&db, created.id.clone(), "Done".into()).expect("protocol");
    assert_eq!(ready.status, "ready");
    assert!(ready.collected_at.is_none());
    assert!(ready.warranty_years.is_none());

    let handed = record_repair_summary_handover(&db, ready.id.clone(), "2026-08-28".into(), 1)
        .expect("handover");
    assert_eq!(handed.status, "ready");
    assert_eq!(handed.collected_at.as_deref(), Some("2026-08-28T00:00:00Z"));
    assert_eq!(handed.warranty_years, Some(1));

    let updated = record_repair_summary_handover(&db, ready.id.clone(), "2026-08-29".into(), 2)
        .expect("re-handover");
    assert_eq!(updated.status, "ready");
    assert_eq!(
        updated.collected_at.as_deref(),
        Some("2026-08-29T00:00:00Z")
    );
    assert_eq!(updated.warranty_years, Some(2));
}

#[test]
fn record_repair_summary_handover_rejects_wrong_status() {
    let (_dir, db) = open_temp_db();
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-HO2");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    let err = record_repair_summary_handover(&db, created.id.clone(), "2026-08-28".into(), 1)
        .expect_err("wrong status");
    match err {
        AppError::Validation { field, message } => {
            assert_eq!(field.as_deref(), Some("status"));
            assert!(message.contains("ready"));
        }
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn record_repair_summary_handover_rejects_invalid_date() {
    let (_dir, db) = open_temp_db();
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-HO3");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    advance_to_in_repair(&db, &created.id.clone());
    let ready = complete_repair_protocol(&db, created.id.clone(), "Done".into()).expect("protocol");

    let err = record_repair_summary_handover(&db, ready.id.clone(), "28-08-2026".into(), 1)
        .expect_err("invalid date");
    match err {
        AppError::Validation { field, message } => {
            assert_eq!(field.as_deref(), Some("collectedAt"));
            assert!(message.contains("YYYY-MM-DD"));
        }
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn record_repair_summary_handover_rejects_warranty_out_of_range() {
    let (_dir, db) = open_temp_db();
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-HO4");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    advance_to_in_repair(&db, &created.id.clone());
    let ready = complete_repair_protocol(&db, created.id.clone(), "Done".into()).expect("protocol");

    for bad in [-1_i64, 11] {
        let err = record_repair_summary_handover(&db, ready.id.clone(), "2026-08-28".into(), bad)
            .expect_err("out of range");
        match err {
            AppError::Validation { field, message } => {
                assert_eq!(field.as_deref(), Some("warrantyYears"));
                assert!(message.contains("0") && message.contains("10"));
            }
            other => panic!("unexpected for {bad}: {other:?}"),
        }
    }

    let ok = record_repair_summary_handover(&db, ready.id.clone(), "2026-08-28".into(), 0)
        .expect("zero warranty ok");
    assert_eq!(ok.warranty_years, Some(0));
    let max = record_repair_summary_handover(&db, ready.id.clone(), "2026-08-28".into(), 10)
        .expect("max warranty ok");
    assert_eq!(max.warranty_years, Some(10));
}

#[test]
fn confirm_customer_approval_requires_waiting_customer() {
    let (_dir, db) = open_temp_db();
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    let err = confirm_customer_approval(&db, created.id.clone()).expect_err("wrong status");
    assert!(matches!(err, AppError::Validation { .. }));
}

#[test]
fn complete_protocol_requires_work_performed() {
    let (_dir, db) = open_temp_db();
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-1");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");
    advance_to_in_repair(&db, &created.id.clone());

    let err =
        complete_repair_protocol(&db, created.id.clone(), "  ".into()).expect_err("empty work");
    match err {
        AppError::Validation { field, .. } => {
            assert_eq!(field.as_deref(), Some("workPerformed"));
        }
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn upload_repair_document_writes_blob_and_display_file() {
    let (_dir, db) = open_temp_db();
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-DOC");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    let pdf = db.paths().root.join("entrance.pdf");
    write_sample_pdf(&pdf);
    upload_repair_document(
        &db,
        created.id.clone(),
        RepairDocumentType::EntranceSigned,
        pdf.to_string_lossy().into_owned(),
    )
    .expect("upload");

    let (file_path, content_hash): (String, String) = db
        .conn()
        .query_row(
            "SELECT file_path, content_hash FROM repair_documents
             WHERE repair_id = ?1 AND document_type = 'entrance_signed' AND deleted_at IS NULL",
            [&created.id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("document row");

    assert!(!content_hash.is_empty(), "upload must store content_hash");
    let display = db.paths().root.join(&file_path);
    assert!(display.is_file(), "upload must materialize display file");

    let blob = crate::domain::sync::blobs::blob_absolute(&db.paths().root, &content_hash);
    assert!(blob.is_file(), "upload must write content-addressed blob");

    let blob_kind: String = db
        .conn()
        .query_row(
            "SELECT kind FROM content_blobs WHERE content_hash = ?1",
            [&content_hash],
            |row| row.get(0),
        )
        .expect("content_blobs row");
    assert_eq!(blob_kind, "document");
}

#[test]
fn resolve_repair_document_rematerializes_from_blob_when_display_missing() {
    let (_dir, db) = open_temp_db();
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-BLOB");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    let pdf = db.paths().root.join("entrance.pdf");
    write_sample_pdf(&pdf);
    upload_repair_document(
        &db,
        created.id.clone(),
        RepairDocumentType::EntranceSigned,
        pdf.to_string_lossy().into_owned(),
    )
    .expect("upload");

    let (file_path, content_hash): (String, String) = db
        .conn()
        .query_row(
            "SELECT file_path, content_hash FROM repair_documents
             WHERE repair_id = ?1 AND document_type = 'entrance_signed' AND deleted_at IS NULL",
            [&created.id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("document row");

    let display = db.paths().root.join(&file_path);
    std::fs::remove_file(&display).expect("delete display");
    assert!(!display.exists());
    assert!(
        crate::domain::sync::blobs::blob_absolute(&db.paths().root, &content_hash).is_file(),
        "blob must remain"
    );

    let resolved = resolve_repair_document_absolute(
        &db,
        created.id.clone(),
        RepairDocumentType::EntranceSigned,
    )
    .expect("resolve via blob");
    assert!(resolved.is_file());
    assert!(display.is_file(), "display path must be rematerialized");
}

#[test]
fn upload_repair_document_sync_payload_includes_file_path_and_content_hash() {
    let (_dir, db) = open_temp_db();
    let company_id = company(&db);
    let customer_id = customer(&db, "Owner");
    let device_id = device(&db, &customer_id, "SN-SYNC");
    let created = create_repair(
        db.conn(),
        sample_repair(&customer_id, &device_id, &company_id),
    )
    .expect("create");

    let pdf = db.paths().root.join("entrance.pdf");
    write_sample_pdf(&pdf);
    upload_repair_document(
        &db,
        created.id.clone(),
        RepairDocumentType::EntranceSigned,
        pdf.to_string_lossy().into_owned(),
    )
    .expect("upload");

    let (doc_id, file_path, content_hash): (String, String, String) = db
        .conn()
        .query_row(
            "SELECT id, file_path, content_hash FROM repair_documents
             WHERE repair_id = ?1 AND document_type = 'entrance_signed' AND deleted_at IS NULL",
            [&created.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("document row");

    let payload_json: String = db
        .conn()
        .query_row(
            "SELECT payload_json FROM sync_changes
             WHERE entity_table = 'repair_documents' AND entity_id = ?1
             ORDER BY hlc_wall_ms DESC, hlc_counter DESC LIMIT 1",
            [&doc_id],
            |row| row.get(0),
        )
        .expect("document sync change");
    let payload: serde_json::Value = serde_json::from_str(&payload_json).expect("json");

    assert_eq!(
        payload.get("id").and_then(|v| v.as_str()),
        Some(doc_id.as_str())
    );
    assert_eq!(
        payload.get("repairId").and_then(|v| v.as_str()),
        Some(created.id.as_str())
    );
    assert_eq!(
        payload.get("documentType").and_then(|v| v.as_str()),
        Some("entrance_signed")
    );
    assert_eq!(
        payload.get("filePath").and_then(|v| v.as_str()),
        Some(file_path.as_str())
    );
    assert_eq!(
        payload.get("contentHash").and_then(|v| v.as_str()),
        Some(content_hash.as_str())
    );
    assert!(payload
        .get("originalFilename")
        .and_then(|v| v.as_str())
        .is_some());
    assert!(payload.get("createdAt").and_then(|v| v.as_str()).is_some());
    assert!(payload.get("updatedAt").and_then(|v| v.as_str()).is_some());
    assert!(payload.get("updatedByStaffId").is_some());
    assert!(payload
        .get("deletedAt")
        .map(|v| v.is_null())
        .unwrap_or(false));
}
