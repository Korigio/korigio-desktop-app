//! Print domain integration tests (temporary / in-memory DB only).

use serde_json::json;

use crate::db::Db;
use crate::domain::companies::{create_company, CompanyInput};
use crate::domain::customers::{create_customer, CustomerInput};
use crate::domain::devices::{create_device, DeviceInput};
use crate::domain::diagnosis::constants::{KIND_CHECKBOX, KIND_TEXT};
use crate::domain::diagnosis::types::{DiagnosisResult, DiagnosisResultItem, RepairDiagnosisInput};
use crate::domain::diagnosis::upsert_repair_diagnosis;
use crate::domain::print::types::PrintDiagnosisValue;
use crate::domain::print::{
    get_repair_diagnosis_print_report, get_repair_print_report, get_repair_summary_print_report,
};
use crate::domain::repairs::types::CompleteDiagnosisMode;
use crate::domain::repairs::{
    complete_repair_diagnosis, create_repair, CompleteRepairDiagnosisInput, RepairInput,
};
use crate::domain::settings::{set_shop_settings, ShopSettingsInput};
use crate::error::AppError;

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

fn seed_repair(db: &Db) -> (String, String) {
    let company_id = company(db);
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
            customer_id: customer.id.clone(),
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
            customer_id: customer.id.clone(),
            device_id: device.id.clone(),
            company_id,
            status: None,
            reported_problem: Some("Won't boot".into()),
            accessories_received: Some("Charger".into()),
            device_condition: Some("Ok".into()),
            diagnosis_notes: Some("Board check".into()),
            work_performed: Some("Replaced SSD".into()),
            notes: Some("Rush".into()),
            expected_pickup_at: None,
        },
    )
    .expect("repair");
    (repair.id.clone(), repair.repair_number)
}

#[test]
fn print_report_assembles_core_fields_without_diagnosis() {
    let db = Db::open_in_memory().expect("db");
    let (repair_id, repair_number) = seed_repair(&db);

    let report = get_repair_print_report(&db, repair_id.clone()).expect("report");

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
    let company = report.company.expect("company");
    assert_eq!(company.legal_name, "Test Company");
    assert!(report.company_logo_absolute_path.is_none());
    assert!(report.diagnosis.is_none());
}

#[test]
fn print_report_includes_diagnosis_items() {
    let db = Db::open_in_memory().expect("db");
    let (repair_id, _) = seed_repair(&db);

    upsert_repair_diagnosis(
        db.conn(),
        RepairDiagnosisInput {
            repair_id: repair_id.clone(),
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

    let report = get_repair_print_report(&db, repair_id.clone()).expect("report");
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
    let err =
        get_repair_print_report(&db, crate::domain::ids::new_entity_id()).expect_err("missing");
    assert!(matches!(err, AppError::NotFound));
}

#[test]
fn diagnosis_print_report_includes_estimate_and_currency() {
    let db = Db::open_in_memory().expect("db");
    let (repair_id, repair_number) = seed_repair(&db);

    set_shop_settings(
        db.conn(),
        ShopSettingsInput {
            tax_rate_percent: None,
            currency: Some("CHF".into()),
        },
    )
    .expect("currency");

    complete_repair_diagnosis(
        db.conn(),
        CompleteRepairDiagnosisInput {
            repair_id: repair_id.clone(),
            mode: CompleteDiagnosisMode::Finalize,
            diagnosis_notes: Some("Needs board".into()),
            expected_pickup_at: Some("2026-10-01".into()),
            estimate_base_cents: Some(10_000),
        },
    )
    .expect("diagnose");

    let report = get_repair_diagnosis_print_report(&db, repair_id.clone()).expect("report");
    assert_eq!(report.repair.repair_number, repair_number);
    assert_eq!(report.repair.status, "waiting_customer");
    assert_eq!(
        report.repair.diagnosis_notes.as_deref(),
        Some("Needs board")
    );
    assert_eq!(
        report.repair.expected_pickup_at.as_deref(),
        Some("2026-10-01")
    );
    assert_eq!(report.repair.estimate_base_cents, Some(10_000));
    assert_eq!(report.repair.estimate_tax_rate_bps, Some(1_900));
    assert_eq!(report.repair.estimate_tax_cents, Some(1_900));
    assert_eq!(report.repair.estimate_gross_cents, Some(11_900));
    assert_eq!(report.currency, "CHF");
    assert_eq!(report.customer.name, "Print Customer");
    let company = report.company.expect("company");
    assert_eq!(company.legal_name, "Test Company");
}

#[test]
fn summary_print_report_includes_work_and_dates() {
    let db = Db::open_in_memory().expect("db");
    let (repair_id, repair_number) = seed_repair(&db);

    set_shop_settings(
        db.conn(),
        ShopSettingsInput {
            tax_rate_percent: None,
            currency: Some("EUR".into()),
        },
    )
    .expect("currency");

    complete_repair_diagnosis(
        db.conn(),
        CompleteRepairDiagnosisInput {
            repair_id: repair_id.clone(),
            mode: CompleteDiagnosisMode::Finalize,
            diagnosis_notes: Some("Screen fault".into()),
            expected_pickup_at: Some("2026-10-05".into()),
            estimate_base_cents: Some(8_000),
        },
    )
    .expect("diagnose");

    let report = get_repair_summary_print_report(&db, repair_id.clone()).expect("report");
    assert_eq!(report.repair.repair_number, repair_number);
    assert_eq!(
        report.repair.reported_problem.as_deref(),
        Some("Won't boot")
    );
    assert_eq!(
        report.repair.expected_pickup_at.as_deref(),
        Some("2026-10-05")
    );
    assert_eq!(report.repair.estimate_gross_cents, Some(9_520));
    assert_eq!(report.currency, "EUR");
    assert_eq!(report.customer.name, "Print Customer");
}
