//! Thin device IPC adapters.

use tauri::State;

use crate::db::DbState;
use crate::domain::devices::{
    self, Device, DeviceInput, DeviceListQuery, DeviceListResult,
};
use crate::error::{AppError, CommandError};

fn lock_db<'a>(
    state: &'a State<'_, DbState>,
) -> Result<std::sync::MutexGuard<'a, crate::db::Db>, AppError> {
    state.0.lock().map_err(|_| AppError::Internal {
        message: "database lock poisoned".into(),
    })
}

#[tauri::command]
pub fn list_devices(
    state: State<'_, DbState>,
    query: DeviceListQuery,
) -> Result<DeviceListResult, CommandError> {
    let db = lock_db(&state)?;
    Ok(devices::list_devices(db.conn(), query)?)
}

#[tauri::command]
pub fn get_device(state: State<'_, DbState>, id: i64) -> Result<Device, CommandError> {
    let db = lock_db(&state)?;
    Ok(devices::get_device(db.conn(), id)?)
}

#[tauri::command]
pub fn create_device(
    state: State<'_, DbState>,
    input: DeviceInput,
) -> Result<Device, CommandError> {
    let db = lock_db(&state)?;
    Ok(devices::create_device(db.conn(), input)?)
}

#[tauri::command]
pub fn update_device(
    state: State<'_, DbState>,
    id: i64,
    input: DeviceInput,
) -> Result<Device, CommandError> {
    let db = lock_db(&state)?;
    Ok(devices::update_device(db.conn(), id, input)?)
}

#[tauri::command]
pub fn archive_device(state: State<'_, DbState>, id: i64) -> Result<Device, CommandError> {
    let db = lock_db(&state)?;
    Ok(devices::archive_device(db.conn(), id)?)
}

#[tauri::command]
pub fn unarchive_device(state: State<'_, DbState>, id: i64) -> Result<Device, CommandError> {
    let db = lock_db(&state)?;
    Ok(devices::unarchive_device(db.conn(), id)?)
}
