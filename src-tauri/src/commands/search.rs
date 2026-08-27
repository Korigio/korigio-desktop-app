//! Thin global search IPC adapter.

use tauri::State;

use crate::db::DbState;
use crate::domain::search::{self, GlobalSearchQuery, GlobalSearchResult};
use crate::error::{AppError, CommandError};

fn lock_db<'a>(
    state: &'a State<'_, DbState>,
) -> Result<std::sync::MutexGuard<'a, crate::db::Db>, AppError> {
    state.0.lock().map_err(|_| AppError::Internal {
        message: "database lock poisoned".into(),
    })
}

#[tauri::command]
pub fn global_search(
    state: State<'_, DbState>,
    query: GlobalSearchQuery,
) -> Result<GlobalSearchResult, CommandError> {
    let db = lock_db(&state)?;
    Ok(search::global_search(db.conn(), query)?)
}
