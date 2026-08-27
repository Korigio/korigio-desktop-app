use crate::domain::devices::constants::{
    ACCESSORIES_MAX_LEN, DEVICE_TYPE_MAX_LEN, MANUFACTURER_MAX_LEN, MODEL_MAX_LEN, NOTES_MAX_LEN,
    SERIAL_MAX_LEN,
};
use crate::domain::devices::types::DeviceInput;
use crate::error::AppError;

#[derive(Debug)]
pub struct ValidatedDeviceInput {
    pub customer_id: i64,
    pub device_type: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub accessories: Option<String>,
    pub notes: Option<String>,
}

pub fn validate_device_input(input: &DeviceInput) -> Result<ValidatedDeviceInput, AppError> {
    if input.customer_id <= 0 {
        return Err(AppError::Validation {
            field: Some("customerId".into()),
            message: "Customer is required.".into(),
        });
    }

    let device_type = normalize_optional(&input.device_type, "deviceType", DEVICE_TYPE_MAX_LEN)?;
    let manufacturer =
        normalize_optional(&input.manufacturer, "manufacturer", MANUFACTURER_MAX_LEN)?;
    let model = normalize_optional(&input.model, "model", MODEL_MAX_LEN)?;
    let serial_number =
        normalize_optional(&input.serial_number, "serialNumber", SERIAL_MAX_LEN)?;
    let accessories =
        normalize_optional(&input.accessories, "accessories", ACCESSORIES_MAX_LEN)?;
    let notes = normalize_optional(&input.notes, "notes", NOTES_MAX_LEN)?;

    if device_type.is_none()
        && manufacturer.is_none()
        && model.is_none()
        && serial_number.is_none()
    {
        return Err(AppError::Validation {
            field: None,
            message: "Enter at least a type, manufacturer, model, or serial number.".into(),
        });
    }

    Ok(ValidatedDeviceInput {
        customer_id: input.customer_id,
        device_type,
        manufacturer,
        model,
        serial_number,
        accessories,
        notes,
    })
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

    #[test]
    fn rejects_empty_identity() {
        let err = validate_device_input(&DeviceInput {
            customer_id: 1,
            device_type: None,
            manufacturer: Some("  ".into()),
            model: None,
            serial_number: None,
            accessories: None,
            notes: None,
        })
        .expect_err("empty");
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn accepts_serial_only() {
        let ok = validate_device_input(&DeviceInput {
            customer_id: 1,
            device_type: None,
            manufacturer: None,
            model: None,
            serial_number: Some("SN-1".into()),
            accessories: None,
            notes: None,
        })
        .expect("valid");
        assert_eq!(ok.serial_number.as_deref(), Some("SN-1"));
    }
}
