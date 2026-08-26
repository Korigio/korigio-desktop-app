//! SQLite connection setup (WAL, foreign keys).

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;

use crate::db::migrate;
use crate::error::AppError;
use crate::paths::AppPaths;

/// Shared database handle managed by Tauri.
pub struct DbState(pub Mutex<Db>);

pub struct Db {
    conn: Connection,
    paths: AppPaths,
}

impl Db {
    pub fn open(paths: AppPaths) -> Result<Self, AppError> {
        let conn = Connection::open(&paths.database)?;
        let db = Self { conn, paths };
        db.configure()?;
        migrate::run(&db.conn)?;
        Ok(db)
    }

    /// In-memory database for tests (no real AppData files required).
    #[allow(dead_code)] // used by tests and Phase 3+ repositories
    pub fn open_in_memory() -> Result<Self, AppError> {
        let paths = AppPaths {
            root: PathBuf::from(":memory:"),
            database: PathBuf::from(":memory:"),
            images: PathBuf::from(":memory:/images"),
            thumbs: PathBuf::from(":memory:/thumbs"),
            backups: PathBuf::from(":memory:/backups"),
            logs: PathBuf::from(":memory:/logs"),
        };
        let conn = Connection::open_in_memory()?;
        let db = Self { conn, paths };
        db.configure()?;
        migrate::run(&db.conn)?;
        Ok(db)
    }

    fn configure(&self) -> Result<(), AppError> {
        self.conn.pragma_update(None, "foreign_keys", true)?;
        self.conn.pragma_update(None, "journal_mode", "WAL")?;
        self.conn.pragma_update(None, "busy_timeout", 5000)?;
        Ok(())
    }

    #[allow(dead_code)] // Phase 3+ repositories
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    #[allow(dead_code)] // Phase 3+ path helpers
    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    #[allow(dead_code)] // tests + diagnostics
    pub fn database_path(&self) -> &Path {
        &self.paths.database
    }

    /// Lightweight health check used by smoke commands/tests.
    pub fn health_check(&self) -> Result<DbHealth, AppError> {
        let migrations_applied: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?;
        let customers: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM customers", [], |row| row.get(0))?;

        Ok(DbHealth {
            migrations_applied,
            customers,
            database_path: self.paths.database.display().to_string(),
        })
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DbHealth {
    pub migrations_applied: i64,
    pub customers: i64,
    /// Local path for diagnostics (not a secret).
    pub database_path: String,
}
