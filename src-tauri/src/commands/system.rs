//! Thin system/diagnostics commands.

use tauri::State;

use crate::db::{DbHealth, DbState};
use crate::error::{AppError, CommandError};

fn lock_db<'a>(
    state: &'a State<'_, DbState>,
) -> Result<std::sync::MutexGuard<'a, crate::db::Db>, AppError> {
    state.0.lock().map_err(|_| AppError::Internal {
        message: "database lock poisoned".into(),
    })
}

#[tauri::command]
pub fn app_status(state: State<'_, DbState>) -> Result<String, CommandError> {
    let db = lock_db(&state)?;
    let health = db.health_check()?;
    Ok(format!(
        "Repair Manager backend ready (migrations={})",
        health.migrations_applied
    ))
}

#[tauri::command]
pub fn db_health(state: State<'_, DbState>) -> Result<DbHealth, CommandError> {
    let db = lock_db(&state)?;
    Ok(db.health_check()?)
}
