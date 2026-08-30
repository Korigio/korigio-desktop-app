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
use crate::domain::sync::{self, begin_write, WriteContext};
use crate::error::AppError;

fn record_template(
    conn: &Connection,
    template: &DiagnosisTemplate,
    ctx: &WriteContext,
) -> Result<(), AppError> {
    let payload = serde_json::to_value(template).map_err(|err| AppError::Internal {
        message: format!("serialize diagnosis template: {err}"),
    })?;
    sync::record_upsert(conn, "diagnosis_templates", &template.id, payload, ctx)?;
    Ok(())
}

fn record_repair_diagnosis(
    conn: &Connection,
    row: &RepairDiagnosis,
    ctx: &WriteContext,
) -> Result<(), AppError> {
    let payload = serde_json::to_value(row).map_err(|err| AppError::Internal {
        message: format!("serialize repair diagnosis: {err}"),
    })?;
    sync::record_upsert(conn, "repair_diagnosis", &row.id, payload, ctx)?;
    Ok(())
}

pub fn create_diagnosis_template(
    conn: &Connection,
    input: DiagnosisTemplateInput,
) -> Result<DiagnosisTemplate, AppError> {
    let validated = validate_template_input(&input)?;
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let template = repository::insert_template(conn, &validated, &now, &ctx)?;
    record_template(conn, &template, &ctx)?;
    Ok(template)
}

pub fn update_diagnosis_template(
    conn: &Connection,
    id: String,
    input: DiagnosisTemplateInput,
) -> Result<DiagnosisTemplate, AppError> {
    let validated = validate_template_input(&input)?;
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let template = repository::update_template(conn, &id, &validated, &now, &ctx)?;
    record_template(conn, &template, &ctx)?;
    Ok(template)
}

pub fn get_diagnosis_template(
    conn: &Connection,
    id: String,
) -> Result<DiagnosisTemplate, AppError> {
    repository::get_template_by_id(conn, &id)?.ok_or(AppError::NotFound)
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

pub fn delete_diagnosis_template(conn: &Connection, id: String) -> Result<(), AppError> {
    let _existing = get_diagnosis_template(conn, id.clone())?;
    let refs = repository::count_repair_diagnosis_by_template(conn, &id)?;
    if refs > 0 {
        return Err(AppError::Validation {
            field: None,
            message: "This template is used by one or more repairs and cannot be deleted.".into(),
        });
    }
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    repository::delete_template(conn, &id, &now, &ctx)?;
    sync::record_delete(
        conn,
        "diagnosis_templates",
        &id,
        serde_json::json!({ "id": id }),
        &ctx,
    )?;
    Ok(())
}

pub fn get_repair_diagnosis(
    conn: &Connection,
    repair_id: String,
) -> Result<Option<RepairDiagnosis>, AppError> {
    repository::get_by_repair_id(conn, &repair_id)
}

pub fn upsert_repair_diagnosis(
    conn: &Connection,
    input: RepairDiagnosisInput,
) -> Result<RepairDiagnosis, AppError> {
    let validated = validate_repair_diagnosis_input(&input)?;
    // Ensure the repair exists (cross-domain via service).
    let _repair = repairs::get_repair(conn, validated.repair_id.clone())?;

    if let Some(template_id) = validated.template_id.clone() {
        let _template = get_diagnosis_template(conn, template_id)?;
    }

    let ctx = begin_write(conn)?;
    let row = repository::upsert_repair_diagnosis(conn, &validated, &ctx)?;
    record_repair_diagnosis(conn, &row, &ctx)?;
    Ok(row)
}
