//! Thin backup / restore IPC adapters.

use tauri::State;

use crate::db::DbState;
use crate::domain::backup::{
    self, AutoBackupResult, BackupInfo, BackupValidationResult, CreateBackupInput,
    LocalBackupListResult, RestoreBackupResult,
};
use crate::error::{AppError, CommandError};

fn lock_db<'a>(
    state: &'a State<'_, DbState>,
) -> Result<std::sync::MutexGuard<'a, crate::db::Db>, AppError> {
    state.0.lock().map_err(|_| AppError::Internal {
        message: "database lock poisoned".into(),
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn create_backup(
    state: State<'_, DbState>,
    input: CreateBackupInput,
) -> Result<BackupInfo, CommandError> {
    let db = lock_db(&state)?;
    Ok(backup::create_backup(&db, input)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn validate_backup(path: String) -> Result<BackupValidationResult, CommandError> {
    Ok(backup::validate_backup(std::path::Path::new(&path))?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn restore_backup(
    state: State<'_, DbState>,
    path: String,
) -> Result<RestoreBackupResult, CommandError> {
    let mut db = lock_db(&state)?;
    Ok(backup::restore_backup(&mut db, std::path::Path::new(&path))?)
}

#[tauri::command]
pub fn list_local_backups(
    state: State<'_, DbState>,
) -> Result<LocalBackupListResult, CommandError> {
    let db = lock_db(&state)?;
    Ok(backup::list_local_backups(db.paths())?)
}

#[tauri::command]
pub fn run_auto_backup_if_due(
    state: State<'_, DbState>,
) -> Result<AutoBackupResult, CommandError> {
    let db = lock_db(&state)?;
    Ok(backup::run_auto_backup_if_due(&db)?)
}
