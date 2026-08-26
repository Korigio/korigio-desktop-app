//! Thin Tauri command adapters. Business logic will live under `domain/` in later phases.

use tauri::State;

use crate::db::{DbHealth, DbState};
use crate::error::{AppError, CommandError};

#[tauri::command]
pub fn app_status(state: State<'_, DbState>) -> Result<String, CommandError> {
    let db = state
        .0
        .lock()
        .map_err(|_| AppError::Internal {
            message: "database lock poisoned".into(),
        })?;
    let health = db.health_check()?;
    Ok(format!(
        "Repair Manager backend ready (migrations={})",
        health.migrations_applied
    ))
}

#[tauri::command]
pub fn db_health(state: State<'_, DbState>) -> Result<DbHealth, CommandError> {
    let db = state
        .0
        .lock()
        .map_err(|_| AppError::Internal {
            message: "database lock poisoned".into(),
        })?;
    Ok(db.health_check()?)
}
