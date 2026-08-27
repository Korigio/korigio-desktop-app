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

    /// Open a real on-disk database under a temporary AppData root (image/backup tests).
    #[allow(dead_code)]
    pub fn open_temp(app_data_dir: PathBuf) -> Result<Self, AppError> {
        let paths = AppPaths::from_app_data_dir(app_data_dir)?;
        Self::open(paths)
    }

    fn configure(&self) -> Result<(), AppError> {
        self.conn.pragma_update(None, "foreign_keys", true)?;
        self.conn.pragma_update(None, "journal_mode", "WAL")?;
        self.conn.pragma_update(None, "busy_timeout", 5000)?;
        Ok(())
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    #[allow(dead_code)] // tests + diagnostics
    pub fn database_path(&self) -> &Path {
        &self.paths.database
    }

    /// Flush WAL so the main database file can be replaced safely.
    pub fn checkpoint_wal(&self) -> Result<(), AppError> {
        self.conn
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        Ok(())
    }

    /// Drop the live connection so AppData files can be overwritten during restore.
    pub fn close_connection_for_restore(&mut self) -> Result<(), AppError> {
        let dummy = Connection::open_in_memory()?;
        let old = std::mem::replace(&mut self.conn, dummy);
        match old.close() {
            Ok(()) => Ok(()),
            Err((_conn, err)) => Err(AppError::from(err)),
        }
    }

    /// Re-open `database.sqlite` after a restore file swap (runs migrations).
    pub fn reopen_after_restore(&mut self) -> Result<(), AppError> {
        let conn = Connection::open(&self.paths.database)?;
        let old = std::mem::replace(&mut self.conn, conn);
        drop(old);
        self.configure()?;
        migrate::run(&self.conn)?;
        Ok(())
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
