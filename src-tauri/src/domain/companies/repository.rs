use rusqlite::{params, Connection, OptionalExtension};

use crate::db::repository::like_pattern;
use crate::domain::companies::types::Company;
use crate::domain::companies::validation::ValidatedCompanyInput;
use crate::domain::sync::WriteContext;
use crate::error::AppError;

pub fn insert_company(
    conn: &Connection,
    id: &str,
    input: &ValidatedCompanyInput,
    is_default: bool,
    now: &str,
    ctx: &WriteContext,
) -> Result<Company, AppError> {
    conn.execute(
        "INSERT INTO companies (
            id, legal_name, trade_name, tax_id, address, phone, email, website,
            logo_path, logo_content_hash, is_default, created_at, updated_at, archived_at,
            hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, NULL, ?9, ?10, ?11, NULL, ?12, ?13, ?14, ?15, NULL)",
        params![
            id,
            input.legal_name,
            input.trade_name,
            input.tax_id,
            input.address,
            input.phone,
            input.email,
            input.website,
            i64::from(is_default),
            now,
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id
        ],
    )?;
    get_company_by_id(conn, id)?.ok_or(AppError::Internal {
        message: "company missing after insert".into(),
    })
}

pub fn update_company(
    conn: &Connection,
    id: &str,
    input: &ValidatedCompanyInput,
    now: &str,
    ctx: &WriteContext,
) -> Result<Company, AppError> {
    let updated = conn.execute(
        "UPDATE companies SET
            legal_name = ?1,
            trade_name = ?2,
            tax_id = ?3,
            address = ?4,
            phone = ?5,
            email = ?6,
            website = ?7,
            updated_at = ?8,
            hlc_wall_ms = ?9, hlc_counter = ?10, origin_device_id = ?11, updated_by_staff_id = ?12
         WHERE id = ?13 AND archived_at IS NULL",
        params![
            input.legal_name,
            input.trade_name,
            input.tax_id,
            input.address,
            input.phone,
            input.email,
            input.website,
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id,
            id
        ],
    )?;
    if updated == 0 {
        return match get_company_by_id(conn, id)? {
            Some(_) => Err(AppError::Validation {
                field: None,
                message: "Archived companies cannot be edited. Unarchive first.".into(),
            }),
            None => Err(AppError::NotFound),
        };
    }
    get_company_by_id(conn, id)?.ok_or(AppError::NotFound)
}

pub fn get_company_by_id(conn: &Connection, id: &str) -> Result<Option<Company>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, legal_name, trade_name, tax_id, address, phone, email, website,
                logo_path, is_default, created_at, updated_at, archived_at
         FROM companies WHERE id = ?1",
    )?;
    let company = stmt.query_row(params![id], map_company).optional()?;
    Ok(company)
}

pub fn count_companies(conn: &Connection) -> Result<i64, AppError> {
    Ok(conn.query_row("SELECT COUNT(*) FROM companies", [], |row| row.get(0))?)
}

pub fn set_archived_at(
    conn: &Connection,
    id: &str,
    archived_at: Option<&str>,
    clear_default: bool,
    now: &str,
    ctx: &WriteContext,
) -> Result<Company, AppError> {
    let updated = if clear_default {
        conn.execute(
            "UPDATE companies SET archived_at = ?1, is_default = 0, updated_at = ?2,
                hlc_wall_ms = ?3, hlc_counter = ?4, origin_device_id = ?5, updated_by_staff_id = ?6
             WHERE id = ?7",
            params![
                archived_at,
                now,
                ctx.hlc.wall,
                ctx.hlc.counter,
                ctx.hlc.origin_device_id,
                ctx.staff_id,
                id
            ],
        )?
    } else {
        conn.execute(
            "UPDATE companies SET archived_at = ?1, updated_at = ?2,
                hlc_wall_ms = ?3, hlc_counter = ?4, origin_device_id = ?5, updated_by_staff_id = ?6
             WHERE id = ?7",
            params![
                archived_at,
                now,
                ctx.hlc.wall,
                ctx.hlc.counter,
                ctx.hlc.origin_device_id,
                ctx.staff_id,
                id
            ],
        )?
    };
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    get_company_by_id(conn, id)?.ok_or(AppError::NotFound)
}

pub fn list_default_company_ids(conn: &Connection) -> Result<Vec<String>, AppError> {
    let mut stmt = conn.prepare("SELECT id FROM companies WHERE is_default = 1")?;
    let rows = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Clear all defaults, then set `id` as the sole default. Caller must hold a transaction.
pub fn set_default_in_tx(
    conn: &Connection,
    id: &str,
    now: &str,
    ctx: &WriteContext,
) -> Result<Company, AppError> {
    conn.execute(
        "UPDATE companies SET is_default = 0, updated_at = ?1,
            hlc_wall_ms = ?2, hlc_counter = ?3, origin_device_id = ?4, updated_by_staff_id = ?5
         WHERE is_default = 1 AND id != ?6",
        params![
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id,
            id
        ],
    )?;
    let updated = conn.execute(
        "UPDATE companies SET is_default = 1, updated_at = ?1,
            hlc_wall_ms = ?2, hlc_counter = ?3, origin_device_id = ?4, updated_by_staff_id = ?5
         WHERE id = ?6 AND archived_at IS NULL",
        params![
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id,
            id
        ],
    )?;
    if updated == 0 {
        return match get_company_by_id(conn, id)? {
            Some(_) => Err(AppError::Validation {
                field: Some("id".into()),
                message: "Archived companies cannot be set as default. Unarchive first.".into(),
            }),
            None => Err(AppError::NotFound),
        };
    }
    get_company_by_id(conn, id)?.ok_or(AppError::NotFound)
}

pub fn set_logo_path(
    conn: &Connection,
    id: &str,
    logo_path: Option<&str>,
    logo_content_hash: Option<&str>,
    now: &str,
) -> Result<Company, AppError> {
    let ctx = crate::domain::sync::begin_write(conn)?;
    let updated = conn.execute(
        "UPDATE companies SET logo_path = ?1, logo_content_hash = ?2, updated_at = ?3,
            hlc_wall_ms = ?4, hlc_counter = ?5, origin_device_id = ?6, updated_by_staff_id = ?7
         WHERE id = ?8 AND archived_at IS NULL",
        params![
            logo_path,
            logo_content_hash,
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id,
            id
        ],
    )?;
    if updated == 0 {
        return match get_company_by_id(conn, id)? {
            Some(_) => Err(AppError::Validation {
                field: None,
                message: "Archived companies cannot be edited. Unarchive first.".into(),
            }),
            None => Err(AppError::NotFound),
        };
    }
    let company = get_company_by_id(conn, id)?.ok_or(AppError::NotFound)?;
    let payload = serde_json::to_value(&company).map_err(|err| AppError::Internal {
        message: format!("serialize company: {err}"),
    })?;
    crate::domain::sync::record_upsert(conn, "companies", id, payload, &ctx)?;
    Ok(company)
}

pub fn list_companies(
    conn: &Connection,
    search: Option<&str>,
    include_archived: bool,
    limit: u32,
    offset: u32,
) -> Result<(Vec<Company>, i64), AppError> {
    let pattern = like_pattern(search);

    let total: i64 = match (&pattern, include_archived) {
        (None, false) => conn.query_row(
            "SELECT COUNT(*) FROM companies WHERE archived_at IS NULL",
            [],
            |row| row.get(0),
        )?,
        (None, true) => conn.query_row("SELECT COUNT(*) FROM companies", [], |row| row.get(0))?,
        (Some(p), false) => conn.query_row(
            "SELECT COUNT(*) FROM companies
             WHERE archived_at IS NULL
               AND (
                 legal_name LIKE ?1 ESCAPE '\\'
                 OR IFNULL(trade_name, '') LIKE ?1 ESCAPE '\\'
                 OR IFNULL(tax_id, '') LIKE ?1 ESCAPE '\\'
               )",
            params![p],
            |row| row.get(0),
        )?,
        (Some(p), true) => conn.query_row(
            "SELECT COUNT(*) FROM companies
             WHERE legal_name LIKE ?1 ESCAPE '\\'
                OR IFNULL(trade_name, '') LIKE ?1 ESCAPE '\\'
                OR IFNULL(tax_id, '') LIKE ?1 ESCAPE '\\'",
            params![p],
            |row| row.get(0),
        )?,
    };

    let sql = match (&pattern, include_archived) {
        (None, false) => {
            "SELECT id, legal_name, trade_name, tax_id, address, phone, email, website,
                    logo_path, is_default, created_at, updated_at, archived_at
             FROM companies
             WHERE archived_at IS NULL
             ORDER BY is_default DESC, legal_name COLLATE NOCASE ASC, id ASC
             LIMIT ?1 OFFSET ?2"
        }
        (None, true) => {
            "SELECT id, legal_name, trade_name, tax_id, address, phone, email, website,
                    logo_path, is_default, created_at, updated_at, archived_at
             FROM companies
             ORDER BY is_default DESC, legal_name COLLATE NOCASE ASC, id ASC
             LIMIT ?1 OFFSET ?2"
        }
        (Some(_), false) => {
            "SELECT id, legal_name, trade_name, tax_id, address, phone, email, website,
                    logo_path, is_default, created_at, updated_at, archived_at
             FROM companies
             WHERE archived_at IS NULL
               AND (
                 legal_name LIKE ?3 ESCAPE '\\'
                 OR IFNULL(trade_name, '') LIKE ?3 ESCAPE '\\'
                 OR IFNULL(tax_id, '') LIKE ?3 ESCAPE '\\'
               )
             ORDER BY is_default DESC, legal_name COLLATE NOCASE ASC, id ASC
             LIMIT ?1 OFFSET ?2"
        }
        (Some(_), true) => {
            "SELECT id, legal_name, trade_name, tax_id, address, phone, email, website,
                    logo_path, is_default, created_at, updated_at, archived_at
             FROM companies
             WHERE legal_name LIKE ?3 ESCAPE '\\'
                OR IFNULL(trade_name, '') LIKE ?3 ESCAPE '\\'
                OR IFNULL(tax_id, '') LIKE ?3 ESCAPE '\\'
             ORDER BY is_default DESC, legal_name COLLATE NOCASE ASC, id ASC
             LIMIT ?1 OFFSET ?2"
        }
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = match &pattern {
        None => stmt
            .query_map(params![limit, offset], map_company)?
            .collect::<Result<Vec<_>, _>>()?,
        Some(p) => stmt
            .query_map(params![limit, offset, p], map_company)?
            .collect::<Result<Vec<_>, _>>()?,
    };

    Ok((rows, total))
}

fn map_company(row: &rusqlite::Row<'_>) -> rusqlite::Result<Company> {
    let is_default_i: i64 = row.get(9)?;
    Ok(Company {
        id: row.get(0)?,
        legal_name: row.get(1)?,
        trade_name: row.get(2)?,
        tax_id: row.get(3)?,
        address: row.get(4)?,
        phone: row.get(5)?,
        email: row.get(6)?,
        website: row.get(7)?,
        logo_path: row.get(8)?,
        is_default: is_default_i != 0,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
        archived_at: row.get(12)?,
    })
}
