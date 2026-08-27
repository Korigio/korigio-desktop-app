//! Thin print IPC adapters.

use tauri::State;

use crate::db::DbState;
use crate::domain::print::{self, RepairPrintReport};
use crate::error::{AppError, CommandError};

fn lock_db<'a>(
    state: &'a State<'_, DbState>,
) -> Result<std::sync::MutexGuard<'a, crate::db::Db>, AppError> {
    state.0.lock().map_err(|_| AppError::Internal {
        message: "database lock poisoned".into(),
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_repair_print_report(
    state: State<'_, DbState>,
    repair_id: i64,
) -> Result<RepairPrintReport, CommandError> {
    let db = lock_db(&state)?;
    Ok(print::get_repair_print_report(db.conn(), repair_id)?)
}
