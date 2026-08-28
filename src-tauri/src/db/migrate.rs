//! Ordered SQL migrations. Never edit applied migrations; add a new file instead.

use rusqlite::Connection;
use time::OffsetDateTime;

use crate::error::AppError;

/// (version, sql). Versions must be unique and increasing.
pub const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("../../migrations/001_initial.sql")),
    (2, include_str!("../../migrations/002_search_indexes.sql")),
    (3, include_str!("../../migrations/003_diagnosis_unique_repair.sql")),
    (4, include_str!("../../migrations/004_expected_pickup_at.sql")),
    (5, include_str!("../../migrations/005_companies.sql")),
    (6, include_str!("../../migrations/006_repair_estimate_settings.sql")),
    (7, include_str!("../../migrations/007_repair_workflow_documents.sql")),
    (8, include_str!("../../migrations/008_repair_documents.sql")),
    (9, include_str!("../../migrations/009_repair_awaiting_pickup_status.sql")),
];

pub fn run(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY NOT NULL,
            applied_at TEXT NOT NULL
        );",
    )?;

    let applied = applied_versions(conn)?;

    for (version, sql) in MIGRATIONS {
        if applied.contains(version) {
            continue;
        }

        conn.pragma_update(None, "foreign_keys", false)?;
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(sql).map_err(|source| AppError::Migration {
            message: format!("migration {version} failed: {source}"),
        })?;

        let applied_at = OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .map_err(|err| AppError::Internal {
                message: format!("timestamp format failed: {err}"),
            })?;

        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            rusqlite::params![version, applied_at],
        )?;
        tx.commit()?;
        conn.pragma_update(None, "foreign_keys", true)?;
    }

    Ok(())
}

fn applied_versions(conn: &Connection) -> Result<Vec<i64>, AppError> {
    let mut stmt = conn.prepare("SELECT version FROM schema_migrations ORDER BY version")?;
    let versions = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<i64>, _>>()?;
    Ok(versions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn applies_initial_migration_once() {
        let conn = Connection::open_in_memory().expect("open");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("fk");
        run(&conn).expect("migrate");
        run(&conn).expect("migrate again");

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("count");
        assert_eq!(count, 9);

        let tables: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'customers'",
                [],
                |row| row.get(0),
            )
            .expect("tables");
        assert_eq!(tables, 1);

        let indexes: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_customers_phone'",
                [],
                |row| row.get(0),
            )
            .expect("indexes");
        assert_eq!(indexes, 1);
    }
}
