//! Thin company IPC adapters.

use tauri::State;

use crate::db::DbState;
use crate::domain::companies::{
    self, AttachCompanyLogoInput, Company, CompanyInput, CompanyListQuery, CompanyListResult,
    ResolveCompanyLogoPathResult,
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
pub fn list_companies(
    state: State<'_, DbState>,
    query: CompanyListQuery,
) -> Result<CompanyListResult, CommandError> {
    let db = lock_db(&state)?;
    Ok(companies::list_companies(db.conn(), query)?)
}

#[tauri::command]
pub fn get_company(state: State<'_, DbState>, id: i64) -> Result<Company, CommandError> {
    let db = lock_db(&state)?;
    Ok(companies::get_company(db.conn(), id)?)
}

#[tauri::command]
pub fn create_company(
    state: State<'_, DbState>,
    input: CompanyInput,
) -> Result<Company, CommandError> {
    let db = lock_db(&state)?;
    Ok(companies::create_company(db.conn(), input)?)
}

#[tauri::command]
pub fn update_company(
    state: State<'_, DbState>,
    id: i64,
    input: CompanyInput,
) -> Result<Company, CommandError> {
    let db = lock_db(&state)?;
    Ok(companies::update_company(db.conn(), id, input)?)
}

#[tauri::command]
pub fn archive_company(state: State<'_, DbState>, id: i64) -> Result<Company, CommandError> {
    let db = lock_db(&state)?;
    Ok(companies::archive_company(db.conn(), id)?)
}

#[tauri::command]
pub fn unarchive_company(state: State<'_, DbState>, id: i64) -> Result<Company, CommandError> {
    let db = lock_db(&state)?;
    Ok(companies::unarchive_company(db.conn(), id)?)
}

#[tauri::command]
pub fn set_default_company(state: State<'_, DbState>, id: i64) -> Result<Company, CommandError> {
    let db = lock_db(&state)?;
    Ok(companies::set_default_company(db.conn(), id)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn attach_company_logo(
    state: State<'_, DbState>,
    input: AttachCompanyLogoInput,
) -> Result<Company, CommandError> {
    let db = lock_db(&state)?;
    Ok(companies::attach_company_logo(&db, input)?)
}

#[tauri::command]
pub fn clear_company_logo(state: State<'_, DbState>, id: i64) -> Result<Company, CommandError> {
    let db = lock_db(&state)?;
    Ok(companies::clear_company_logo(&db, id)?)
}

#[tauri::command]
pub fn resolve_company_logo_path(
    state: State<'_, DbState>,
    id: i64,
) -> Result<ResolveCompanyLogoPathResult, CommandError> {
    let db = lock_db(&state)?;
    Ok(companies::resolve_company_logo_path(&db, id)?)
}
