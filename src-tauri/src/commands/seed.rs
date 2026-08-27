//! Thin synthetic-seed IPC adapters (always registered; requires confirm: true).

use tauri::State;

use crate::db::DbState;
use crate::domain::seed::{self, SeedSyntheticDataInput, SeedSyntheticDataResult};
use crate::error::{AppError, CommandError};

fn lock_db<'a>(
    state: &'a State<'_, DbState>,
) -> Result<std::sync::MutexGuard<'a, crate::db::Db>, AppError> {
    state.0.lock().map_err(|_| AppError::Internal {
        message: "database lock poisoned".into(),
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn seed_synthetic_data(
    state: State<'_, DbState>,
    input: SeedSyntheticDataInput,
) -> Result<SeedSyntheticDataResult, CommandError> {
    let db = lock_db(&state)?;
    Ok(seed::seed_synthetic_data(db.conn(), input)?)
}
