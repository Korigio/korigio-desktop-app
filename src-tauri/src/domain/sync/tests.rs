//! HLC last-write-wins apply tests (temporary / in-memory DB only).

use crate::db::Db;
use crate::domain::companies::{archive_company, create_company, CompanyInput};
use crate::domain::diagnosis::types::{
    DiagnosisTemplateBody, DiagnosisTemplateBodyItem, DiagnosisTemplateInput,
};
use crate::domain::diagnosis::{create_diagnosis_template, update_diagnosis_template};
use crate::domain::ids::new_entity_id;
use crate::domain::sync::apply::ApplyOutcome;
use crate::domain::sync::apply_remote_change;
use crate::domain::sync::apply_snapshot_row;
use crate::domain::sync::hlc::{hlc_greater, Hlc};
use crate::domain::sync::record::SyncChange;
use crate::domain::sync::snapshot::dump_snapshot;

fn change(id: &str, entity_id: &str, name: &str, hlc: Hlc) -> SyncChange {
    let payload = serde_json::json!({
        "id": entity_id,
        "name": name,
        "createdAt": "2026-01-01T00:00:00Z",
        "updatedAt": "2026-01-01T00:00:00Z",
    });
    SyncChange {
        id: id.to_string(),
        entity_table: "customers".into(),
        entity_id: entity_id.into(),
        op: "upsert".into(),
        payload_json: payload.to_string(),
        hlc,
        created_at: "2026-01-01T00:00:00Z".into(),
    }
}

fn customer_name(db: &Db, id: &str) -> String {
    db.conn()
        .query_row("SELECT name FROM customers WHERE id = ?1", [id], |row| {
            row.get(0)
        })
        .expect("name")
}

#[test]
fn hlc_greater_matrix() {
    let a = Hlc::new(10, 1, "aaa");
    let b = Hlc::new(10, 2, "aaa");
    let c = Hlc::new(11, 0, "aaa");
    let d = Hlc::new(11, 0, "zzz");
    assert!(hlc_greater(&b, &a));
    assert!(hlc_greater(&c, &b));
    assert!(hlc_greater(&d, &c));
    assert!(!hlc_greater(&a, &b));
    assert!(!hlc_greater(&c, &d));
}

#[test]
fn newer_remote_wins_and_older_is_ignored() {
    let db = Db::open_in_memory().expect("db");
    let entity = new_entity_id();
    let first_id = new_entity_id();
    let older_id = new_entity_id();
    let newer_id = new_entity_id();

    let first = change(&first_id, &entity, "First", Hlc::new(100, 1, "device-a"));
    assert_eq!(
        apply_remote_change(db.conn(), &first).expect("first"),
        ApplyOutcome::Applied
    );
    assert_eq!(customer_name(&db, &entity), "First");

    let older = change(&older_id, &entity, "Older", Hlc::new(90, 9, "device-z"));
    assert_eq!(
        apply_remote_change(db.conn(), &older).expect("older"),
        ApplyOutcome::IgnoredOlder
    );
    assert_eq!(customer_name(&db, &entity), "First");

    let newer = change(&newer_id, &entity, "Newer", Hlc::new(100, 2, "device-a"));
    assert_eq!(
        apply_remote_change(db.conn(), &newer).expect("newer"),
        ApplyOutcome::Applied
    );
    assert_eq!(customer_name(&db, &entity), "Newer");
}

#[test]
fn same_change_id_is_idempotent() {
    let db = Db::open_in_memory().expect("db");
    let entity = new_entity_id();
    let change_id = new_entity_id();
    let first = change(&change_id, &entity, "Once", Hlc::new(50, 0, "dev"));
    assert_eq!(
        apply_remote_change(db.conn(), &first).expect("apply"),
        ApplyOutcome::Applied
    );
    assert_eq!(
        apply_remote_change(db.conn(), &first).expect("again"),
        ApplyOutcome::Idempotent
    );
    assert_eq!(customer_name(&db, &entity), "Once");
}

#[test]
fn device_id_tie_break_is_lexicographic() {
    let db = Db::open_in_memory().expect("db");
    let entity = new_entity_id();
    let low = change(&new_entity_id(), &entity, "Low", Hlc::new(10, 1, "aaa"));
    apply_remote_change(db.conn(), &low).expect("low");
    let high = change(&new_entity_id(), &entity, "High", Hlc::new(10, 1, "zzz"));
    assert_eq!(
        apply_remote_change(db.conn(), &high).expect("high"),
        ApplyOutcome::Applied
    );
    assert_eq!(customer_name(&db, &entity), "High");
}

fn count_sync_rows(db: &Db, table: &str) -> i64 {
    db.conn()
        .query_row(
            "SELECT COUNT(*) FROM sync_changes WHERE entity_table = ?1",
            [table],
            |row| row.get(0),
        )
        .expect("count")
}

#[test]
fn updating_template_and_archiving_company_inserts_sync_changes() {
    let db = Db::open_in_memory().expect("db");
    let template = create_diagnosis_template(
        db.conn(),
        DiagnosisTemplateInput {
            name: "Intake".into(),
            body: DiagnosisTemplateBody {
                items: vec![DiagnosisTemplateBodyItem {
                    id: "screen".into(),
                    label: "Screen intact".into(),
                    kind: "checkbox".into(),
                }],
            },
        },
    )
    .expect("create template");
    let before_update = count_sync_rows(&db, "diagnosis_templates");
    update_diagnosis_template(
        db.conn(),
        template.id.clone(),
        DiagnosisTemplateInput {
            name: "Intake v2".into(),
            body: template.body.clone(),
        },
    )
    .expect("update template");
    assert!(
        count_sync_rows(&db, "diagnosis_templates") > before_update,
        "template update must append sync_changes"
    );

    let company = create_company(
        db.conn(),
        CompanyInput {
            legal_name: "Acme".into(),
            trade_name: None,
            tax_id: None,
            address: None,
            phone: None,
            email: None,
            website: None,
        },
    )
    .expect("company");
    let before_archive = count_sync_rows(&db, "companies");
    archive_company(db.conn(), company.id).expect("archive");
    assert!(
        count_sync_rows(&db, "companies") > before_archive,
        "company archive must append sync_changes"
    );
}

#[test]
fn join_snapshot_applies_team_name_from_teams_row() {
    let host = Db::open_in_memory().expect("host");
    let staff = crate::domain::staff::create_staff(
        host.conn(),
        crate::domain::staff::StaffInput {
            name: "Ada".into(),
            pin: "1234".into(),
            role: None,
        },
    )
    .expect("staff");
    crate::domain::staff::sign_in_staff(host.conn(), &staff.id, "1234").expect("sign in");
    crate::domain::team::create_team(
        host.conn(),
        crate::domain::team::CreateTeamInput {
            name: "Front Desk".into(),
            member_name: "Ada".into(),
        },
    )
    .expect("team");

    let (_version, rows) = dump_snapshot(host.conn()).expect("dump");
    let teams = rows
        .iter()
        .find(|row| row.table == "teams")
        .expect("teams row");
    assert_eq!(
        teams.payload.get("name").and_then(|v| v.as_str()),
        Some("Front Desk")
    );

    let joiner = Db::open_in_memory().expect("joiner");
    apply_snapshot_row(joiner.conn(), "teams", teams.payload.clone()).expect("apply");
    let name: String = joiner
        .conn()
        .query_row("SELECT name FROM teams LIMIT 1", [], |row| row.get(0))
        .expect("name");
    assert_eq!(name, "Front Desk");
}

#[test]
fn received_blob_uses_metadata_kind_and_writes_display_path() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = Db::open_temp(dir.path().to_path_buf()).expect("db");
    let now = crate::db::repository::now_utc_rfc3339().expect("now");
    let bytes = b"logo-bytes";
    let hash = crate::domain::sync::blobs::sha256_hex(bytes);
    crate::domain::sync::blobs::upsert_blob_row(db.conn(), &hash, bytes.len() as i64, "logo", &now)
        .expect("meta");
    db.conn()
        .execute(
            "INSERT INTO companies (
                id, legal_name, trade_name, tax_id, address, phone, email, website,
                logo_path, logo_content_hash, is_default, created_at, updated_at, archived_at,
                hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
             ) VALUES (
                'c1', 'Acme', NULL, NULL, NULL, NULL, NULL, NULL,
                'images/companies/c1/logo.bin', ?1, 1, ?2, ?2, NULL,
                1, 0, 'dev', NULL, NULL
             )",
            rusqlite::params![hash, now],
        )
        .expect("company");

    let complete =
        crate::domain::sync::blobs::accept_blob_chunk(&db.paths().root, &hash, 0, bytes, true)
            .expect("chunk")
            .expect("eof");
    crate::domain::sync::blobs::finalize_received_blob(
        &db.paths().root,
        db.conn(),
        &hash,
        &complete,
    )
    .expect("finalize");

    let kind: String = db
        .conn()
        .query_row(
            "SELECT kind FROM content_blobs WHERE content_hash = ?1",
            [&hash],
            |row| row.get(0),
        )
        .expect("kind");
    assert_eq!(kind, "logo");
    let canonical = db
        .paths()
        .root
        .join(crate::domain::sync::blobs::blob_relative_path(&hash));
    assert!(canonical.exists(), "canonical blob path");
    let display = db.paths().root.join("images/companies/c1/logo.bin");
    assert!(display.exists(), "display path");
}

fn staff_change(
    change_id: &str,
    staff_id: &str,
    name: &str,
    role: &str,
    deactivated_at: Option<&str>,
    actor_id: &str,
    hlc: Hlc,
) -> SyncChange {
    let payload = serde_json::json!({
        "id": staff_id,
        "name": name,
        "role": role,
        "deactivatedAt": deactivated_at,
        "updatedByStaffId": actor_id,
        "createdAt": "2026-01-01T00:00:00Z",
        "updatedAt": "2026-01-01T00:00:00Z",
    });
    SyncChange {
        id: change_id.to_string(),
        entity_table: "staff".into(),
        entity_id: staff_id.into(),
        op: "upsert".into(),
        payload_json: payload.to_string(),
        hlc,
        created_at: "2026-01-01T00:00:00Z".into(),
    }
}

fn staff_row_hlc(db: &Db, id: &str) -> Hlc {
    db.conn()
        .query_row(
            "SELECT hlc_wall_ms, hlc_counter, origin_device_id FROM staff WHERE id = ?1",
            [id],
            |row| Ok(Hlc::new(row.get(0)?, row.get(1)?, row.get::<_, String>(2)?)),
        )
        .expect("staff hlc")
}

fn team_with_staff(db: &Db) -> (String, String) {
    let created = crate::domain::team::create_team(
        db.conn(),
        crate::domain::team::CreateTeamInput {
            name: "Shop".into(),
            member_name: "Ada".into(),
        },
    )
    .expect("team");
    let admin_id = created.session.staff.id.clone();
    let staff = crate::domain::staff::create_staff(
        db.conn(),
        crate::domain::staff::StaffInput {
            name: "Bob".into(),
            pin: "5678".into(),
            role: Some(crate::domain::staff::StaffRole::Staff),
        },
    )
    .expect("staff");
    (admin_id, staff.id)
}

#[test]
fn staff_actor_same_role_deactivated_at_applies() {
    let db = Db::open_in_memory().expect("db");
    let (admin_id, staff_id) = team_with_staff(&db);
    let local = staff_row_hlc(&db, &admin_id);
    let newer = Hlc::new(local.wall + 1, 0, "peer-device");
    let change = staff_change(
        &new_entity_id(),
        &admin_id,
        "Ada",
        "admin",
        Some("2026-06-01T00:00:00Z"),
        &staff_id,
        newer,
    );
    assert_eq!(
        apply_remote_change(db.conn(), &change).expect("apply"),
        ApplyOutcome::Applied
    );
    let deactivated: Option<String> = db
        .conn()
        .query_row(
            "SELECT deactivated_at FROM staff WHERE id = ?1",
            [&admin_id],
            |row| row.get(0),
        )
        .expect("deactivated");
    assert_eq!(deactivated.as_deref(), Some("2026-06-01T00:00:00Z"));
}

#[test]
fn staff_actor_role_change_is_rejected() {
    let db = Db::open_in_memory().expect("db");
    let (admin_id, staff_id) = team_with_staff(&db);
    let local = staff_row_hlc(&db, &admin_id);
    let newer = Hlc::new(local.wall + 1, 0, "peer-device");
    let change = staff_change(
        &new_entity_id(),
        &admin_id,
        "Ada",
        "staff",
        None,
        &staff_id,
        newer,
    );
    let err = apply_remote_change(db.conn(), &change).expect_err("rejected");
    assert!(matches!(err, crate::error::AppError::Forbidden { .. }));
    let role: String = db
        .conn()
        .query_row("SELECT role FROM staff WHERE id = ?1", [&admin_id], |row| {
            row.get(0)
        })
        .expect("role");
    assert_eq!(role, "admin");
}
