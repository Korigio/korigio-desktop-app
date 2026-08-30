//! Diagnosis domain integration tests (temporary / in-memory DB only).

use serde_json::json;

use crate::db::Db;
use crate::domain::companies::{create_company, CompanyInput};
use crate::domain::customers::{create_customer, CustomerInput};
use crate::domain::devices::{create_device, DeviceInput};
use crate::domain::diagnosis::constants::{KIND_CHECKBOX, KIND_TEXT};
use crate::domain::diagnosis::types::{
    DiagnosisResult, DiagnosisResultItem, DiagnosisTemplateBody, DiagnosisTemplateBodyItem,
    DiagnosisTemplateInput, DiagnosisTemplateListQuery, RepairDiagnosisInput,
};
use crate::domain::diagnosis::{
    create_diagnosis_template, delete_diagnosis_template, get_diagnosis_template,
    get_repair_diagnosis, list_diagnosis_templates, update_diagnosis_template,
    upsert_repair_diagnosis,
};
use crate::domain::repairs::{create_repair, RepairInput};

fn body_item(id: &str, label: &str, kind: &str) -> DiagnosisTemplateBodyItem {
    DiagnosisTemplateBodyItem {
        id: id.into(),
        label: label.into(),
        kind: kind.into(),
    }
}

fn sample_template(name: &str) -> DiagnosisTemplateInput {
    DiagnosisTemplateInput {
        name: name.into(),
        body: DiagnosisTemplateBody {
            items: vec![
                body_item("screen", "Screen intact", KIND_CHECKBOX),
                body_item("notes", "Technician notes", KIND_TEXT),
            ],
        },
    }
}

fn sample_result() -> DiagnosisResult {
    DiagnosisResult {
        items: vec![
            DiagnosisResultItem {
                id: "screen".into(),
                label: "Screen intact".into(),
                kind: KIND_CHECKBOX.into(),
                value: json!(false),
            },
            DiagnosisResultItem {
                id: "notes".into(),
                label: "Technician notes".into(),
                kind: KIND_TEXT.into(),
                value: json!("Hairline crack"),
            },
        ],
    }
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

fn seed_repair(db: &Db) -> String {
    let company_id = company(db);
    let customer_id = create_customer(
        db.conn(),
        CustomerInput {
            name: "Owner".into(),
            phone: None,
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
            device_type: Some("Phone".into()),
            manufacturer: Some("Acme".into()),
            model: Some("X1".into()),
            serial_number: Some("SN-1".into()),
            accessories: None,
            notes: None,
        },
    )
    .expect("device")
    .id;
    create_repair(
        db.conn(),
        RepairInput {
            customer_id: customer_id.to_string(),
            device_id: device_id.to_string(),
            company_id: company_id.to_string(),
            status: None,
            reported_problem: Some("Broken".into()),
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: None,
        },
    )
    .expect("repair")
    .id
}

#[test]
fn template_crud_flow() {
    let db = Db::open_in_memory().expect("db");
    let created =
        create_diagnosis_template(db.conn(), sample_template("Intake checklist")).expect("create");
    assert!(!created.id.clone().is_empty());
    assert_eq!(created.name, "Intake checklist");
    assert_eq!(created.body.items.len(), 2);

    let listed = list_diagnosis_templates(
        db.conn(),
        DiagnosisTemplateListQuery {
            query: Some("intake".into()),
            page: Some(1),
            page_size: Some(10),
        },
    )
    .expect("list");
    assert_eq!(listed.total, 1);
    assert_eq!(listed.items[0].name, "Intake checklist");

    let updated = update_diagnosis_template(
        db.conn(),
        created.id.clone(),
        DiagnosisTemplateInput {
            name: "Updated checklist".into(),
            body: DiagnosisTemplateBody {
                items: vec![body_item("power", "Powers on", KIND_CHECKBOX)],
            },
        },
    )
    .expect("update");
    assert_eq!(updated.name, "Updated checklist");
    assert_eq!(updated.body.items.len(), 1);

    let fetched = get_diagnosis_template(db.conn(), created.id.clone()).expect("get");
    assert_eq!(fetched.name, "Updated checklist");

    delete_diagnosis_template(db.conn(), created.id.clone()).expect("delete");
    let err = get_diagnosis_template(db.conn(), created.id.clone()).expect_err("gone");
    assert_eq!(err.code(), "not_found");
}

#[test]
fn delete_blocked_when_template_used() {
    let db = Db::open_in_memory().expect("db");
    let template = create_diagnosis_template(db.conn(), sample_template("Used")).expect("template");
    let repair_id = seed_repair(&db);

    upsert_repair_diagnosis(
        db.conn(),
        RepairDiagnosisInput {
            repair_id: repair_id.clone(),
            template_id: Some(template.id.clone()),
            result: sample_result(),
        },
    )
    .expect("upsert");

    let err = delete_diagnosis_template(db.conn(), template.id.clone()).expect_err("blocked");
    assert_eq!(err.code(), "validation");
}

#[test]
fn upsert_creates_then_updates_same_repair() {
    let db = Db::open_in_memory().expect("db");
    let template =
        create_diagnosis_template(db.conn(), sample_template("Apply")).expect("template");
    let repair_id = seed_repair(&db);

    let created = upsert_repair_diagnosis(
        db.conn(),
        RepairDiagnosisInput {
            repair_id: repair_id.clone(),
            template_id: Some(template.id.clone()),
            result: sample_result(),
        },
    )
    .expect("create");
    assert_eq!(created.repair_id, repair_id.clone());
    assert_eq!(created.template_id, Some(template.id.clone()));

    let mut updated_result = sample_result();
    updated_result.items[0].value = json!(true);
    updated_result.items[1].value = json!("Fixed");

    let updated = upsert_repair_diagnosis(
        db.conn(),
        RepairDiagnosisInput {
            repair_id: repair_id.clone(),
            template_id: Some(template.id.clone()),
            result: updated_result,
        },
    )
    .expect("update");
    assert_eq!(updated.id.clone(), created.id.clone());
    assert_eq!(updated.result.items[0].value, json!(true));
    assert_eq!(updated.result.items[1].value, json!("Fixed"));

    let fetched = get_repair_diagnosis(db.conn(), repair_id.clone())
        .expect("get")
        .expect("present");
    assert_eq!(fetched.id.clone(), created.id.clone());
}

#[test]
fn get_repair_diagnosis_null_when_none() {
    let db = Db::open_in_memory().expect("db");
    let repair_id = seed_repair(&db);
    let none = get_repair_diagnosis(db.conn(), repair_id).expect("get");
    assert!(none.is_none());
}

#[test]
fn rejects_invalid_kind_and_value() {
    let db = Db::open_in_memory().expect("db");

    let kind_err = create_diagnosis_template(
        db.conn(),
        DiagnosisTemplateInput {
            name: "Bad".into(),
            body: DiagnosisTemplateBody {
                items: vec![body_item("a", "A", "number")],
            },
        },
    )
    .expect_err("bad kind");
    assert_eq!(kind_err.code(), "validation");

    let repair_id = seed_repair(&db);
    let value_err = upsert_repair_diagnosis(
        db.conn(),
        RepairDiagnosisInput {
            repair_id: repair_id.clone(),
            template_id: None,
            result: DiagnosisResult {
                items: vec![DiagnosisResultItem {
                    id: "a".into(),
                    label: "A".into(),
                    kind: KIND_CHECKBOX.into(),
                    value: json!("nope"),
                }],
            },
        },
    )
    .expect_err("bad value");
    assert_eq!(value_err.code(), "validation");
}

#[test]
fn list_orders_by_name_asc() {
    let db = Db::open_in_memory().expect("db");
    create_diagnosis_template(db.conn(), sample_template("Zebra")).expect("z");
    create_diagnosis_template(db.conn(), sample_template("Alpha")).expect("a");

    let listed = list_diagnosis_templates(
        db.conn(),
        DiagnosisTemplateListQuery {
            query: None,
            page: Some(1),
            page_size: Some(25),
        },
    )
    .expect("list");
    assert_eq!(listed.items[0].name, "Alpha");
    assert_eq!(listed.items[1].name, "Zebra");
}
