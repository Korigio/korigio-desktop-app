use std::fs;
use std::path::Path;

use rusqlite::Connection;

use crate::db::repository::now_utc_rfc3339;
use crate::db::Db;
use crate::domain::companies::constants::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};
use crate::domain::companies::repository;
use crate::domain::companies::types::{
    AttachCompanyLogoInput, Company, CompanyInput, CompanyListQuery, CompanyListResult,
    ResolveCompanyLogoPathResult,
};
use crate::domain::companies::validation::{validate_attach_logo_input, validate_company_input};
use crate::domain::images::service::resolve_safe_absolute;
use crate::domain::sync::{self, begin_write, WriteContext};
use crate::error::AppError;

fn record(conn: &Connection, company: &Company, ctx: &WriteContext) -> Result<(), AppError> {
    let payload = serde_json::to_value(company).map_err(|err| AppError::Internal {
        message: format!("serialize company: {err}"),
    })?;
    sync::record_upsert(conn, "companies", &company.id, payload, ctx)?;
    Ok(())
}

pub fn create_company(conn: &Connection, input: CompanyInput) -> Result<Company, AppError> {
    let validated = validate_company_input(&input)?;
    let now = now_utc_rfc3339()?;
    let is_first = repository::count_companies(conn)? == 0;
    let ctx = begin_write(conn)?;
    let id = crate::domain::ids::new_entity_id();
    let company = repository::insert_company(conn, &id, &validated, is_first, &now, &ctx)?;
    record(conn, &company, &ctx)?;
    Ok(company)
}

pub fn update_company(
    conn: &Connection,
    id: String,
    input: CompanyInput,
) -> Result<Company, AppError> {
    let validated = validate_company_input(&input)?;
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let company = repository::update_company(conn, &id, &validated, &now, &ctx)?;
    record(conn, &company, &ctx)?;
    Ok(company)
}

pub fn get_company(conn: &Connection, id: String) -> Result<Company, AppError> {
    repository::get_company_by_id(conn, &id)?.ok_or(AppError::NotFound)
}

pub fn list_companies(
    conn: &Connection,
    query: CompanyListQuery,
) -> Result<CompanyListResult, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query
        .page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let include_archived = query.include_archived.unwrap_or(false);
    let offset = (page - 1).saturating_mul(page_size);

    let (items, total) = repository::list_companies(
        conn,
        query.query.as_deref(),
        include_archived,
        page_size,
        offset,
    )?;

    Ok(CompanyListResult {
        items,
        total,
        page,
        page_size,
    })
}

pub fn archive_company(conn: &Connection, id: String) -> Result<Company, AppError> {
    let existing = get_company(conn, id)?;
    if existing.archived_at.is_some() {
        return Ok(existing);
    }
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    // Archiving the default company clears is_default (does not block archive).
    let company = repository::set_archived_at(
        conn,
        &existing.id,
        Some(&now),
        existing.is_default,
        &now,
        &ctx,
    )?;
    record(conn, &company, &ctx)?;
    Ok(company)
}

pub fn unarchive_company(conn: &Connection, id: String) -> Result<Company, AppError> {
    let existing = get_company(conn, id)?;
    if existing.archived_at.is_none() {
        return Ok(existing);
    }
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let company = repository::set_archived_at(conn, &existing.id, None, false, &now, &ctx)?;
    record(conn, &company, &ctx)?;
    Ok(company)
}

pub fn set_default_company(conn: &Connection, id: String) -> Result<Company, AppError> {
    let _ = get_company(conn, id.clone())?;
    let ctx = begin_write(conn)?;
    let now = now_utc_rfc3339()?;
    let previous = repository::list_default_company_ids(conn)?;
    let tx = conn.unchecked_transaction()?;
    let company = repository::set_default_in_tx(&tx, &id, &now, &ctx)?;
    for prev_id in previous {
        if prev_id == company.id {
            continue;
        }
        if let Some(prev) = repository::get_company_by_id(&tx, &prev_id)? {
            record(&tx, &prev, &ctx)?;
        }
    }
    record(&tx, &company, &ctx)?;
    tx.commit()?;
    Ok(company)
}

pub fn attach_company_logo(db: &Db, input: AttachCompanyLogoInput) -> Result<Company, AppError> {
    let validated = validate_attach_logo_input(&input)?;
    let company_id = validated.company_id.clone();
    let existing = get_company(db.conn(), company_id.clone())?;
    if existing.archived_at.is_some() {
        return Err(AppError::Validation {
            field: None,
            message: "Archived companies cannot be edited. Unarchive first.".into(),
        });
    }

    let ext = validated
        .source_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_else(|| "jpg".into());
    let now = now_utc_rfc3339()?;
    let bytes = fs::read(&validated.source_path)?;
    let hash = crate::domain::sync::blobs::write_blob_bytes(
        &db.paths().root,
        db.conn(),
        &bytes,
        "logo",
        &now,
    )?;
    crate::domain::sync::blobs::record_local_blob(
        db.conn(),
        &hash,
        "logo",
        bytes.len() as i64,
        &now,
    )?;
    let relative = format!("images/companies/{company_id}/{hash}.{ext}");
    crate::domain::sync::blobs::copy_to_display_path(&db.paths().root, &hash, &relative)?;

    let updated =
        repository::set_logo_path(db.conn(), &company_id, Some(&relative), Some(&hash), &now)?;

    if let Some(old) = existing.logo_path {
        if old != relative {
            remove_relative_logo(db.paths().root.as_path(), &old);
        }
    }

    Ok(updated)
}

pub fn clear_company_logo(db: &Db, id: String) -> Result<Company, AppError> {
    let existing = get_company(db.conn(), id.clone())?;
    if existing.archived_at.is_some() {
        return Err(AppError::Validation {
            field: None,
            message: "Archived companies cannot be edited. Unarchive first.".into(),
        });
    }
    if existing.logo_path.is_none() {
        return Ok(existing);
    }

    let now = now_utc_rfc3339()?;
    let updated = repository::set_logo_path(db.conn(), &id, None, None, &now)?;
    if let Some(old) = existing.logo_path {
        remove_relative_logo(db.paths().root.as_path(), &old);
    }
    Ok(updated)
}

pub fn resolve_company_logo_path(
    db: &Db,
    id: String,
) -> Result<ResolveCompanyLogoPathResult, AppError> {
    let existing = get_company(db.conn(), id)?;
    let relative = existing.logo_path.ok_or(AppError::NotFound)?;
    let absolute = resolve_safe_absolute(db.paths(), &relative)?;
    Ok(ResolveCompanyLogoPathResult {
        absolute_path: absolute.to_string_lossy().into_owned(),
    })
}

fn remove_relative_logo(root: &Path, relative: &str) {
    let absolute = root.join(relative);
    match fs::remove_file(&absolute) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            eprintln!("failed to remove company logo file err={err}");
        }
    }
}
