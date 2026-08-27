use std::path::{Path, PathBuf};

use crate::domain::companies::constants::{
    ADDRESS_MAX_LEN, ALLOWED_LOGO_EXTENSIONS, EMAIL_MAX_LEN, LEGAL_NAME_MAX_LEN, MAX_LOGO_BYTES,
    PHONE_MAX_LEN, TAX_ID_MAX_LEN, TRADE_NAME_MAX_LEN, WEBSITE_MAX_LEN,
};
use crate::domain::companies::types::{AttachCompanyLogoInput, CompanyInput};
use crate::error::AppError;

#[derive(Debug)]
pub struct ValidatedCompanyInput {
    pub legal_name: String,
    pub trade_name: Option<String>,
    pub tax_id: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug)]
pub struct ValidatedAttachLogoInput {
    pub company_id: i64,
    pub source_path: PathBuf,
}

pub fn validate_company_input(input: &CompanyInput) -> Result<ValidatedCompanyInput, AppError> {
    let legal_name = input.legal_name.trim().to_string();
    if legal_name.is_empty() {
        return Err(AppError::Validation {
            field: Some("legalName".into()),
            message: "Legal name is required.".into(),
        });
    }
    if legal_name.chars().count() > LEGAL_NAME_MAX_LEN {
        return Err(AppError::Validation {
            field: Some("legalName".into()),
            message: format!("Legal name must be at most {LEGAL_NAME_MAX_LEN} characters."),
        });
    }

    let trade_name = normalize_optional(&input.trade_name, "tradeName", TRADE_NAME_MAX_LEN)?;
    let tax_id = normalize_optional(&input.tax_id, "taxId", TAX_ID_MAX_LEN)?;
    let address = normalize_optional(&input.address, "address", ADDRESS_MAX_LEN)?;
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
    let website = normalize_optional(&input.website, "website", WEBSITE_MAX_LEN)?;

    Ok(ValidatedCompanyInput {
        legal_name,
        trade_name,
        tax_id,
        address,
        phone,
        email,
        website,
    })
}

pub fn validate_attach_logo_input(
    input: &AttachCompanyLogoInput,
) -> Result<ValidatedAttachLogoInput, AppError> {
    if input.company_id <= 0 {
        return Err(AppError::Validation {
            field: Some("companyId".into()),
            message: "Company is required.".into(),
        });
    }

    let path = PathBuf::from(input.source_path.trim());
    validate_logo_source(&path)?;

    Ok(ValidatedAttachLogoInput {
        company_id: input.company_id,
        source_path: path,
    })
}

pub fn validate_logo_source(path: &Path) -> Result<(), AppError> {
    if !path.is_file() {
        return Err(AppError::Validation {
            field: Some("sourcePath".into()),
            message: "Image file was not found.".into(),
        });
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    if !ALLOWED_LOGO_EXTENSIONS.iter().any(|allowed| *allowed == ext) {
        return Err(AppError::Validation {
            field: Some("sourcePath".into()),
            message: "Only JPEG and PNG images are supported.".into(),
        });
    }

    let meta = std::fs::metadata(path)?;
    if meta.len() > MAX_LOGO_BYTES {
        return Err(AppError::Validation {
            field: Some("sourcePath".into()),
            message: format!(
                "Logo must be at most {} MB.",
                MAX_LOGO_BYTES / (1024 * 1024)
            ),
        });
    }

    Ok(())
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
    fn rejects_empty_legal_name() {
        let err = validate_company_input(&CompanyInput {
            legal_name: "  ".into(),
            trade_name: None,
            tax_id: None,
            address: None,
            phone: None,
            email: None,
            website: None,
        })
        .expect_err("empty legal name");
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn accepts_minimal_valid_company() {
        let ok = validate_company_input(&CompanyInput {
            legal_name: "Acme GmbH".into(),
            trade_name: Some(" ".into()),
            tax_id: None,
            address: None,
            phone: None,
            email: None,
            website: None,
        })
        .expect("valid");
        assert_eq!(ok.legal_name, "Acme GmbH");
        assert!(ok.trade_name.is_none());
    }
}
