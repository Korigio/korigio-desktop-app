use std::fs::{self, File};
use std::path::{Component, Path};

use rusqlite::Connection;
use zip::ZipArchive;

use crate::db::Db;
use crate::domain::backup::constants::{DATABASE_ENTRY, MANIFEST_NAME};
use crate::domain::backup::types::RestoreBackupResult;
use crate::error::AppError;

pub fn restore_backup(db: &mut Db, path: &Path) -> Result<RestoreBackupResult, AppError> {
    let validation = super::validation::validate_backup(path)?;
    if !validation.valid {
        return Err(AppError::Validation {
            field: Some("path".into()),
            message: if validation.errors.is_empty() {
                "Backup is invalid.".into()
            } else {
                validation.errors.join(" ")
            },
        });
    }
    let safety = super::package::create_safety_backup(db)?;
    let paths = db.paths().clone();
    let staging = tempfile::tempdir()?;
    extract_backup_to(path, staging.path())?;
    let staged_db = staging.path().join(DATABASE_ENTRY);
    if !staged_db.is_file() {
        return Err(AppError::Validation {
            field: Some("path".into()),
            message: "Backup is missing database.sqlite.".into(),
        });
    }
    let max_version: i64 = {
        let conn = Connection::open(&staged_db)?;
        conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0)
    };
    if max_version < 10 {
        return Err(AppError::conflict(
            "This backup is from an older Korigio version and cannot be restored.",
        ));
    }
    db.checkpoint_wal()?;
    db.close_connection_for_restore()?;
    replace_file(&staged_db, &paths.database)?;
    replace_tree(&staging.path().join("images"), &paths.images)?;
    replace_tree(&staging.path().join("thumbs"), &paths.thumbs)?;
    let _ = fs::remove_file(format!("{}-wal", paths.database.display()));
    let _ = fs::remove_file(format!("{}-shm", paths.database.display()));
    db.reopen_after_restore()?;
    Ok(RestoreBackupResult {
        restored_from: path.to_string_lossy().into_owned(),
        safety_backup_path: safety.path,
    })
}

fn extract_backup_to(path: &Path, dest: &Path) -> Result<(), AppError> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file).map_err(|err| AppError::Validation {
        field: Some("path".into()),
        message: format!("Backup archive could not be opened: {err}"),
    })?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|err| AppError::Internal {
            message: format!("failed to read zip entry: {err}"),
        })?;
        let name = entry.name().to_string();
        if name == MANIFEST_NAME {
            continue;
        }
        if has_unsafe_zip_path(&name) {
            return Err(AppError::Validation {
                field: Some("path".into()),
                message: "Backup contains an unsafe path.".into(),
            });
        }
        let out_path = dest.join(&name);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output = File::create(out_path)?;
        std::io::copy(&mut entry, &mut output)?;
    }
    Ok(())
}

fn has_unsafe_zip_path(name: &str) -> bool {
    let path = Path::new(name);
    path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir))
}

fn replace_file(src: &Path, dest: &Path) -> Result<(), AppError> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    if dest.exists() {
        fs::remove_file(dest)?;
    }
    fs::copy(src, dest)?;
    Ok(())
}

fn replace_tree(src: &Path, dest: &Path) -> Result<(), AppError> {
    if dest.exists() {
        fs::remove_dir_all(dest)?;
    }
    fs::create_dir_all(dest)?;
    if !src.exists() {
        return Ok(());
    }
    copy_dir_recursive(src, dest)
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), AppError> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            fs::copy(from, to)?;
        }
    }
    Ok(())
}
