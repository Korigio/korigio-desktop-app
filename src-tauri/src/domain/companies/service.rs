use std::fs;
use std::path::Path;

use rusqlite::Connection;
use uuid::Uuid;

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
use crate::error::AppError;

pub fn create_company(conn: &Connection, input: CompanyInput) -> Result<Company, AppError> {
    let validated = validate_company_input(&input)?;
    let now = now_utc_rfc3339()?;
    let is_first = repository::count_companies(conn)? == 0;
    repository::insert_company(conn, &validated, is_first, &now)
}

pub fn update_company(
    conn: &Connection,
    id: i64,
    input: CompanyInput,
) -> Result<Company, AppError> {
    let validated = validate_company_input(&input)?;
    let now = now_utc_rfc3339()?;
    repository::update_company(conn, id, &validated, &now)
}

pub fn get_company(conn: &Connection, id: i64) -> Result<Company, AppError> {
    repository::get_company_by_id(conn, id)?.ok_or(AppError::NotFound)
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

pub fn archive_company(conn: &Connection, id: i64) -> Result<Company, AppError> {
    let existing = get_company(conn, id)?;
    if existing.archived_at.is_some() {
        return Ok(existing);
    }
    let now = now_utc_rfc3339()?;
    // Archiving the default company clears is_default (does not block archive).
    repository::set_archived_at(conn, id, Some(&now), existing.is_default, &now)
}

pub fn unarchive_company(conn: &Connection, id: i64) -> Result<Company, AppError> {
    let existing = get_company(conn, id)?;
    if existing.archived_at.is_none() {
        return Ok(existing);
    }
    let now = now_utc_rfc3339()?;
    repository::set_archived_at(conn, id, None, false, &now)
}

pub fn set_default_company(conn: &Connection, id: i64) -> Result<Company, AppError> {
    let _ = get_company(conn, id)?;
    let now = now_utc_rfc3339()?;
    let tx = conn.unchecked_transaction()?;
    let company = repository::set_default_in_tx(&tx, id, &now)?;
    tx.commit()?;
    Ok(company)
}

pub fn attach_company_logo(
    db: &Db,
    input: AttachCompanyLogoInput,
) -> Result<Company, AppError> {
    let validated = validate_attach_logo_input(&input)?;
    let existing = get_company(db.conn(), validated.company_id)?;
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
    let file_stem = Uuid::new_v4().to_string();
    let relative = format!(
        "images/companies/{}/{file_stem}.{ext}",
        validated.company_id
    );
    let absolute = db.paths().root.join(&relative);

    if let Some(parent) = absolute.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&validated.source_path, &absolute)?;

    let now = now_utc_rfc3339()?;
    let updated = repository::set_logo_path(db.conn(), validated.company_id, Some(&relative), &now)?;

    if let Some(old) = existing.logo_path {
        if old != relative {
            remove_relative_logo(db.paths().root.as_path(), &old);
        }
    }

    Ok(updated)
}

pub fn clear_company_logo(db: &Db, id: i64) -> Result<Company, AppError> {
    let existing = get_company(db.conn(), id)?;
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
    let updated = repository::set_logo_path(db.conn(), id, None, &now)?;
    if let Some(old) = existing.logo_path {
        remove_relative_logo(db.paths().root.as_path(), &old);
    }
    Ok(updated)
}

pub fn resolve_company_logo_path(
    db: &Db,
    id: i64,
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
