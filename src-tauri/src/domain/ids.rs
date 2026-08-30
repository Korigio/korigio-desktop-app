//! UUID v7 entity identifiers (lowercase 8-4-4-4-12).

use uuid::Uuid;

use crate::error::AppError;

/// Generate a new lowercase UUID v7 for an entity primary key.
pub fn new_entity_id() -> String {
    Uuid::now_v7().as_hyphenated().to_string()
}

/// Parse and reject anything that is not a UUID v7. Returns the canonical lowercase form.
pub fn parse_entity_id(s: &str) -> Result<String, AppError> {
    parse_entity_id_field(s, "id")
}

pub fn parse_entity_id_field(s: &str, field: &str) -> Result<String, AppError> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation {
            field: Some(field.into()),
            message: "This record is required.".into(),
        });
    }
    let parsed = Uuid::parse_str(trimmed).map_err(|_| AppError::Validation {
        field: Some(field.into()),
        message: "Invalid record id.".into(),
    })?;
    if parsed.get_version() != Some(uuid::Version::SortRand) {
        return Err(AppError::Validation {
            field: Some(field.into()),
            message: "Invalid record id.".into(),
        });
    }
    Ok(parsed.as_hyphenated().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_v7() {
        let id = new_entity_id();
        let parsed = Uuid::parse_str(&id).expect("uuid");
        assert_eq!(parsed.get_version(), Some(uuid::Version::SortRand));
        assert_eq!(id, id.to_ascii_lowercase());
    }

    #[test]
    fn rejects_non_v7() {
        let v4 = Uuid::new_v4().to_string();
        assert!(parse_entity_id(&v4).is_err());
        assert!(parse_entity_id("not-a-uuid").is_err());
        assert!(parse_entity_id("").is_err());
    }

    #[test]
    fn accepts_canonical_v7() {
        let id = new_entity_id();
        let again = parse_entity_id(&id).expect("ok");
        assert_eq!(again, id);
    }
}
