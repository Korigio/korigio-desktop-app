//! Thin repair IPC adapters.

use tauri::State;

use crate::db::DbState;
use crate::domain::repairs::{
    self, CompleteRepairDiagnosisInput, CompleteRepairDiagnosisResult, Repair, RepairDocument,
    RepairDocumentType, RepairInput, RepairListQuery, RepairListResult,
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

#[tauri::command(rename_all = "camelCase")]
pub fn complete_repair_diagnosis(
    state: State<'_, DbState>,
    input: CompleteRepairDiagnosisInput,
) -> Result<CompleteRepairDiagnosisResult, CommandError> {
    let db = lock_db(&state)?;
    Ok(repairs::complete_repair_diagnosis(db.conn(), input)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn list_repair_documents(
    state: State<'_, DbState>,
    repair_id: i64,
) -> Result<Vec<RepairDocument>, CommandError> {
    let db = lock_db(&state)?;
    Ok(repairs::list_repair_documents(db.conn(), repair_id)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn upload_repair_document(
    state: State<'_, DbState>,
    repair_id: i64,
    document_type: RepairDocumentType,
    source_path: String,
) -> Result<RepairDocument, CommandError> {
    let db = lock_db(&state)?;
    Ok(repairs::upload_repair_document(
        &db,
        repair_id,
        document_type,
        source_path,
    )?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn delete_repair_document(
    state: State<'_, DbState>,
    repair_id: i64,
    document_type: RepairDocumentType,
) -> Result<(), CommandError> {
    let db = lock_db(&state)?;
    Ok(repairs::delete_repair_document(&db, repair_id, document_type)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn open_repair_document(
    state: State<'_, DbState>,
    repair_id: i64,
    document_type: RepairDocumentType,
) -> Result<(), CommandError> {
    let db = lock_db(&state)?;
    Ok(repairs::open_repair_document(&db, repair_id, document_type)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn confirm_repair_intake(
    state: State<'_, DbState>,
    repair_id: i64,
) -> Result<Repair, CommandError> {
    let db = lock_db(&state)?;
    Ok(repairs::confirm_repair_intake(&db, repair_id)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn confirm_customer_approval(
    state: State<'_, DbState>,
    repair_id: i64,
) -> Result<Repair, CommandError> {
    let db = lock_db(&state)?;
    Ok(repairs::confirm_customer_approval(&db, repair_id)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn confirm_repair_parts_received(
    state: State<'_, DbState>,
    repair_id: i64,
) -> Result<Repair, CommandError> {
    let db = lock_db(&state)?;
    Ok(repairs::confirm_repair_parts_received(&db, repair_id)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn complete_repair_protocol(
    state: State<'_, DbState>,
    repair_id: i64,
    work_performed: String,
) -> Result<Repair, CommandError> {
    let db = lock_db(&state)?;
    Ok(repairs::complete_repair_protocol(&db, repair_id, work_performed)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn confirm_repair_summary(
    state: State<'_, DbState>,
    repair_id: i64,
) -> Result<Repair, CommandError> {
    let db = lock_db(&state)?;
    Ok(repairs::confirm_repair_summary(&db, repair_id)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn complete_repair_pickup(
    state: State<'_, DbState>,
    repair_id: i64,
    collected_at: String,
) -> Result<Repair, CommandError> {
    let db = lock_db(&state)?;
    Ok(repairs::complete_repair_pickup(&db, repair_id, collected_at)?)
}
