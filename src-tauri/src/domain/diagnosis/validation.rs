use std::collections::HashSet;

use serde_json::Value;

use crate::domain::diagnosis::constants::{
    KIND_CHECKBOX, KIND_TEXT, LABEL_MAX_LEN, NAME_MAX_LEN, TEXT_VALUE_MAX_LEN,
};
use crate::domain::diagnosis::types::{
    DiagnosisResult, DiagnosisResultItem, DiagnosisTemplateBody, DiagnosisTemplateBodyItem,
    DiagnosisTemplateInput, RepairDiagnosisInput,
};
use crate::error::AppError;

#[derive(Debug)]
pub struct ValidatedTemplateInput {
    pub name: String,
    pub body: DiagnosisTemplateBody,
}

#[derive(Debug)]
pub struct ValidatedRepairDiagnosisInput {
    pub repair_id: String,
    pub template_id: Option<String>,
    pub result: DiagnosisResult,
}

pub fn validate_template_input(
    input: &DiagnosisTemplateInput,
) -> Result<ValidatedTemplateInput, AppError> {
    let name = validate_name(&input.name)?;
    let body = validate_template_body(&input.body)?;
    Ok(ValidatedTemplateInput { name, body })
}

pub fn validate_repair_diagnosis_input(
    input: &RepairDiagnosisInput,
) -> Result<ValidatedRepairDiagnosisInput, AppError> {
    let repair_id = crate::domain::ids::parse_entity_id_field(&input.repair_id, "repairId")?;
    let template_id = match &input.template_id {
        Some(raw) => Some(crate::domain::ids::parse_entity_id_field(
            raw,
            "templateId",
        )?),
        None => None,
    };

    let result = validate_result(&input.result)?;
    Ok(ValidatedRepairDiagnosisInput {
        repair_id,
        template_id,
        result,
    })
}

fn validate_name(raw: &str) -> Result<String, AppError> {
    let name = raw.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Validation {
            field: Some("name".into()),
            message: "Name is required.".into(),
        });
    }
    if name.chars().count() > NAME_MAX_LEN {
        return Err(AppError::Validation {
            field: Some("name".into()),
            message: format!("Name must be at most {NAME_MAX_LEN} characters."),
        });
    }
    Ok(name)
}

fn validate_template_body(body: &DiagnosisTemplateBody) -> Result<DiagnosisTemplateBody, AppError> {
    if body.items.is_empty() {
        return Err(AppError::Validation {
            field: Some("body".into()),
            message: "At least one checklist item is required.".into(),
        });
    }

    let mut seen_ids = HashSet::new();
    let mut items = Vec::with_capacity(body.items.len());

    for (index, item) in body.items.iter().enumerate() {
        items.push(validate_body_item(item, index, &mut seen_ids)?);
    }

    Ok(DiagnosisTemplateBody { items })
}

fn validate_body_item(
    item: &DiagnosisTemplateBodyItem,
    index: usize,
    seen_ids: &mut HashSet<String>,
) -> Result<DiagnosisTemplateBodyItem, AppError> {
    let field_prefix = format!("body.items[{index}]");
    let id = item.id.trim().to_string();
    if id.is_empty() {
        return Err(AppError::Validation {
            field: Some(format!("{field_prefix}.id")),
            message: "Item id is required.".into(),
        });
    }
    if !seen_ids.insert(id.clone()) {
        return Err(AppError::Validation {
            field: Some(format!("{field_prefix}.id")),
            message: "Item ids must be unique.".into(),
        });
    }

    let label = item.label.trim().to_string();
    if label.is_empty() {
        return Err(AppError::Validation {
            field: Some(format!("{field_prefix}.label")),
            message: "Item label is required.".into(),
        });
    }
    if label.chars().count() > LABEL_MAX_LEN {
        return Err(AppError::Validation {
            field: Some(format!("{field_prefix}.label")),
            message: format!("Item label must be at most {LABEL_MAX_LEN} characters."),
        });
    }

    let kind = validate_kind(&item.kind, &format!("{field_prefix}.kind"))?;

    Ok(DiagnosisTemplateBodyItem { id, label, kind })
}

fn validate_result(result: &DiagnosisResult) -> Result<DiagnosisResult, AppError> {
    if result.items.is_empty() {
        return Err(AppError::Validation {
            field: Some("result".into()),
            message: "At least one result item is required.".into(),
        });
    }

    let mut seen_ids = HashSet::new();
    let mut items = Vec::with_capacity(result.items.len());

    for (index, item) in result.items.iter().enumerate() {
        items.push(validate_result_item(item, index, &mut seen_ids)?);
    }

    Ok(DiagnosisResult { items })
}

fn validate_result_item(
    item: &DiagnosisResultItem,
    index: usize,
    seen_ids: &mut HashSet<String>,
) -> Result<DiagnosisResultItem, AppError> {
    let field_prefix = format!("result.items[{index}]");
    let id = item.id.trim().to_string();
    if id.is_empty() {
        return Err(AppError::Validation {
            field: Some(format!("{field_prefix}.id")),
            message: "Item id is required.".into(),
        });
    }
    if !seen_ids.insert(id.clone()) {
        return Err(AppError::Validation {
            field: Some(format!("{field_prefix}.id")),
            message: "Item ids must be unique.".into(),
        });
    }

    let label = item.label.trim().to_string();
    if label.is_empty() {
        return Err(AppError::Validation {
            field: Some(format!("{field_prefix}.label")),
            message: "Item label is required.".into(),
        });
    }
    if label.chars().count() > LABEL_MAX_LEN {
        return Err(AppError::Validation {
            field: Some(format!("{field_prefix}.label")),
            message: format!("Item label must be at most {LABEL_MAX_LEN} characters."),
        });
    }

    let kind = validate_kind(&item.kind, &format!("{field_prefix}.kind"))?;
    let value = validate_value_for_kind(&kind, &item.value, &format!("{field_prefix}.value"))?;

    Ok(DiagnosisResultItem {
        id,
        label,
        kind,
        value,
    })
}

fn validate_kind(raw: &str, field: &str) -> Result<String, AppError> {
    let kind = raw.trim().to_string();
    if kind == KIND_CHECKBOX || kind == KIND_TEXT {
        return Ok(kind);
    }
    Err(AppError::Validation {
        field: Some(field.into()),
        message: "Item kind must be checkbox or text.".into(),
    })
}

fn validate_value_for_kind(kind: &str, value: &Value, field: &str) -> Result<Value, AppError> {
    match kind {
        KIND_CHECKBOX => match value {
            Value::Bool(b) => Ok(Value::Bool(*b)),
            _ => Err(AppError::Validation {
                field: Some(field.into()),
                message: "Checkbox value must be a boolean.".into(),
            }),
        },
        KIND_TEXT => match value {
            Value::String(s) => {
                if s.chars().count() > TEXT_VALUE_MAX_LEN {
                    return Err(AppError::Validation {
                        field: Some(field.into()),
                        message: format!(
                            "Text value must be at most {TEXT_VALUE_MAX_LEN} characters."
                        ),
                    });
                }
                Ok(Value::String(s.clone()))
            }
            _ => Err(AppError::Validation {
                field: Some(field.into()),
                message: "Text value must be a string.".into(),
            }),
        },
        _ => Err(AppError::Validation {
            field: Some(field.into()),
            message: "Item kind must be checkbox or text.".into(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn body_item(id: &str, label: &str, kind: &str) -> DiagnosisTemplateBodyItem {
        DiagnosisTemplateBodyItem {
            id: id.into(),
            label: label.into(),
            kind: kind.into(),
        }
    }

    #[test]
    fn rejects_empty_name() {
        let err = validate_template_input(&DiagnosisTemplateInput {
            name: "  ".into(),
            body: DiagnosisTemplateBody {
                items: vec![body_item("a", "A", KIND_CHECKBOX)],
            },
        })
        .expect_err("empty name");
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn rejects_invalid_kind() {
        let err = validate_template_input(&DiagnosisTemplateInput {
            name: "Checklist".into(),
            body: DiagnosisTemplateBody {
                items: vec![body_item("a", "A", "number")],
            },
        })
        .expect_err("bad kind");
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn rejects_mismatched_checkbox_value() {
        let err = validate_repair_diagnosis_input(&RepairDiagnosisInput {
            repair_id: crate::domain::ids::new_entity_id(),
            template_id: None,
            result: DiagnosisResult {
                items: vec![DiagnosisResultItem {
                    id: "a".into(),
                    label: "A".into(),
                    kind: KIND_CHECKBOX.into(),
                    value: json!("yes"),
                }],
            },
        })
        .expect_err("bad value");
        assert!(matches!(err, AppError::Validation { .. }));
    }
}
