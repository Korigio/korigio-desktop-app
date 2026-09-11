use std::fs::File;
use std::io::Read;
use std::path::Path;

use rusqlite::Connection;
use zip::ZipArchive;

use crate::domain::backup::constants::{DATABASE_ENTRY, MANIFEST_NAME};
use crate::domain::backup::types::{BackupManifest, BackupValidationResult};
use crate::error::AppError;

use super::package::hash_reader;

pub fn validate_backup(path: &Path) -> Result<BackupValidationResult, AppError> {
    let mut errors = Vec::new();
    if !path.is_file() {
        return Ok(BackupValidationResult {
            valid: false,
            app_version: None,
            created_at: None,
            errors: vec!["Backup file was not found.".into()],
        });
    }
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file).map_err(|err| AppError::Validation {
        field: Some("path".into()),
        message: format!("Backup archive could not be opened: {err}"),
    })?;
    let manifest = match read_manifest(&mut archive) {
        Ok(manifest) => manifest,
        Err(err) => {
            return Ok(BackupValidationResult {
                valid: false,
                app_version: None,
                created_at: None,
                errors: vec![err],
            });
        }
    };
    if manifest.app_version.trim().is_empty() {
        errors.push("Manifest is missing appVersion.".into());
    }
    if manifest.created_at.trim().is_empty() {
        errors.push("Manifest is missing createdAt.".into());
    }
    let mut has_db = false;
    for entry in &manifest.files {
        has_db |= entry.path == DATABASE_ENTRY;
        match archive.by_name(&entry.path) {
            Ok(mut file) => {
                if hash_reader(&mut file).unwrap_or_default() != entry.sha256 {
                    errors.push(format!("Checksum mismatch for {}.", entry.path));
                }
            }
            Err(_) => errors.push(format!("Missing file in backup: {}.", entry.path)),
        }
    }
    if !has_db {
        errors.push("Backup does not include database.sqlite.".into());
    }
    if errors.is_empty() {
        if let Ok(mut archived_db) = archive.by_name(DATABASE_ENTRY) {
            let tmp = tempfile::Builder::new().suffix(".sqlite").tempfile()?;
            {
                let mut output = File::create(tmp.path())?;
                std::io::copy(&mut archived_db, &mut output)?;
            }
            match Connection::open(tmp.path()) {
                Ok(conn) => {
                    let check: String = conn
                        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
                        .unwrap_or_else(|err| format!("error:{err}"));
                    if check != "ok" {
                        errors.push(format!("Restored database failed integrity check: {check}"));
                    }
                }
                Err(err) => {
                    errors.push(format!("database.sqlite is not a valid SQLite file: {err}"))
                }
            }
        }
    }
    Ok(BackupValidationResult {
        valid: errors.is_empty(),
        app_version: Some(manifest.app_version),
        created_at: Some(manifest.created_at),
        errors,
    })
}

fn read_manifest(archive: &mut ZipArchive<File>) -> Result<BackupManifest, String> {
    let mut file = archive
        .by_name(MANIFEST_NAME)
        .map_err(|_| "Backup is missing manifest.json.".to_string())?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .map_err(|err| format!("Could not read manifest.json: {err}"))?;
    serde_json::from_str(&buf).map_err(|err| format!("manifest.json is invalid: {err}"))
}
