use hkdf::Hkdf;
use sha2::{Digest, Sha256};

use crate::error::AppError;

pub const LAN_SALT: &[u8] = b"servioo-lan-v1";
pub const JOIN_SALT: &[u8] = b"servioo-join-v1";
pub const HKDF_INFO: &[u8] = b"tcp-aead";
pub const PROTO: &str = "servioo-sync/1";

pub fn derive_aead_key(ikm: &[u8], salt: &[u8]) -> Result<[u8; 32], AppError> {
    let hk = Hkdf::<Sha256>::new(Some(salt), ikm);
    let mut okm = [0u8; 32];
    hk.expand(HKDF_INFO, &mut okm)
        .map_err(|_| AppError::Internal {
            message: "hkdf expand failed".into(),
        })?;
    Ok(okm)
}

pub fn team_key_from_psk_hex(psk_hex: &str) -> Result<[u8; 32], AppError> {
    let bytes = hex::decode(psk_hex).map_err(|_| AppError::Internal {
        message: "invalid team key".into(),
    })?;
    derive_aead_key(&bytes, LAN_SALT)
}

pub fn join_key_from_pin(pin: &str) -> Result<[u8; 32], AppError> {
    derive_aead_key(pin.as_bytes(), JOIN_SALT)
}

pub fn join_key_from_code(canonical_code: &str) -> Result<[u8; 32], AppError> {
    join_key_from_pin(canonical_code)
}

pub fn auth_mac(psk: &[u8], peer_nonce: &[u8], device_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(psk);
    hasher.update(peer_nonce);
    hasher.update(device_id.as_bytes());
    hex::encode(hasher.finalize())
}
