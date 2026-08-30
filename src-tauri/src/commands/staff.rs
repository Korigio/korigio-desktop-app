//! Thin staff / session IPC adapters.

use tauri::State;

use crate::db::DbState;
use crate::domain::staff::{
    self, Session, Staff, StaffInput, StaffListQuery, StaffListResult, StaffNameInput,
    StaffPinInput, StaffRole,
};
use crate::error::{AppError, CommandError};

fn lock_db<'a>(
    state: &'a State<'_, DbState>,
) -> Result<std::sync::MutexGuard<'a, crate::db::Db>, AppError> {
    state.0.lock().map_err(|_| AppError::Internal {
        message: "database lock poisoned".into(),
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn list_staff(
    state: State<'_, DbState>,
    query: StaffListQuery,
) -> Result<StaffListResult, CommandError> {
    let db = lock_db(&state)?;
    Ok(staff::list_staff(db.conn(), query)?)
}

#[tauri::command]
pub fn get_staff(state: State<'_, DbState>, id: String) -> Result<Staff, CommandError> {
    let db = lock_db(&state)?;
    Ok(staff::get_staff(db.conn(), &id)?)
}

#[tauri::command]
pub fn create_staff(state: State<'_, DbState>, input: StaffInput) -> Result<Staff, CommandError> {
    let db = lock_db(&state)?;
    Ok(staff::create_staff(db.conn(), input)?)
}

#[tauri::command]
pub fn update_staff(
    state: State<'_, DbState>,
    id: String,
    input: StaffNameInput,
) -> Result<Staff, CommandError> {
    let db = lock_db(&state)?;
    Ok(staff::update_staff(db.conn(), &id, input)?)
}

#[tauri::command]
pub fn deactivate_staff(state: State<'_, DbState>, id: String) -> Result<Staff, CommandError> {
    let db = lock_db(&state)?;
    Ok(staff::deactivate_staff(db.conn(), &id)?)
}

#[tauri::command]
pub fn reactivate_staff(state: State<'_, DbState>, id: String) -> Result<Staff, CommandError> {
    let db = lock_db(&state)?;
    Ok(staff::reactivate_staff(db.conn(), &id)?)
}

#[tauri::command]
pub fn change_staff_role(
    state: State<'_, DbState>,
    id: String,
    role: StaffRole,
) -> Result<Staff, CommandError> {
    let db = lock_db(&state)?;
    Ok(staff::change_staff_role(db.conn(), &id, role)?)
}

#[tauri::command]
pub fn set_staff_pin(
    state: State<'_, DbState>,
    id: String,
    input: StaffPinInput,
) -> Result<(), CommandError> {
    let db = lock_db(&state)?;
    Ok(staff::set_staff_pin(db.conn(), &id, input)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn sign_in_staff(
    state: State<'_, DbState>,
    staff_id: String,
    pin: String,
) -> Result<Session, CommandError> {
    let db = lock_db(&state)?;
    Ok(staff::sign_in_staff(db.conn(), &staff_id, &pin)?)
}

#[tauri::command]
pub fn sign_out_staff(state: State<'_, DbState>) -> Result<(), CommandError> {
    let db = lock_db(&state)?;
    Ok(staff::sign_out_staff(db.conn())?)
}

#[tauri::command]
pub fn get_current_session(state: State<'_, DbState>) -> Result<Option<Session>, CommandError> {
    let db = lock_db(&state)?;
    Ok(staff::get_current_session(db.conn())?)
}
