//! Signed repair documents (entrance, diagnosis, summary).

use std::fs;
use std::path::{Component, Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};

use crate::db::repository::now_utc_rfc3339;
use crate::db::Db;
use crate::domain::repairs::repository;
use crate::domain::repairs::types::{Repair, RepairDocument, RepairDocumentType};
use crate::domain::repairs::validation::validate_repair_document_source_path;
use crate::error::AppError;
use crate::paths::AppPaths;

pub fn list_repair_documents(
    conn: &Connection,
    repair_id: String,
) -> Result<Vec<RepairDocument>, AppError> {
    let repair_id = crate::domain::ids::parse_entity_id_field(&repair_id, "repairId")?;
    let _ = repository::get_repair_by_id(conn, &repair_id)?.ok_or(AppError::NotFound)?;

    let mut stmt = conn.prepare(
        "SELECT repair_id, document_type, original_filename, created_at, updated_at
         FROM repair_documents
         WHERE repair_id = ?1 AND deleted_at IS NULL
         ORDER BY document_type",
    )?;
    let rows = stmt
        .query_map(params![repair_id], map_repair_document)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn upload_repair_document(
    db: &Db,
    repair_id: String,
    document_type: RepairDocumentType,
    source_path: String,
) -> Result<RepairDocument, AppError> {
    let repair_id = crate::domain::ids::parse_entity_id_field(&repair_id, "repairId")?;
    let type_slug = document_type.as_slug();
    let source = validate_repair_document_source_path(&source_path)?;
    let existing_repair =
        repository::get_repair_by_id(db.conn(), &repair_id)?.ok_or(AppError::NotFound)?;
    ensure_repair_documents_editable(&existing_repair)?;

    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_else(|| "pdf".into());
    let bytes = fs::read(&source)?;
    let now = now_utc_rfc3339()?;
    let hash = crate::domain::sync::blobs::write_blob_bytes(
        &db.paths().root,
        db.conn(),
        &bytes,
        "document",
        &now,
    )?;
    crate::domain::sync::blobs::record_local_blob(
        db.conn(),
        &hash,
        "document",
        bytes.len() as i64,
        &now,
    )?;
    let relative = format!("documents/repairs/{repair_id}/{hash}.{ext}");
    crate::domain::sync::blobs::copy_to_display_path(&db.paths().root, &hash, &relative)?;

    let filename = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("document")
        .to_string();

    let previous = get_document_row(db.conn(), &repair_id, type_slug)?;
    let document = upsert_document_row(
        db.conn(),
        &repair_id,
        type_slug,
        &relative,
        &filename,
        &hash,
        previous
            .as_ref()
            .map(|d| d.created_at.as_str())
            .unwrap_or(&now),
        &now,
    )?;

    if let Some(old) = previous {
        if old.file_path != relative {
            remove_relative_document(db.paths(), &old.file_path);
        }
    }

    Ok(document)
}

pub fn delete_repair_document(
    db: &Db,
    repair_id: String,
    document_type: RepairDocumentType,
) -> Result<(), AppError> {
    let repair_id = crate::domain::ids::parse_entity_id_field(&repair_id, "repairId")?;
    let existing_repair =
        repository::get_repair_by_id(db.conn(), &repair_id)?.ok_or(AppError::NotFound)?;
    ensure_repair_documents_editable(&existing_repair)?;

    let type_slug = document_type.as_slug();
    let row = get_document_row(db.conn(), &repair_id, type_slug)?.ok_or(AppError::NotFound)?;
    let ctx = crate::domain::sync::begin_write(db.conn())?;
    let now = now_utc_rfc3339()?;

    let deleted = db.conn().execute(
        "UPDATE repair_documents SET deleted_at = ?1, updated_at = ?1,
            hlc_wall_ms = ?2, hlc_counter = ?3, origin_device_id = ?4, updated_by_staff_id = ?5
         WHERE repair_id = ?6 AND document_type = ?7 AND deleted_at IS NULL",
        params![
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id,
            repair_id,
            type_slug
        ],
    )?;
    if deleted == 0 {
        return Err(AppError::NotFound);
    }
    crate::domain::sync::record_delete(
        db.conn(),
        "repair_documents",
        &row.id,
        serde_json::json!({ "id": row.id }),
        &ctx,
    )?;

    remove_relative_document(db.paths(), &row.file_path);
    Ok(())
}

pub fn open_repair_document(
    db: &Db,
    repair_id: String,
    document_type: RepairDocumentType,
) -> Result<(), AppError> {
    let absolute = resolve_repair_document_absolute(db, repair_id, document_type)?;
    open_document_with_shell(&absolute)
}

pub fn resolve_repair_document_absolute(
    db: &Db,
    repair_id: String,
    document_type: RepairDocumentType,
) -> Result<PathBuf, AppError> {
    let repair_id = crate::domain::ids::parse_entity_id_field(&repair_id, "repairId")?;
    let type_slug = document_type.as_slug();
    let row = get_document_row(db.conn(), &repair_id, type_slug)?.ok_or(AppError::NotFound)?;
    validate_document_relative(&row.file_path)?;

    let display = db.paths().root.join(&row.file_path);
    if display.is_file() {
        return resolve_document_path(db.paths(), &row.file_path);
    }

    if !row.content_hash.is_empty() {
        let blob = crate::domain::sync::blobs::blob_absolute(&db.paths().root, &row.content_hash);
        if blob.is_file() {
            crate::domain::sync::blobs::copy_to_display_path(
                &db.paths().root,
                &row.content_hash,
                &row.file_path,
            )?;
            return resolve_document_path(db.paths(), &row.file_path);
        }
    }

    Err(AppError::Validation {
        field: Some("path".into()),
        message: "Document file was not found.".into(),
    })
}

fn ensure_repair_documents_editable(repair: &Repair) -> Result<(), AppError> {
    if repair.archived_at.is_some() {
        return Err(AppError::Validation {
            field: None,
            message: "Archived repairs cannot be edited.".into(),
        });
    }
    if repair.status == "cancelled" {
        return Err(AppError::Validation {
            field: None,
            message: "Cancelled repairs cannot be edited.".into(),
        });
    }
    Ok(())
}

struct DocumentRow {
    id: String,
    file_path: String,
    content_hash: String,
    created_at: String,
}

fn get_document_row(
    conn: &Connection,
    repair_id: &str,
    document_type: &str,
) -> Result<Option<DocumentRow>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, file_path, content_hash, created_at FROM repair_documents
         WHERE repair_id = ?1 AND document_type = ?2 AND deleted_at IS NULL",
    )?;
    let row = stmt
        .query_row(params![repair_id, document_type], |row| {
            Ok(DocumentRow {
                id: row.get(0)?,
                file_path: row.get(1)?,
                content_hash: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .optional()?;
    Ok(row)
}

fn upsert_document_row(
    conn: &Connection,
    repair_id: &str,
    document_type: &str,
    file_path: &str,
    original_filename: &str,
    content_hash: &str,
    created_at: &str,
    updated_at: &str,
) -> Result<RepairDocument, AppError> {
    let id = crate::domain::ids::new_entity_id();
    let ctx = crate::domain::sync::begin_write(conn)?;
    conn.execute(
        "INSERT INTO repair_documents (
            id, repair_id, document_type, file_path, original_filename, content_hash,
            created_at, updated_at, hlc_wall_ms, hlc_counter, origin_device_id,
            updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, NULL)
         ON CONFLICT(repair_id, document_type) DO UPDATE SET
            file_path = excluded.file_path,
            original_filename = excluded.original_filename,
            content_hash = excluded.content_hash,
            updated_at = excluded.updated_at,
            hlc_wall_ms = excluded.hlc_wall_ms,
            hlc_counter = excluded.hlc_counter,
            origin_device_id = excluded.origin_device_id,
            updated_by_staff_id = excluded.updated_by_staff_id,
            deleted_at = NULL",
        params![
            id,
            repair_id,
            document_type,
            file_path,
            original_filename,
            content_hash,
            created_at,
            updated_at,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id,
        ],
    )?;
    let doc = get_document_public(conn, repair_id, document_type)?.ok_or(AppError::Internal {
        message: "document missing after upsert".into(),
    })?;
    let stored_id: String = conn.query_row(
        "SELECT id FROM repair_documents WHERE repair_id = ?1 AND document_type = ?2",
        params![repair_id, document_type],
        |row| row.get(0),
    )?;
    crate::domain::sync::record_upsert(
        conn,
        "repair_documents",
        &stored_id,
        serde_json::json!({
            "id": stored_id,
            "repairId": repair_id,
            "documentType": document_type,
            "filePath": file_path,
            "originalFilename": original_filename,
            "contentHash": content_hash,
            "createdAt": created_at,
            "updatedAt": updated_at,
            "updatedByStaffId": ctx.staff_id,
            "deletedAt": serde_json::Value::Null,
        }),
        &ctx,
    )?;
    Ok(doc)
}

fn get_document_public(
    conn: &Connection,
    repair_id: &str,
    document_type: &str,
) -> Result<Option<RepairDocument>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT repair_id, document_type, original_filename, created_at, updated_at
         FROM repair_documents
         WHERE repair_id = ?1 AND document_type = ?2 AND deleted_at IS NULL",
    )?;
    let row = stmt
        .query_row(params![repair_id, document_type], map_repair_document)
        .optional()?;
    Ok(row)
}

fn map_repair_document(row: &rusqlite::Row<'_>) -> rusqlite::Result<RepairDocument> {
    let slug: String = row.get(1)?;
    let document_type = RepairDocumentType::from_slug(&slug)
        .map(|t| t.to_camel().to_string())
        .unwrap_or(slug);
    Ok(RepairDocument {
        repair_id: row.get(0)?,
        document_type,
        original_filename: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

fn validate_document_relative(relative: &str) -> Result<(), AppError> {
    if relative.is_empty()
        || Path::new(relative).is_absolute()
        || has_parent_dir_component(relative)
    {
        return Err(AppError::Validation {
            field: Some("path".into()),
            message: "Document path is invalid.".into(),
        });
    }

    let prefix = "documents/repairs/";
    if !relative.starts_with(prefix) && !relative.starts_with("documents\\repairs\\") {
        return Err(AppError::Validation {
            field: Some("path".into()),
            message: "Document path is invalid.".into(),
        });
    }
    Ok(())
}

pub(crate) fn resolve_document_path(paths: &AppPaths, relative: &str) -> Result<PathBuf, AppError> {
    validate_document_relative(relative)?;

    let candidate = paths.root.join(relative);
    if !candidate.is_file() {
        return Err(AppError::Validation {
            field: Some("path".into()),
            message: "Document file was not found.".into(),
        });
    }
    let canonical = candidate.canonicalize().map_err(|_| AppError::Validation {
        field: Some("path".into()),
        message: "Document file was not found.".into(),
    })?;

    let documents_root = paths.root.join("documents");
    let allowed_root = documents_root
        .canonicalize()
        .map_err(|_| AppError::Validation {
            field: Some("path".into()),
            message: "Document path is invalid.".into(),
        })?;
    if !canonical.starts_with(&allowed_root) {
        return Err(AppError::Validation {
            field: Some("path".into()),
            message: "Document path is invalid.".into(),
        });
    }

    Ok(canonical)
}

pub(crate) fn remove_relative_document(paths: &AppPaths, relative: &str) {
    let absolute = paths.root.join(relative);
    match fs::remove_file(&absolute) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            eprintln!("failed to remove repair document file err={err}");
        }
    }
}

fn has_parent_dir_component(relative: &str) -> bool {
    Path::new(relative)
        .components()
        .any(|c| matches!(c, Component::ParentDir))
}

#[cfg(windows)]
fn open_document_with_shell(path: &Path) -> Result<(), AppError> {
    use std::process::Command;

    let path_str = path.to_string_lossy().into_owned();
    Command::new("cmd")
        .args(["/C", "start", "", &path_str])
        .spawn()
        .map_err(|err| AppError::Internal {
            message: format!("failed to open document: {err}"),
        })?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn open_document_with_shell(path: &Path) -> Result<(), AppError> {
    use std::process::Command;

    Command::new("open")
        .arg(path)
        .spawn()
        .map_err(|err| AppError::Internal {
            message: format!("failed to open document: {err}"),
        })?;
    Ok(())
}

#[cfg(all(not(windows), not(target_os = "macos")))]
fn open_document_with_shell(_path: &Path) -> Result<(), AppError> {
    Err(AppError::Internal {
        message: "Opening documents is only supported on Windows and macOS.".into(),
    })
}
