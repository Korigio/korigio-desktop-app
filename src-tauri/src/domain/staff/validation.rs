use crate::domain::staff::constants::{NAME_MAX_LEN, PIN_MAX_LEN, PIN_MIN_LEN};
use crate::error::AppError;

pub fn validate_staff_name(name: &str) -> Result<String, AppError> {
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        return Err(AppError::Validation {
            field: Some("name".into()),
            message: "Name is required.".into(),
        });
    }
    if trimmed.chars().count() > NAME_MAX_LEN {
        return Err(AppError::Validation {
            field: Some("name".into()),
            message: format!("Name must be at most {NAME_MAX_LEN} characters."),
        });
    }
    Ok(trimmed)
}

pub fn validate_pin(pin: &str) -> Result<String, AppError> {
    let trimmed = pin.trim();
    if trimmed.is_empty() || !trimmed.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::Validation {
            field: Some("pin".into()),
            message: "PIN must be 4 to 8 digits.".into(),
        });
    }
    let len = trimmed.len();
    if len < PIN_MIN_LEN || len > PIN_MAX_LEN {
        return Err(AppError::Validation {
            field: Some("pin".into()),
            message: "PIN must be 4 to 8 digits.".into(),
        });
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_rules() {
        assert!(validate_pin("1234").is_ok());
        assert!(validate_pin("12345678").is_ok());
        assert!(validate_pin("123").is_err());
        assert!(validate_pin("123456789").is_err());
        assert!(validate_pin("12ab").is_err());
        assert!(validate_pin("  ").is_err());
    }
}
