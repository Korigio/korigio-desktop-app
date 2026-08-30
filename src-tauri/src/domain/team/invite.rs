//! Crockford base32 invite codes (`XXXX-XXXX`) hashed with SHA-256.

use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::error::AppError;

const CROCKFORD: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

pub fn generate_invite_code() -> String {
    let mut bytes = [0u8; 8];
    rand::thread_rng().fill_bytes(&mut bytes);
    let mut raw = String::with_capacity(8);
    for b in bytes {
        raw.push(CROCKFORD[(b as usize) % CROCKFORD.len()] as char);
    }
    format!("{}-{}", &raw[..4], &raw[4..])
}

pub fn normalize_invite_code(raw: &str) -> Result<String, AppError> {
    let mut out = String::new();
    for ch in raw.chars() {
        if ch == '-' || ch.is_whitespace() {
            continue;
        }
        let up = ch.to_ascii_uppercase();
        let mapped = match up {
            'I' | 'L' => '1',
            'O' => '0',
            'U' => 'V',
            c if CROCKFORD.contains(&(c as u8)) => c,
            _ => {
                return Err(AppError::Validation {
                    field: Some("inviteCode".into()),
                    message: "Invite code is not valid.".into(),
                });
            }
        };
        out.push(mapped);
    }
    if out.len() != 8 {
        return Err(AppError::Validation {
            field: Some("inviteCode".into()),
            message: "Invite code is not valid.".into(),
        });
    }
    Ok(format!("{}-{}", &out[..4], &out[4..]))
}

pub fn hash_invite_code(canonical: &str) -> String {
    hex::encode(Sha256::digest(canonical.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_and_hash() {
        let code = generate_invite_code();
        assert_eq!(code.len(), 9);
        let again = normalize_invite_code(&code.to_ascii_lowercase()).expect("ok");
        assert_eq!(again, code);
        assert_eq!(hash_invite_code(&again).len(), 64);
    }
}
