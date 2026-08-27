use time::Date;
use time::format_description;

use crate::domain::repairs::constants::{
    ACCESSORIES_RECEIVED_MAX_LEN, DEFAULT_STATUS, DEVICE_CONDITION_MAX_LEN,
    DIAGNOSIS_NOTES_MAX_LEN, NOTES_MAX_LEN, REPAIR_STATUSES, REPORTED_PROBLEM_MAX_LEN,
    WORK_PERFORMED_MAX_LEN,
};
use crate::domain::repairs::types::RepairInput;
use crate::error::AppError;

#[derive(Debug)]
pub struct ValidatedCreateRepairInput {
    pub customer_id: i64,
    pub device_id: i64,
    pub company_id: i64,
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

pub fn validate_create_input(input: &RepairInput) -> Result<ValidatedCreateRepairInput, AppError> {
    if input.customer_id <= 0 {
        return Err(AppError::Validation {
            field: Some("customerId".into()),
            message: "Customer is required.".into(),
        });
    }
    if input.device_id <= 0 {
        return Err(AppError::Validation {
            field: Some("deviceId".into()),
            message: "Device is required.".into(),
        });
    }
    if input.company_id <= 0 {
        return Err(AppError::Validation {
            field: Some("companyId".into()),
            message: "Company is required.".into(),
        });
    }

    let fields = validate_text_fields(input)?;
    let status = normalize_status(input.status.as_deref(), None)?;
    let expected_pickup_at = validate_expected_pickup_at(&input.expected_pickup_at)?;

    Ok(ValidatedCreateRepairInput {
        customer_id: input.customer_id,
        device_id: input.device_id,
        company_id: input.company_id,
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
fn validate_expected_pickup_at(value: &Option<String>) -> Result<Option<String>, AppError> {
    match value {
        None => Ok(None),
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            let format = format_description::parse_borrowed::<2>("[year]-[month]-[day]").map_err(
                |err| AppError::Internal {
                    message: format!("date format parse failed: {err}"),
                },
            )?;
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

fn normalize_status(
    status: Option<&str>,
    existing: Option<&str>,
) -> Result<String, AppError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> RepairInput {
        RepairInput {
            customer_id: 1,
            device_id: 2,
            company_id: 3,
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
        input.company_id = 0;
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
