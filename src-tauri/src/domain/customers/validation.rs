use crate::domain::customers::constants::{
    ADDRESS_MAX_LEN, EMAIL_MAX_LEN, NAME_MAX_LEN, NOTES_MAX_LEN, PHONE_MAX_LEN,
};
use crate::domain::customers::types::CustomerInput;
use crate::error::AppError;

#[derive(Debug)]
pub struct ValidatedCustomerInput {
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub notes: Option<String>,
}

pub fn validate_customer_input(input: &CustomerInput) -> Result<ValidatedCustomerInput, AppError> {
    let name = input.name.trim().to_string();
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

    let phone = normalize_optional(&input.phone, "phone", PHONE_MAX_LEN)?;
    let email = normalize_optional(&input.email, "email", EMAIL_MAX_LEN)?;
    if let Some(ref email_value) = email {
        if !email_value.contains('@') || email_value.starts_with('@') || email_value.ends_with('@')
        {
            return Err(AppError::Validation {
                field: Some("email".into()),
                message: "Email format looks invalid.".into(),
            });
        }
    }
    let address = normalize_optional(&input.address, "address", ADDRESS_MAX_LEN)?;
    let notes = normalize_optional(&input.notes, "notes", NOTES_MAX_LEN)?;

    Ok(ValidatedCustomerInput {
        name,
        phone,
        email,
        address,
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
    fn rejects_empty_name() {
        let err = validate_customer_input(&CustomerInput {
            name: "  ".into(),
            phone: None,
            email: None,
            address: None,
            notes: None,
        })
        .expect_err("empty name");
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn accepts_minimal_valid_customer() {
        let ok = validate_customer_input(&CustomerInput {
            name: "Max Mustermann".into(),
            phone: Some(" ".into()),
            email: None,
            address: None,
            notes: None,
        })
        .expect("valid");
        assert_eq!(ok.name, "Max Mustermann");
        assert!(ok.phone.is_none());
    }
}
