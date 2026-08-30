use crate::domain::team::constants::{DEVICE_NAME_MAX_LEN, MEMBER_NAME_MAX_LEN, NAME_MAX_LEN};
use crate::error::AppError;

pub fn validate_team_name(name: &str) -> Result<String, AppError> {
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        return Err(AppError::Validation {
            field: Some("name".into()),
            message: "Team name is required.".into(),
        });
    }
    if trimmed.chars().count() > NAME_MAX_LEN {
        return Err(AppError::Validation {
            field: Some("name".into()),
            message: format!("Team name must be at most {NAME_MAX_LEN} characters."),
        });
    }
    Ok(trimmed)
}

pub fn validate_device_name(name: &str) -> Result<String, AppError> {
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        return Err(AppError::Validation {
            field: Some("deviceName".into()),
            message: "Device name is required.".into(),
        });
    }
    if trimmed.chars().count() > DEVICE_NAME_MAX_LEN {
        return Err(AppError::Validation {
            field: Some("deviceName".into()),
            message: format!("Device name must be at most {DEVICE_NAME_MAX_LEN} characters."),
        });
    }
    Ok(trimmed)
}

pub fn validate_member_name(name: &str) -> Result<String, AppError> {
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        return Err(AppError::Validation {
            field: Some("memberName".into()),
            message: "Name is required.".into(),
        });
    }
    if trimmed.chars().count() > MEMBER_NAME_MAX_LEN {
        return Err(AppError::Validation {
            field: Some("memberName".into()),
            message: format!("Name must be at most {MEMBER_NAME_MAX_LEN} characters."),
        });
    }
    Ok(trimmed)
}
