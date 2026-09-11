use std::fs;
use std::path::Path;

use rusqlite::Connection;

use crate::domain::settings::repository;
use crate::domain::settings::types::{
    AutoBackupInterval, AutoBackupSettings, SetAutoBackupSettingsInput, AUTO_BACKUP_FOLDER_KEY,
};
use crate::error::AppError;

pub fn get_auto_backup_settings(conn: &Connection) -> Result<AutoBackupSettings, AppError> {
    let interval = match repository::get_setting(conn, AutoBackupInterval::STORAGE_KEY)? {
        Some(raw) => AutoBackupInterval::parse(&raw).unwrap_or(AutoBackupInterval::Never),
        None => AutoBackupInterval::Never,
    };
    let folder_path = repository::get_setting(conn, AUTO_BACKUP_FOLDER_KEY)?
        .and_then(|raw| normalize_optional_folder_path(Some(&raw)));
    Ok(AutoBackupSettings {
        interval,
        folder_path,
    })
}

pub fn set_auto_backup_settings(
    conn: &Connection,
    input: SetAutoBackupSettingsInput,
) -> Result<AutoBackupSettings, AppError> {
    let folder = normalize_optional_folder_path(input.folder_path.as_deref());
    match input.interval {
        AutoBackupInterval::Never => {
            repository::upsert_setting(
                conn,
                AutoBackupInterval::STORAGE_KEY,
                input.interval.as_storage_value(),
            )?;
            repository::upsert_setting(
                conn,
                AUTO_BACKUP_FOLDER_KEY,
                folder.as_deref().unwrap_or(""),
            )?;
        }
        _ => {
            let Some(path) = folder else {
                return Err(AppError::Validation {
                    field: Some("folderPath".into()),
                    message: "A backup folder is required when scheduled copies are enabled."
                        .into(),
                });
            };
            let persisted = ensure_auto_backup_folder(&path)?;
            repository::upsert_setting(
                conn,
                AutoBackupInterval::STORAGE_KEY,
                input.interval.as_storage_value(),
            )?;
            repository::upsert_setting(conn, AUTO_BACKUP_FOLDER_KEY, &persisted)?;
        }
    }
    get_auto_backup_settings(conn)
}

fn normalize_optional_folder_path(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn ensure_auto_backup_folder(path: &str) -> Result<String, AppError> {
    let folder = Path::new(path);
    if !folder.is_absolute() || folder.is_file() {
        return Err(folder_path_error());
    }
    fs::create_dir_all(folder).map_err(|_| folder_path_error())?;
    if !folder.is_dir() {
        return Err(folder_path_error());
    }
    Ok(path.to_string())
}

fn folder_path_error() -> AppError {
    AppError::Validation {
        field: Some("folderPath".into()),
        message: "Backup folder must be an absolute directory path.".into(),
    }
}
