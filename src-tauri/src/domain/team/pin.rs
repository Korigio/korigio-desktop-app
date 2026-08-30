//! 6-digit team join PIN. SHA-256 hex is stored on `teams`; plaintext stays on this PC.

use rand::Rng;
use sha2::{Digest, Sha256};

use crate::domain::team::constants::TEAM_PIN_LEN;
use crate::error::AppError;

pub fn generate_team_pin() -> String {
    let mut rng = rand::thread_rng();
    (0..TEAM_PIN_LEN)
        .map(|_| char::from(b'0' + rng.gen_range(0..10)))
        .collect()
}

pub fn hash_team_pin(pin: &str) -> String {
    hex::encode(Sha256::digest(pin.as_bytes()))
}

pub fn validate_team_pin(pin: &str) -> Result<String, AppError> {
    let trimmed = pin.trim();
    if trimmed.len() != TEAM_PIN_LEN || !trimmed.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::Validation {
            field: Some("pin".into()),
            message: "PIN must be 6 digits.".into(),
        });
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_six_digits() {
        let pin = generate_team_pin();
        assert_eq!(pin.len(), 6);
        assert!(pin.chars().all(|c| c.is_ascii_digit()));
        assert_eq!(hash_team_pin(&pin).len(), 64);
        assert_ne!(hash_team_pin(&pin), pin);
    }

    #[test]
    fn rejects_bad_pin() {
        assert!(validate_team_pin("12345").is_err());
        assert!(validate_team_pin("1234567").is_err());
        assert!(validate_team_pin("12ab56").is_err());
        assert!(validate_team_pin("123456").is_ok());
    }
}
