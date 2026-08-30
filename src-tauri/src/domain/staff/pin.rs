use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::error::AppError;

pub fn hash_pin(pin: &str) -> Result<(String, String), AppError> {
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    let hash = digest(&salt, pin);
    Ok((hex::encode(salt), hash))
}

pub fn verify_pin(pin: &str, salt_hex: &str, hash_hex: &str) -> bool {
    let Ok(salt) = hex::decode(salt_hex) else {
        return false;
    };
    digest(&salt, pin) == hash_hex
}

fn digest(salt: &[u8], pin: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt);
    hasher.update(pin.as_bytes());
    hex::encode(hasher.finalize())
}
