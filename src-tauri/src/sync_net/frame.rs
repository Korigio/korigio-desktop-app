//! Encrypted TCP frames: `u32_be length | nonce_24 | ciphertext`.

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use rand::RngCore;

use crate::error::AppError;

pub const NONCE_LEN: usize = 24;
pub const MAX_PLAINTEXT: usize = 256 * 1024 + 1024;

pub fn encode_frame(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, AppError> {
    if plaintext.len() > MAX_PLAINTEXT {
        return Err(AppError::sync_err("Frame is too large."));
    }
    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| AppError::Internal {
        message: "invalid aead key".into(),
    })?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| AppError::sync_err("Could not encrypt a network frame."))?;
    let len = (NONCE_LEN + ciphertext.len()) as u32;
    let mut out = Vec::with_capacity(4 + NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

pub fn decode_frame(key: &[u8; 32], frame: &[u8]) -> Result<Vec<u8>, AppError> {
    if frame.len() < 4 + NONCE_LEN + 16 {
        return Err(AppError::sync_err("Network frame is truncated."));
    }
    let len = u32::from_be_bytes([frame[0], frame[1], frame[2], frame[3]]) as usize;
    if frame.len() != 4 + len {
        return Err(AppError::sync_err("Network frame length mismatch."));
    }
    let nonce = XNonce::from_slice(&frame[4..4 + NONCE_LEN]);
    let ciphertext = &frame[4 + NONCE_LEN..];
    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| AppError::Internal {
        message: "invalid aead key".into(),
    })?;
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| AppError::sync_err("Could not decrypt a network frame."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_roundtrip() {
        let key = [7u8; 32];
        let data = b"servioo-sync-frame";
        let encoded = encode_frame(&key, data).expect("encode");
        let decoded = decode_frame(&key, &encoded).expect("decode");
        assert_eq!(decoded, data);
    }

    #[test]
    fn wrong_key_fails() {
        let key = [1u8; 32];
        let other = [2u8; 32];
        let encoded = encode_frame(&key, b"secret").expect("encode");
        assert!(decode_frame(&other, &encoded).is_err());
    }
}
