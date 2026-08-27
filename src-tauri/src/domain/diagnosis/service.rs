use rusqlite::Connection;

use crate::db::repository::now_utc_rfc3339;
use crate::domain::diagnosis::constants::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};
use crate::domain::diagnosis::repository;
use crate::domain::diagnosis::types::{
    DiagnosisTemplate, DiagnosisTemplateInput, DiagnosisTemplateListQuery,
    DiagnosisTemplateListResult, RepairDiagnosis, RepairDiagnosisInput,
};
use crate::domain::diagnosis::validation::{
    validate_repair_diagnosis_input, validate_template_input,
};
use crate::domain::repairs;
use crate::error::AppError;

pub fn create_diagnosis_template(
    conn: &Connection,
    input: DiagnosisTemplateInput,
) -> Result<DiagnosisTemplate, AppError> {
    let validated = validate_template_input(&input)?;
    let now = now_utc_rfc3339()?;
    repository::insert_template(conn, &validated, &now)
}

pub fn update_diagnosis_template(
    conn: &Connection,
    id: i64,
    input: DiagnosisTemplateInput,
) -> Result<DiagnosisTemplate, AppError> {
    let validated = validate_template_input(&input)?;
    let now = now_utc_rfc3339()?;
    repository::update_template(conn, id, &validated, &now)
}

pub fn get_diagnosis_template(conn: &Connection, id: i64) -> Result<DiagnosisTemplate, AppError> {
    repository::get_template_by_id(conn, id)?.ok_or(AppError::NotFound)
}

pub fn list_diagnosis_templates(
    conn: &Connection,
    query: DiagnosisTemplateListQuery,
) -> Result<DiagnosisTemplateListResult, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query
        .page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let offset = (page - 1).saturating_mul(page_size);

    let (items, total) =
        repository::list_templates(conn, query.query.as_deref(), page_size, offset)?;

    Ok(DiagnosisTemplateListResult {
        items,
        total,
        page,
        page_size,
    })
}

pub fn delete_diagnosis_template(conn: &Connection, id: i64) -> Result<(), AppError> {
    let _existing = get_diagnosis_template(conn, id)?;
    let refs = repository::count_repair_diagnosis_by_template(conn, id)?;
    if refs > 0 {
        return Err(AppError::Validation {
            field: None,
            message: "This template is used by one or more repairs and cannot be deleted.".into(),
        });
    }
    repository::delete_template(conn, id)
}

pub fn get_repair_diagnosis(
    conn: &Connection,
    repair_id: i64,
) -> Result<Option<RepairDiagnosis>, AppError> {
    repository::get_by_repair_id(conn, repair_id)
}

pub fn upsert_repair_diagnosis(
    conn: &Connection,
    input: RepairDiagnosisInput,
) -> Result<RepairDiagnosis, AppError> {
    let validated = validate_repair_diagnosis_input(&input)?;
    // Ensure the repair exists (cross-domain via service).
    let _repair = repairs::get_repair(conn, validated.repair_id)?;

    if let Some(template_id) = validated.template_id {
        let _template = get_diagnosis_template(conn, template_id)?;
    }

    repository::upsert_repair_diagnosis(conn, &validated)
}
