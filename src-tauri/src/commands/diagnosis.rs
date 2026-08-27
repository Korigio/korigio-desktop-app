//! Thin diagnosis IPC adapters.

use tauri::State;

use crate::db::DbState;
use crate::domain::diagnosis::{
    self, DiagnosisTemplate, DiagnosisTemplateInput, DiagnosisTemplateListQuery,
    DiagnosisTemplateListResult, RepairDiagnosis, RepairDiagnosisInput,
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
pub fn list_diagnosis_templates(
    state: State<'_, DbState>,
    query: DiagnosisTemplateListQuery,
) -> Result<DiagnosisTemplateListResult, CommandError> {
    let db = lock_db(&state)?;
    Ok(diagnosis::list_diagnosis_templates(db.conn(), query)?)
}

#[tauri::command]
pub fn get_diagnosis_template(
    state: State<'_, DbState>,
    id: i64,
) -> Result<DiagnosisTemplate, CommandError> {
    let db = lock_db(&state)?;
    Ok(diagnosis::get_diagnosis_template(db.conn(), id)?)
}

#[tauri::command]
pub fn create_diagnosis_template(
    state: State<'_, DbState>,
    input: DiagnosisTemplateInput,
) -> Result<DiagnosisTemplate, CommandError> {
    let db = lock_db(&state)?;
    Ok(diagnosis::create_diagnosis_template(db.conn(), input)?)
}

#[tauri::command]
pub fn update_diagnosis_template(
    state: State<'_, DbState>,
    id: i64,
    input: DiagnosisTemplateInput,
) -> Result<DiagnosisTemplate, CommandError> {
    let db = lock_db(&state)?;
    Ok(diagnosis::update_diagnosis_template(db.conn(), id, input)?)
}

#[tauri::command]
pub fn delete_diagnosis_template(
    state: State<'_, DbState>,
    id: i64,
) -> Result<(), CommandError> {
    let db = lock_db(&state)?;
    Ok(diagnosis::delete_diagnosis_template(db.conn(), id)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn get_repair_diagnosis(
    state: State<'_, DbState>,
    repair_id: i64,
) -> Result<Option<RepairDiagnosis>, CommandError> {
    let db = lock_db(&state)?;
    Ok(diagnosis::get_repair_diagnosis(db.conn(), repair_id)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn upsert_repair_diagnosis(
    state: State<'_, DbState>,
    input: RepairDiagnosisInput,
) -> Result<RepairDiagnosis, CommandError> {
    let db = lock_db(&state)?;
    Ok(diagnosis::upsert_repair_diagnosis(db.conn(), input)?)
}
