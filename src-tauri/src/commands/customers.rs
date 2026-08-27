//! Thin customer IPC adapters.

use tauri::State;

use crate::db::DbState;
use crate::domain::customers::{
    self, Customer, CustomerInput, CustomerListQuery, CustomerListResult,
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
pub fn list_customers(
    state: State<'_, DbState>,
    query: CustomerListQuery,
) -> Result<CustomerListResult, CommandError> {
    let db = lock_db(&state)?;
    Ok(customers::list_customers(db.conn(), query)?)
}

#[tauri::command]
pub fn get_customer(state: State<'_, DbState>, id: i64) -> Result<Customer, CommandError> {
    let db = lock_db(&state)?;
    Ok(customers::get_customer(db.conn(), id)?)
}

#[tauri::command]
pub fn create_customer(
    state: State<'_, DbState>,
    input: CustomerInput,
) -> Result<Customer, CommandError> {
    let db = lock_db(&state)?;
    Ok(customers::create_customer(db.conn(), input)?)
}

#[tauri::command]
pub fn update_customer(
    state: State<'_, DbState>,
    id: i64,
    input: CustomerInput,
) -> Result<Customer, CommandError> {
    let db = lock_db(&state)?;
    Ok(customers::update_customer(db.conn(), id, input)?)
}

#[tauri::command]
pub fn archive_customer(state: State<'_, DbState>, id: i64) -> Result<Customer, CommandError> {
    let db = lock_db(&state)?;
    Ok(customers::archive_customer(db.conn(), id)?)
}

#[tauri::command]
pub fn unarchive_customer(state: State<'_, DbState>, id: i64) -> Result<Customer, CommandError> {
    let db = lock_db(&state)?;
    Ok(customers::unarchive_customer(db.conn(), id)?)
}
