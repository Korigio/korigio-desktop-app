//! Signed repair documents (entrance, diagnosis, summary).

use std::fs;
use std::path::{Component, Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, params};

use crate::db::repository::now_utc_rfc3339;
use crate::db::Db;
use crate::domain::repairs::repository;
use crate::domain::repairs::types::{Repair, RepairDocument, RepairDocumentType};
use crate::domain::repairs::validation::validate_repair_document_source_path;
use crate::error::AppError;
use crate::paths::AppPaths;

pub fn list_repair_documents(
    conn: &Connection,
    repair_id: i64,
) -> Result<Vec<RepairDocument>, AppError> {
    if repair_id <= 0 {
        return Err(AppError::Validation {
            field: Some("repairId".into()),
            message: "Repair is required.".into(),
        });
    }
    let _ = repository::get_repair_by_id(conn, repair_id)?
        .ok_or(AppError::NotFound)?;

    let mut stmt = conn.prepare(
        "SELECT repair_id, document_type, original_filename, created_at, updated_at
         FROM repair_documents
         WHERE repair_id = ?1
         ORDER BY document_type",
    )?;
    let rows = stmt
        .query_map(params![repair_id], map_repair_document)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn upload_repair_document(
    db: &Db,
    repair_id: i64,
    document_type: RepairDocumentType,
    source_path: String,
) -> Result<RepairDocument, AppError> {
    if repair_id <= 0 {
        return Err(AppError::Validation {
            field: Some("repairId".into()),
            message: "Repair is required.".into(),
        });
    }

    let type_slug = document_type.as_slug();
    let source = validate_repair_document_source_path(&source_path)?;
    let existing_repair = repository::get_repair_by_id(db.conn(), repair_id)?
        .ok_or(AppError::NotFound)?;
    ensure_repair_documents_editable(&existing_repair)?;

    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_else(|| "pdf".into());
    let file_stem = type_slug.replace('_', "-");
    let relative = format!("documents/repairs/{repair_id}/{file_stem}.{ext}");
    let absolute = db.paths().root.join(&relative);

    if let Some(parent) = absolute.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&source, &absolute)?;

    let filename = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&file_stem)
        .to_string();

    let now = now_utc_rfc3339()?;
    let previous = get_document_row(db.conn(), repair_id, type_slug)?;
    let document = upsert_document_row(
        db.conn(),
        repair_id,
        type_slug,
        &relative,
        &filename,
        previous.as_ref().map(|d| d.created_at.as_str()).unwrap_or(&now),
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
    repair_id: i64,
    document_type: RepairDocumentType,
) -> Result<(), AppError> {
    if repair_id <= 0 {
        return Err(AppError::Validation {
            field: Some("repairId".into()),
            message: "Repair is required.".into(),
        });
    }

    let existing_repair = repository::get_repair_by_id(db.conn(), repair_id)?
        .ok_or(AppError::NotFound)?;
    ensure_repair_documents_editable(&existing_repair)?;

    let type_slug = document_type.as_slug();
    let row = get_document_row(db.conn(), repair_id, type_slug)?.ok_or(AppError::NotFound)?;

    let deleted = db.conn().execute(
        "DELETE FROM repair_documents WHERE repair_id = ?1 AND document_type = ?2",
        params![repair_id, type_slug],
    )?;
    if deleted == 0 {
        return Err(AppError::NotFound);
    }

    remove_relative_document(db.paths(), &row.file_path);
    Ok(())
}

pub fn open_repair_document(
    db: &Db,
    repair_id: i64,
    document_type: RepairDocumentType,
) -> Result<(), AppError> {
    let absolute = resolve_repair_document_absolute(db, repair_id, document_type)?;
    open_document_with_shell(&absolute)
}

pub fn resolve_repair_document_absolute(
    db: &Db,
    repair_id: i64,
    document_type: RepairDocumentType,
) -> Result<PathBuf, AppError> {
    let type_slug = document_type.as_slug();
    let row = get_document_row(db.conn(), repair_id, type_slug)?.ok_or(AppError::NotFound)?;
    resolve_document_path(db.paths(), &row.file_path)
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
    file_path: String,
    created_at: String,
}

fn get_document_row(
    conn: &Connection,
    repair_id: i64,
    document_type: &str,
) -> Result<Option<DocumentRow>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT file_path, created_at FROM repair_documents
         WHERE repair_id = ?1 AND document_type = ?2",
    )?;
    let row = stmt
        .query_row(params![repair_id, document_type], |row| {
            Ok(DocumentRow {
                file_path: row.get(0)?,
                created_at: row.get(1)?,
            })
        })
        .optional()?;
    Ok(row)
}

fn upsert_document_row(
    conn: &Connection,
    repair_id: i64,
    document_type: &str,
    file_path: &str,
    original_filename: &str,
    created_at: &str,
    updated_at: &str,
) -> Result<RepairDocument, AppError> {
    conn.execute(
        "INSERT INTO repair_documents (repair_id, document_type, file_path, original_filename, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(repair_id, document_type) DO UPDATE SET
            file_path = excluded.file_path,
            original_filename = excluded.original_filename,
            updated_at = excluded.updated_at",
        params![
            repair_id,
            document_type,
            file_path,
            original_filename,
            created_at,
            updated_at,
        ],
    )?;
    get_document_public(conn, repair_id, document_type)?.ok_or(AppError::Internal {
        message: "document missing after upsert".into(),
    })
}

fn get_document_public(
    conn: &Connection,
    repair_id: i64,
    document_type: &str,
) -> Result<Option<RepairDocument>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT repair_id, document_type, original_filename, created_at, updated_at
         FROM repair_documents
         WHERE repair_id = ?1 AND document_type = ?2",
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

pub(crate) fn resolve_document_path(paths: &AppPaths, relative: &str) -> Result<PathBuf, AppError> {
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

    let candidate = paths.root.join(relative);
    let canonical = candidate.canonicalize().map_err(|_| AppError::Validation {
        field: Some("path".into()),
        message: "Document file was not found.".into(),
    })?;

    let documents_root = paths.root.join("documents");
    let allowed_root = documents_root.canonicalize().map_err(|_| AppError::Validation {
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

#[cfg(not(windows))]
fn open_document_with_shell(_path: &Path) -> Result<(), AppError> {
    Err(AppError::Internal {
        message: "Opening documents is only supported on Windows.".into(),
    })
}