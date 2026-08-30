//! Presence and sync-status IPC adapters.

use tauri::State;

use crate::db::DbState;
use crate::domain::sync::{PresenceListResult, SyncStatus};
use crate::error::{AppError, CommandError};
use crate::sync_net::SyncRuntime;

fn lock_db<'a>(
    state: &'a State<'_, DbState>,
) -> Result<std::sync::MutexGuard<'a, crate::db::Db>, AppError> {
    state.0.lock().map_err(|_| AppError::Internal {
        message: "database lock poisoned".into(),
    })
}

#[tauri::command]
pub fn list_presence(
    state: State<'_, DbState>,
    runtime: State<'_, SyncRuntime>,
) -> Result<PresenceListResult, CommandError> {
    let db = lock_db(&state)?;
    Ok(runtime.list_presence(db.conn())?)
}

#[tauri::command]
pub fn get_sync_status(
    state: State<'_, DbState>,
    runtime: State<'_, SyncRuntime>,
) -> Result<SyncStatus, CommandError> {
    let db = lock_db(&state)?;
    Ok(runtime.status(db.conn())?)
}
