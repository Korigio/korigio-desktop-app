//! Thin home dashboard IPC adapter.

use tauri::State;

use crate::db::DbState;
use crate::domain::dashboard::{self, HomeDashboard};
use crate::error::{AppError, CommandError};

fn lock_db<'a>(
    state: &'a State<'_, DbState>,
) -> Result<std::sync::MutexGuard<'a, crate::db::Db>, AppError> {
    state.0.lock().map_err(|_| AppError::Internal {
        message: "database lock poisoned".into(),
    })
}

#[tauri::command]
pub fn get_home_dashboard(
    state: State<'_, DbState>,
) -> Result<HomeDashboard, CommandError> {
    let db = lock_db(&state)?;
    Ok(dashboard::get_home_dashboard(db.conn())?)
}
