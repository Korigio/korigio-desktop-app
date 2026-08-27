//! Print domain integration tests (temporary / in-memory DB only).

use serde_json::json;

use crate::db::Db;
use crate::domain::customers::{CustomerInput, create_customer};
use crate::domain::devices::{DeviceInput, create_device};
use crate::domain::diagnosis::constants::{KIND_CHECKBOX, KIND_TEXT};
use crate::domain::diagnosis::types::{
    DiagnosisResult, DiagnosisResultItem, RepairDiagnosisInput,
};
use crate::domain::diagnosis::upsert_repair_diagnosis;
use crate::domain::print::get_repair_print_report;
use crate::domain::print::types::PrintDiagnosisValue;
use crate::domain::repairs::{RepairInput, create_repair};
use crate::error::AppError;

fn seed_repair(db: &Db) -> (i64, String) {
    let customer = create_customer(
        db.conn(),
        CustomerInput {
            name: "Print Customer".into(),
            phone: Some("+49111".into()),
            email: Some("print@example.com".into()),
            address: Some("Print St 1".into()),
            notes: None,
        },
    )
    .expect("customer");
    let device = create_device(
        db.conn(),
        DeviceInput {
            customer_id: customer.id,
            device_type: Some("Laptop".into()),
            manufacturer: Some("Lenovo".into()),
            model: Some("T14".into()),
            serial_number: Some("SN-PRINT-1".into()),
            accessories: None,
            notes: None,
        },
    )
    .expect("device");
    let repair = create_repair(
        db.conn(),
        RepairInput {
            customer_id: customer.id,
            device_id: device.id,
            status: None,
            reported_problem: Some("Won't boot".into()),
            accessories_received: Some("Charger".into()),
            device_condition: Some("Ok".into()),
            diagnosis_notes: Some("Board check".into()),
            work_performed: Some("Replaced SSD".into()),
            notes: Some("Rush".into()),
        },
    )
    .expect("repair");
    (repair.id, repair.repair_number)
}

#[test]
fn print_report_assembles_core_fields_without_diagnosis() {
    let db = Db::open_in_memory().expect("db");
    let (repair_id, repair_number) = seed_repair(&db);

    let report = get_repair_print_report(db.conn(), repair_id).expect("report");

    assert_eq!(report.repair.id, repair_id);
    assert_eq!(report.repair.repair_number, repair_number);
    assert_eq!(report.repair.status, "received");
    assert_eq!(
        report.repair.reported_problem.as_deref(),
        Some("Won't boot")
    );
    assert_eq!(report.customer.name, "Print Customer");
    assert_eq!(report.customer.phone.as_deref(), Some("+49111"));
    assert_eq!(report.customer.email.as_deref(), Some("print@example.com"));
    assert_eq!(report.customer.address.as_deref(), Some("Print St 1"));
    assert_eq!(report.device.device_type, "Laptop");
    assert_eq!(report.device.manufacturer.as_deref(), Some("Lenovo"));
    assert_eq!(report.device.model.as_deref(), Some("T14"));
    assert_eq!(report.device.serial_number.as_deref(), Some("SN-PRINT-1"));
    assert!(report.diagnosis.is_none());
}

#[test]
fn print_report_includes_diagnosis_items() {
    let db = Db::open_in_memory().expect("db");
    let (repair_id, _) = seed_repair(&db);

    upsert_repair_diagnosis(
        db.conn(),
        RepairDiagnosisInput {
            repair_id,
            template_id: None,
            result: DiagnosisResult {
                items: vec![
                    DiagnosisResultItem {
                        id: "screen".into(),
                        label: "Screen intact".into(),
                        kind: KIND_CHECKBOX.into(),
                        value: json!(true),
                    },
                    DiagnosisResultItem {
                        id: "notes".into(),
                        label: "Notes".into(),
                        kind: KIND_TEXT.into(),
                        value: json!("Hairline crack"),
                    },
                ],
            },
        },
    )
    .expect("diagnosis");

    let report = get_repair_print_report(db.conn(), repair_id).expect("report");
    let diagnosis = report.diagnosis.expect("diagnosis present");
    assert_eq!(diagnosis.items.len(), 2);
    assert_eq!(diagnosis.items[0].id, "screen");
    assert_eq!(diagnosis.items[0].value, PrintDiagnosisValue::Bool(true));
    assert_eq!(
        diagnosis.items[1].value,
        PrintDiagnosisValue::Text("Hairline crack".into())
    );
}

#[test]
fn print_report_missing_repair_is_not_found() {
    let db = Db::open_in_memory().expect("db");
    let err = get_repair_print_report(db.conn(), 999_999).expect_err("missing");
    assert!(matches!(err, AppError::NotFound));
}
