//! Integration tests for migrations, constraints, and connection health.
//! Uses temporary / in-memory databases only — never the real user DB.

use crate::db::{Db, MIGRATIONS};
use crate::paths::AppPaths;

#[test]
fn opens_in_memory_and_reports_health() {
    let db = Db::open_in_memory().expect("open");
    let health = db.health_check().expect("health");
    assert_eq!(health.migrations_applied, 2);
    assert_eq!(health.customers, 0);
}

#[test]
fn opens_file_database_under_temp_app_data() {
    let dir = tempfile::tempdir().expect("tempdir");
    let paths = AppPaths::from_root(dir.path()).expect("paths");
    let db = Db::open(paths).expect("open");
    assert!(db.database_path().exists());
    let health = db.health_check().expect("health");
    assert_eq!(health.migrations_applied, 2);
}

#[test]
fn foreign_keys_reject_orphan_device() {
    let db = Db::open_in_memory().expect("open");
    let err = db
        .conn()
        .execute(
            "INSERT INTO devices (
                customer_id, device_type, manufacturer, model, serial_number,
                accessories, notes, created_at, updated_at, archived_at
            ) VALUES (999, 'laptop', 'Lenovo', 'T480', 'ABC', NULL, NULL, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', NULL)",
            [],
        )
        .expect_err("fk should fail");
    let message = err.to_string();
    assert!(
        message.contains("FOREIGN KEY") || message.contains("constraint"),
        "unexpected error: {message}"
    );
}

#[test]
fn repair_status_check_constraint() {
    let db = Db::open_in_memory().expect("open");
    let conn = db.conn();
    conn.execute(
        "INSERT INTO customers (name, phone, email, address, notes, created_at, updated_at, archived_at)
         VALUES ('Test', NULL, NULL, NULL, NULL, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', NULL)",
        [],
    )
    .expect("customer");
    conn.execute(
        "INSERT INTO devices (
            customer_id, device_type, manufacturer, model, serial_number,
            accessories, notes, created_at, updated_at, archived_at
         ) VALUES (1, 'laptop', 'Lenovo', 'T480', 'ABC', NULL, NULL, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', NULL)",
        [],
    )
    .expect("device");

    let err = conn
        .execute(
            "INSERT INTO repairs (
                repair_number, customer_id, device_id, status, received_at,
                reported_problem, accessories_received, device_condition,
                diagnosis_notes, work_performed, notes, ready_at, collected_at,
                created_at, updated_at, archived_at
             ) VALUES (
                '2026-000001', 1, 1, 'not_a_status', '2026-01-01',
                NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL,
                '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', NULL
             )",
            [],
        )
        .expect_err("check should fail");
    let message = err.to_string();
    assert!(
        message.contains("CHECK") || message.contains("constraint"),
        "unexpected error: {message}"
    );
}

#[test]
fn migration_list_is_non_empty_and_ordered() {
    assert!(!MIGRATIONS.is_empty());
    let mut last = 0_i64;
    for (version, sql) in MIGRATIONS {
        assert!(*version > last, "versions must increase");
        assert!(!sql.trim().is_empty());
        last = *version;
    }
}

#[test]
fn schema_has_expected_core_tables() {
    let db = Db::open_in_memory().expect("open");
    for table in [
        "customers",
        "devices",
        "repairs",
        "repair_images",
        "diagnosis_templates",
        "repair_diagnosis",
        "settings",
        "repair_number_sequences",
        "schema_migrations",
    ] {
        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| panic!("query {table}"));
        assert_eq!(count, 1, "missing table {table}");
    }
}

#[test]
fn settings_roundtrip() {
    let db = Db::open_in_memory().expect("open");
    db.conn()
        .execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)",
            ["locale", "es"],
        )
        .expect("insert");
    let value: String = db
        .conn()
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            ["locale"],
            |row| row.get(0),
        )
        .expect("select");
    assert_eq!(value, "es");
}
