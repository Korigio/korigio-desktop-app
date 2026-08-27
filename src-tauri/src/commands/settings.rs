//! App settings IPC (locale preference).

use tauri::State;

use crate::db::DbState;
use crate::domain::settings::{self, LocalePreference, LocaleSettings};
use crate::error::{AppError, CommandError};

fn lock_db<'a>(
    state: &'a State<'_, DbState>,
) -> Result<std::sync::MutexGuard<'a, crate::db::Db>, AppError> {
    state.0.lock().map_err(|_| AppError::Internal {
        message: "database lock poisoned".into(),
    })
}

#[tauri::command]
pub fn get_locale_settings(state: State<'_, DbState>) -> Result<LocaleSettings, CommandError> {
    let db = lock_db(&state)?;
    Ok(settings::get_locale_settings(db.conn())?)
}

#[tauri::command]
pub fn set_locale_preference(
    state: State<'_, DbState>,
    preference: LocalePreference,
) -> Result<LocaleSettings, CommandError> {
    let db = lock_db(&state)?;
    Ok(settings::set_locale_preference(db.conn(), preference)?)
}
