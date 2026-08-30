//! Content-addressed blob store under `{appData}/blobs/{hash[0:2]}/{hash}`.

use std::fs;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};

use crate::domain::sync::record::{begin_write, record_upsert};
use crate::error::AppError;

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub fn blob_relative_path(hash: &str) -> String {
    let prefix = hash.get(..2).unwrap_or("00");
    format!("blobs/{prefix}/{hash}")
}

pub fn write_blob_bytes(
    root: &Path,
    conn: &Connection,
    bytes: &[u8],
    kind: &str,
    created_at: &str,
) -> Result<String, AppError> {
    let hash = sha256_hex(bytes);
    let relative = blob_relative_path(&hash);
    let absolute = root.join(&relative);
    if let Some(parent) = absolute.parent() {
        fs::create_dir_all(parent)?;
    }
    if !absolute.exists() {
        fs::write(&absolute, bytes)?;
    }
    upsert_blob_row(conn, &hash, bytes.len() as i64, kind, created_at)?;
    Ok(hash)
}

pub fn record_local_blob(
    conn: &Connection,
    hash: &str,
    kind: &str,
    byte_size: i64,
    created_at: &str,
) -> Result<(), AppError> {
    let ctx = begin_write(conn)?;
    record_upsert(
        conn,
        "content_blobs",
        hash,
        serde_json::json!({
            "id": hash,
            "contentHash": hash,
            "kind": kind,
            "byteSize": byte_size,
            "createdAt": created_at,
        }),
        &ctx,
    )?;
    Ok(())
}

pub fn copy_to_display_path(
    root: &Path,
    hash: &str,
    display_relative: &str,
) -> Result<(), AppError> {
    let blob = root.join(blob_relative_path(hash));
    let dest = root.join(display_relative);
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    if dest.exists() {
        return Ok(());
    }
    fs::copy(&blob, &dest)?;
    Ok(())
}

pub fn upsert_blob_row(
    conn: &Connection,
    hash: &str,
    byte_size: i64,
    kind: &str,
    created_at: &str,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO content_blobs (content_hash, byte_size, kind, created_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(content_hash) DO UPDATE SET
            byte_size = excluded.byte_size,
            kind = excluded.kind",
        params![hash, byte_size, kind, created_at],
    )?;
    Ok(())
}

pub fn blob_exists(conn: &Connection, hash: &str) -> Result<bool, AppError> {
    let found: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM content_blobs WHERE content_hash = ?1",
            params![hash],
            |row| row.get(0),
        )
        .optional()?;
    Ok(found.is_some())
}

pub fn blob_absolute(root: &Path, hash: &str) -> PathBuf {
    root.join(blob_relative_path(hash))
}

pub fn blob_kind(conn: &Connection, hash: &str) -> Result<Option<String>, AppError> {
    let kind = conn
        .query_row(
            "SELECT kind FROM content_blobs WHERE content_hash = ?1",
            params![hash],
            |row| row.get(0),
        )
        .optional()?;
    Ok(kind)
}

pub fn infer_blob_kind(conn: &Connection, hash: &str) -> Result<Option<String>, AppError> {
    if let Some(kind) = blob_kind(conn, hash)? {
        return Ok(Some(kind));
    }
    let image: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM repair_images WHERE content_hash = ?1 LIMIT 1",
            params![hash],
            |row| row.get(0),
        )
        .optional()?;
    if image.is_some() {
        return Ok(Some("image".into()));
    }
    let document: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM repair_documents WHERE content_hash = ?1 LIMIT 1",
            params![hash],
            |row| row.get(0),
        )
        .optional()?;
    if document.is_some() {
        return Ok(Some("document".into()));
    }
    let logo: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM companies WHERE logo_content_hash = ?1 LIMIT 1",
            params![hash],
            |row| row.get(0),
        )
        .optional()?;
    if logo.is_some() {
        return Ok(Some("logo".into()));
    }
    Ok(None)
}

pub fn accept_blob_chunk(
    root: &Path,
    hash: &str,
    offset: u64,
    data: &[u8],
    eof: bool,
) -> Result<Option<Vec<u8>>, AppError> {
    if hash.len() < 2 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::sync_err("Blob hash is invalid."));
    }
    let part = root.join(format!("{}.part", blob_relative_path(hash)));
    if let Some(parent) = part.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&part)?;
    file.seek(SeekFrom::Start(offset))?;
    file.write_all(data)?;
    if !eof {
        return Ok(None);
    }
    drop(file);
    let bytes = fs::read(&part)?;
    let computed = sha256_hex(&bytes);
    if computed != hash {
        let _ = fs::remove_file(&part);
        return Err(AppError::sync_err("Blob hash did not match."));
    }
    let dest = root.join(blob_relative_path(hash));
    if dest.exists() {
        let _ = fs::remove_file(&part);
        return Ok(Some(bytes));
    }
    fs::rename(&part, &dest).or_else(|_| {
        fs::copy(&part, &dest)?;
        fs::remove_file(&part)?;
        Ok::<(), AppError>(())
    })?;
    Ok(Some(bytes))
}

pub fn finalize_received_blob(
    root: &Path,
    conn: &Connection,
    hash: &str,
    bytes: &[u8],
) -> Result<(), AppError> {
    let kind = infer_blob_kind(conn, hash)?.unwrap_or_else(|| "document".into());
    let now = crate::db::repository::now_utc_rfc3339()?;
    let dest = root.join(blob_relative_path(hash));
    if !dest.exists() {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&dest, bytes)?;
    }
    upsert_blob_row(conn, hash, bytes.len() as i64, &kind, &now)?;
    copy_to_known_display_paths(root, conn, hash)?;
    Ok(())
}

pub fn copy_to_known_display_paths(
    root: &Path,
    conn: &Connection,
    hash: &str,
) -> Result<(), AppError> {
    let mut paths: Vec<String> = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT original_path FROM repair_images
         WHERE content_hash = ?1 AND deleted_at IS NULL",
    )?;
    let image_paths = stmt
        .query_map(params![hash], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    paths.extend(image_paths);

    let mut stmt = conn.prepare(
        "SELECT file_path FROM repair_documents
         WHERE content_hash = ?1 AND deleted_at IS NULL",
    )?;
    let doc_paths = stmt
        .query_map(params![hash], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    paths.extend(doc_paths);

    let mut stmt = conn.prepare(
        "SELECT logo_path FROM companies
         WHERE logo_content_hash = ?1 AND deleted_at IS NULL AND logo_path IS NOT NULL",
    )?;
    let logo_paths = stmt
        .query_map(params![hash], |row| row.get::<_, Option<String>>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    paths.extend(logo_paths.into_iter().flatten());

    for relative in paths {
        copy_to_display_path(root, hash, &relative)?;
    }
    Ok(())
}
