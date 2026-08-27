//! Thin repair IPC adapters.

use tauri::State;

use crate::db::DbState;
use crate::domain::repairs::{self, Repair, RepairInput, RepairListQuery, RepairListResult};
use crate::error::{AppError, CommandError};

fn lock_db<'a>(
    state: &'a State<'_, DbState>,
) -> Result<std::sync::MutexGuard<'a, crate::db::Db>, AppError> {
    state.0.lock().map_err(|_| AppError::Internal {
        message: "database lock poisoned".into(),
    })
}

#[tauri::command]
pub fn list_repairs(
    state: State<'_, DbState>,
    query: RepairListQuery,
) -> Result<RepairListResult, CommandError> {
    let db = lock_db(&state)?;
    Ok(repairs::list_repairs(db.conn(), query)?)
}

#[tauri::command]
pub fn get_repair(state: State<'_, DbState>, id: i64) -> Result<Repair, CommandError> {
    let db = lock_db(&state)?;
    Ok(repairs::get_repair(db.conn(), id)?)
}

#[tauri::command]
pub fn create_repair(
    state: State<'_, DbState>,
    input: RepairInput,
) -> Result<Repair, CommandError> {
    let db = lock_db(&state)?;
    Ok(repairs::create_repair(db.conn(), input)?)
}

#[tauri::command]
pub fn update_repair(
    state: State<'_, DbState>,
    id: i64,
    input: RepairInput,
) -> Result<Repair, CommandError> {
    let db = lock_db(&state)?;
    Ok(repairs::update_repair(db.conn(), id, input)?)
}
