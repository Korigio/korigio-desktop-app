use std::path::{Path, PathBuf};

use time::format_description;
use time::Date;

use crate::domain::repairs::constants::{
    ACCESSORIES_RECEIVED_MAX_LEN, ALLOWED_REPAIR_DOCUMENT_EXTENSIONS, DEFAULT_STATUS,
    DEVICE_CONDITION_MAX_LEN, DIAGNOSIS_NOTES_MAX_LEN, MAX_REPAIR_DOCUMENT_BYTES, NOTES_MAX_LEN,
    REPAIR_STATUSES, REPORTED_PROBLEM_MAX_LEN, WORK_PERFORMED_MAX_LEN,
};
use crate::domain::repairs::money::tax_cents_from_base;
use crate::domain::repairs::types::{
    CompleteDiagnosisMode, CompleteRepairDiagnosisInput, RepairDocumentType, RepairInput,
};
use crate::domain::settings::parse_tax_rate_percent;
use crate::error::AppError;

#[derive(Debug)]
pub struct ValidatedCreateRepairInput {
    pub customer_id: String,
    pub device_id: String,
    pub company_id: String,
    pub status: String,
    pub reported_problem: Option<String>,
    pub accessories_received: Option<String>,
    pub device_condition: Option<String>,
    pub diagnosis_notes: Option<String>,
    pub work_performed: Option<String>,
    pub notes: Option<String>,
    pub expected_pickup_at: Option<String>,
}

#[derive(Debug)]
pub struct ValidatedUpdateRepairInput {
    pub status: String,
    pub reported_problem: Option<String>,
    pub accessories_received: Option<String>,
    pub device_condition: Option<String>,
    pub diagnosis_notes: Option<String>,
    pub work_performed: Option<String>,
    pub notes: Option<String>,
    pub expected_pickup_at: Option<String>,
}

#[derive(Debug)]
pub struct ValidatedCompleteDiagnosis {
    pub diagnosis_notes: Option<String>,
    pub expected_pickup_at: Option<String>,
    /// None means leave estimate columns untouched (only when never set).
    pub estimate: Option<EstimateSnapshot>,
    pub next_status: String,
}

#[derive(Debug, Clone, Copy)]
pub struct EstimateSnapshot {
    pub base_cents: i64,
    pub tax_rate_bps: i64,
    pub tax_cents: i64,
    pub gross_cents: i64,
}

pub fn validate_create_input(input: &RepairInput) -> Result<ValidatedCreateRepairInput, AppError> {
    let customer_id = crate::domain::ids::parse_entity_id_field(&input.customer_id, "customerId")?;
    let device_id = crate::domain::ids::parse_entity_id_field(&input.device_id, "deviceId")?;
    let company_id = crate::domain::ids::parse_entity_id_field(&input.company_id, "companyId")?;

    let fields = validate_text_fields(input)?;
    let status = normalize_status(input.status.as_deref(), None)?;
    let expected_pickup_at = validate_expected_pickup_at(&input.expected_pickup_at)?;

    Ok(ValidatedCreateRepairInput {
        customer_id,
        device_id,
        company_id,
        status,
        reported_problem: fields.reported_problem,
        accessories_received: fields.accessories_received,
        device_condition: fields.device_condition,
        diagnosis_notes: fields.diagnosis_notes,
        work_performed: fields.work_performed,
        notes: fields.notes,
        expected_pickup_at,
    })
}

pub fn validate_update_input(
    input: &RepairInput,
    existing_status: &str,
) -> Result<ValidatedUpdateRepairInput, AppError> {
    if let Some(raw) = input.status.as_deref() {
        let trimmed = raw.trim();
        if !trimmed.is_empty() && trimmed != existing_status && trimmed != "cancelled" {
            return Err(AppError::Validation {
                field: Some("status".into()),
                message: "Status can only be changed through workflow actions.".into(),
            });
        }
    }

    let fields = validate_text_fields(input)?;
    let status = normalize_status(input.status.as_deref(), Some(existing_status))?;
    let expected_pickup_at = validate_expected_pickup_at(&input.expected_pickup_at)?;

    Ok(ValidatedUpdateRepairInput {
        status,
        reported_problem: fields.reported_problem,
        accessories_received: fields.accessories_received,
        device_condition: fields.device_condition,
        diagnosis_notes: fields.diagnosis_notes,
        work_performed: fields.work_performed,
        notes: fields.notes,
        expected_pickup_at,
    })
}

struct ValidatedTextFields {
    reported_problem: Option<String>,
    accessories_received: Option<String>,
    device_condition: Option<String>,
    diagnosis_notes: Option<String>,
    work_performed: Option<String>,
    notes: Option<String>,
}

fn validate_text_fields(input: &RepairInput) -> Result<ValidatedTextFields, AppError> {
    Ok(ValidatedTextFields {
        reported_problem: normalize_optional(
            &input.reported_problem,
            "reportedProblem",
            REPORTED_PROBLEM_MAX_LEN,
        )?,
        accessories_received: normalize_optional(
            &input.accessories_received,
            "accessoriesReceived",
            ACCESSORIES_RECEIVED_MAX_LEN,
        )?,
        device_condition: normalize_optional(
            &input.device_condition,
            "deviceCondition",
            DEVICE_CONDITION_MAX_LEN,
        )?,
        diagnosis_notes: normalize_optional(
            &input.diagnosis_notes,
            "diagnosisNotes",
            DIAGNOSIS_NOTES_MAX_LEN,
        )?,
        work_performed: normalize_optional(
            &input.work_performed,
            "workPerformed",
            WORK_PERFORMED_MAX_LEN,
        )?,
        notes: normalize_optional(&input.notes, "notes", NOTES_MAX_LEN)?,
    })
}

/// Optional date-only field. Empty/whitespace → None. Present values must be `YYYY-MM-DD`.
pub(crate) fn validate_expected_pickup_at(
    value: &Option<String>,
) -> Result<Option<String>, AppError> {
    match value {
        None => Ok(None),
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            let format =
                format_description::parse_borrowed::<2>("[year]-[month]-[day]").map_err(|err| {
                    AppError::Internal {
                        message: format!("date format parse failed: {err}"),
                    }
                })?;
            match Date::parse(trimmed, &format) {
                Ok(date) => {
                    let formatted = date.format(&format).map_err(|err| AppError::Internal {
                        message: format!("date format failed: {err}"),
                    })?;
                    Ok(Some(formatted))
                }
                Err(_) => Err(AppError::Validation {
                    field: Some("expectedPickupAt".into()),
                    message: "Expected pickup date must be YYYY-MM-DD.".into(),
                }),
            }
        }
    }
}

/// Validate diagnosis-flow save (draft or finalize). Does not upsert checklist rows.
pub fn validate_complete_diagnosis(
    input: &CompleteRepairDiagnosisInput,
    existing_status: &str,
    existing_estimate_base: Option<i64>,
    tax_rate_percent: &str,
) -> Result<ValidatedCompleteDiagnosis, AppError> {
    if existing_status == "cancelled" {
        return Err(AppError::Validation {
            field: Some("status".into()),
            message: "Cancelled repairs cannot be diagnosed.".into(),
        });
    }

    if existing_status == "collected" {
        return Err(AppError::Validation {
            field: Some("status".into()),
            message: "Collected repairs cannot be diagnosed.".into(),
        });
    }

    let diagnosis_notes = normalize_optional(
        &input.diagnosis_notes,
        "diagnosisNotes",
        DIAGNOSIS_NOTES_MAX_LEN,
    )?;

    if input.mode == CompleteDiagnosisMode::Finalize
        && diagnosis_notes
            .as_ref()
            .map(|s| s.is_empty())
            .unwrap_or(true)
    {
        return Err(AppError::Validation {
            field: Some("diagnosisNotes".into()),
            message: "Diagnosis notes are required to finalize.".into(),
        });
    }

    let expected_pickup_at = validate_expected_pickup_at(&input.expected_pickup_at)?;

    let estimate = match input.estimate_base_cents {
        None => {
            if existing_estimate_base.is_some() {
                return Err(AppError::Validation {
                    field: Some("estimateBaseCents".into()),
                    message: "Estimate cannot be cleared once set.".into(),
                });
            }
            None
        }
        Some(base) => {
            if base < 0 {
                return Err(AppError::Validation {
                    field: Some("estimateBaseCents".into()),
                    message: "Estimate must be zero or greater.".into(),
                });
            }
            let (_, tax_rate_bps) = parse_tax_rate_percent(tax_rate_percent)?;
            let tax_cents = tax_cents_from_base(base, tax_rate_bps)?;
            let gross_cents = base.checked_add(tax_cents).ok_or(AppError::Validation {
                field: Some("estimateBaseCents".into()),
                message: "Estimate amount is too large.".into(),
            })?;
            Some(EstimateSnapshot {
                base_cents: base,
                tax_rate_bps,
                tax_cents,
                gross_cents,
            })
        }
    };

    let next_status = resolve_diagnosis_status(input.mode, existing_status);

    Ok(ValidatedCompleteDiagnosis {
        diagnosis_notes,
        expected_pickup_at,
        estimate,
        next_status,
    })
}

fn resolve_diagnosis_status(mode: CompleteDiagnosisMode, current: &str) -> String {
    match mode {
        CompleteDiagnosisMode::Draft => {
            if matches!(current, "received" | "diagnosis") {
                "diagnosis".into()
            } else {
                current.to_string()
            }
        }
        CompleteDiagnosisMode::Finalize => {
            if matches!(current, "received" | "diagnosis" | "waiting_customer") {
                "waiting_customer".into()
            } else {
                current.to_string()
            }
        }
    }
}

fn normalize_status(status: Option<&str>, existing: Option<&str>) -> Result<String, AppError> {
    match status {
        None => Ok(existing.unwrap_or(DEFAULT_STATUS).to_string()),
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Ok(existing.unwrap_or(DEFAULT_STATUS).to_string());
            }
            if !REPAIR_STATUSES.contains(&trimmed) {
                return Err(AppError::Validation {
                    field: Some("status".into()),
                    message: "Invalid repair status.".into(),
                });
            }
            Ok(trimmed.to_string())
        }
    }
}

fn normalize_optional(
    value: &Option<String>,
    field: &str,
    max_len: usize,
) -> Result<Option<String>, AppError> {
    match value {
        None => Ok(None),
        Some(raw) => {
            let trimmed = raw.trim().to_string();
            if trimmed.is_empty() {
                return Ok(None);
            }
            if trimmed.chars().count() > max_len {
                return Err(AppError::Validation {
                    field: Some(field.into()),
                    message: format!("{field} must be at most {max_len} characters."),
                });
            }
            Ok(Some(trimmed))
        }
    }
}

pub fn validate_repair_document_source(path: &Path) -> Result<(), AppError> {
    if !path.is_file() {
        return Err(AppError::Validation {
            field: Some("sourcePath".into()),
            message: "Document file was not found.".into(),
        });
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    if !ALLOWED_REPAIR_DOCUMENT_EXTENSIONS
        .iter()
        .any(|allowed| *allowed == ext)
    {
        return Err(AppError::Validation {
            field: Some("sourcePath".into()),
            message: "Only PDF, JPEG, and PNG documents are supported.".into(),
        });
    }

    let meta = std::fs::metadata(path)?;
    if meta.len() > MAX_REPAIR_DOCUMENT_BYTES {
        return Err(AppError::Validation {
            field: Some("sourcePath".into()),
            message: format!(
                "Document must be at most {} MB.",
                MAX_REPAIR_DOCUMENT_BYTES / (1024 * 1024)
            ),
        });
    }

    Ok(())
}

pub fn validate_repair_document_source_path(source_path: &str) -> Result<PathBuf, AppError> {
    let path = PathBuf::from(source_path.trim());
    validate_repair_document_source(&path)?;
    Ok(path)
}

pub fn validate_repair_document_type(raw: &str) -> Result<RepairDocumentType, AppError> {
    let trimmed = raw.trim();
    let parsed = match trimmed {
        "entranceSigned" | "entrance_signed" => RepairDocumentType::EntranceSigned,
        "diagnosisSigned" | "diagnosis_signed" => RepairDocumentType::DiagnosisSigned,
        "summarySigned" | "summary_signed" => RepairDocumentType::SummarySigned,
        _ => {
            return Err(AppError::Validation {
                field: Some("documentType".into()),
                message: "Invalid document type.".into(),
            });
        }
    };
    Ok(parsed)
}

pub fn validate_workflow_status(
    existing_status: &str,
    required_status: &str,
    action: &str,
) -> Result<(), AppError> {
    if existing_status == "cancelled" {
        return Err(AppError::Validation {
            field: Some("status".into()),
            message: format!("Cancelled repairs cannot {action}."),
        });
    }
    if existing_status != required_status {
        return Err(AppError::Validation {
            field: Some("status".into()),
            message: format!("Repair must be in '{required_status}' status to {action}."),
        });
    }
    Ok(())
}

pub fn validate_work_performed(value: &str) -> Result<String, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation {
            field: Some("workPerformed".into()),
            message: "Work performed is required.".into(),
        });
    }
    if trimmed.chars().count() > WORK_PERFORMED_MAX_LEN {
        return Err(AppError::Validation {
            field: Some("workPerformed".into()),
            message: format!("workPerformed must be at most {WORK_PERFORMED_MAX_LEN} characters."),
        });
    }
    Ok(trimmed.to_string())
}

/// Date-only pickup field (`YYYY-MM-DD`) stored as RFC3339 midnight UTC.
pub fn validate_collected_at_date(value: &str) -> Result<String, AppError> {
    let trimmed = value.trim();
    let format =
        format_description::parse_borrowed::<2>("[year]-[month]-[day]").map_err(|err| {
            AppError::Internal {
                message: format!("date format parse failed: {err}"),
            }
        })?;
    let date = Date::parse(trimmed, &format).map_err(|_| AppError::Validation {
        field: Some("collectedAt".into()),
        message: "Collection date must be YYYY-MM-DD.".into(),
    })?;
    let formatted = date.format(&format).map_err(|err| AppError::Internal {
        message: format!("date format failed: {err}"),
    })?;
    Ok(format!("{formatted}T00:00:00Z"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> RepairInput {
        RepairInput {
            customer_id: crate::domain::ids::new_entity_id(),
            device_id: crate::domain::ids::new_entity_id(),
            company_id: crate::domain::ids::new_entity_id(),
            status: None,
            reported_problem: None,
            accessories_received: None,
            device_condition: None,
            diagnosis_notes: None,
            work_performed: None,
            notes: None,
            expected_pickup_at: None,
        }
    }

    #[test]
    fn rejects_missing_company_id() {
        let mut input = base_input();
        input.company_id = String::new();
        let err = validate_create_input(&input).expect_err("company required");
        match err {
            AppError::Validation { field, .. } => {
                assert_eq!(field.as_deref(), Some("companyId"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn defaults_status_on_create() {
        let ok = validate_create_input(&base_input()).expect("valid");
        assert_eq!(ok.status, DEFAULT_STATUS);
        assert!(ok.expected_pickup_at.is_none());
    }

    #[test]
    fn rejects_invalid_status() {
        let mut input = base_input();
        input.status = Some("broken".into());
        let err = validate_create_input(&input).expect_err("invalid");
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn accepts_valid_expected_pickup_at() {
        let mut input = base_input();
        input.expected_pickup_at = Some("2026-09-15".into());
        let ok = validate_create_input(&input).expect("valid");
        assert_eq!(ok.expected_pickup_at.as_deref(), Some("2026-09-15"));
    }

    #[test]
    fn empty_expected_pickup_at_becomes_none() {
        let mut input = base_input();
        input.expected_pickup_at = Some("  ".into());
        let ok = validate_create_input(&input).expect("valid");
        assert!(ok.expected_pickup_at.is_none());
    }

    #[test]
    fn rejects_invalid_expected_pickup_at_format() {
        let mut input = base_input();
        input.expected_pickup_at = Some("15-09-2026".into());
        let err = validate_create_input(&input).expect_err("invalid date");
        match err {
            AppError::Validation { field, .. } => {
                assert_eq!(field.as_deref(), Some("expectedPickupAt"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn rejects_nonexistent_calendar_date() {
        let mut input = base_input();
        input.expected_pickup_at = Some("2026-02-30".into());
        let err = validate_create_input(&input).expect_err("invalid calendar date");
        assert!(matches!(
            err,
            AppError::Validation {
                field: Some(ref f),
                ..
            } if f == "expectedPickupAt"
        ));
    }
}
