use rusqlite::{params, Connection, OptionalExtension};

use crate::db::repository::like_pattern;
use crate::domain::customers::types::Customer;
use crate::domain::customers::validation::ValidatedCustomerInput;
use crate::domain::sync::WriteContext;
use crate::error::AppError;

pub fn insert_customer(
    conn: &Connection,
    id: &str,
    input: &ValidatedCustomerInput,
    now: &str,
    ctx: &WriteContext,
) -> Result<Customer, AppError> {
    conn.execute(
        "INSERT INTO customers (
            id, name, phone, email, address, notes, created_at, updated_at, archived_at,
            hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, ?9, ?10, ?11, ?12, NULL)",
        params![
            id,
            input.name,
            input.phone,
            input.email,
            input.address,
            input.notes,
            now,
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id
        ],
    )?;
    get_customer_by_id(conn, id)?.ok_or(AppError::Internal {
        message: "customer missing after insert".into(),
    })
}

pub fn update_customer(
    conn: &Connection,
    id: &str,
    input: &ValidatedCustomerInput,
    now: &str,
    ctx: &WriteContext,
) -> Result<Customer, AppError> {
    let updated = conn.execute(
        "UPDATE customers SET
            name = ?1, phone = ?2, email = ?3, address = ?4, notes = ?5, updated_at = ?6,
            hlc_wall_ms = ?7, hlc_counter = ?8, origin_device_id = ?9, updated_by_staff_id = ?10
         WHERE id = ?11 AND archived_at IS NULL AND deleted_at IS NULL",
        params![
            input.name,
            input.phone,
            input.email,
            input.address,
            input.notes,
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id,
            id
        ],
    )?;
    if updated == 0 {
        return match get_customer_by_id(conn, id)? {
            Some(_) => Err(AppError::Validation {
                field: None,
                message: "Archived customers cannot be edited. Unarchive first.".into(),
            }),
            None => Err(AppError::NotFound),
        };
    }
    get_customer_by_id(conn, id)?.ok_or(AppError::NotFound)
}

pub fn get_customer_by_id(conn: &Connection, id: &str) -> Result<Option<Customer>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, phone, email, address, notes, created_at, updated_at, archived_at
         FROM customers WHERE id = ?1 AND deleted_at IS NULL",
    )?;
    let customer = stmt.query_row(params![id], map_customer).optional()?;
    Ok(customer)
}

pub fn set_archived_at(
    conn: &Connection,
    id: &str,
    archived_at: Option<&str>,
    now: &str,
    ctx: &WriteContext,
) -> Result<Customer, AppError> {
    let updated = conn.execute(
        "UPDATE customers SET archived_at = ?1, updated_at = ?2,
            hlc_wall_ms = ?3, hlc_counter = ?4, origin_device_id = ?5, updated_by_staff_id = ?6
         WHERE id = ?7 AND deleted_at IS NULL",
        params![
            archived_at,
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id,
            id
        ],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound);
    }
    get_customer_by_id(conn, id)?.ok_or(AppError::NotFound)
}

pub fn list_customers(
    conn: &Connection,
    search: Option<&str>,
    include_archived: bool,
    limit: u32,
    offset: u32,
) -> Result<(Vec<Customer>, i64), AppError> {
    let pattern = like_pattern(search);

    let total: i64 = match (&pattern, include_archived) {
        (None, false) => conn.query_row(
            "SELECT COUNT(*) FROM customers WHERE archived_at IS NULL AND deleted_at IS NULL",
            [],
            |row| row.get(0),
        )?,
        (None, true) => conn.query_row(
            "SELECT COUNT(*) FROM customers WHERE deleted_at IS NULL",
            [],
            |row| row.get(0),
        )?,
        (Some(p), false) => conn.query_row(
            "SELECT COUNT(*) FROM customers
             WHERE archived_at IS NULL AND deleted_at IS NULL
               AND (
                 name LIKE ?1 ESCAPE '\\'
                 OR IFNULL(phone, '') LIKE ?1 ESCAPE '\\'
                 OR IFNULL(email, '') LIKE ?1 ESCAPE '\\'
               )",
            params![p],
            |row| row.get(0),
        )?,
        (Some(p), true) => conn.query_row(
            "SELECT COUNT(*) FROM customers
             WHERE deleted_at IS NULL
               AND (name LIKE ?1 ESCAPE '\\'
                OR IFNULL(phone, '') LIKE ?1 ESCAPE '\\'
                OR IFNULL(email, '') LIKE ?1 ESCAPE '\\')",
            params![p],
            |row| row.get(0),
        )?,
    };

    let sql = match (&pattern, include_archived) {
        (None, false) => {
            "SELECT id, name, phone, email, address, notes, created_at, updated_at, archived_at
             FROM customers
             WHERE archived_at IS NULL AND deleted_at IS NULL
             ORDER BY name COLLATE NOCASE ASC, id ASC
             LIMIT ?1 OFFSET ?2"
        }
        (None, true) => {
            "SELECT id, name, phone, email, address, notes, created_at, updated_at, archived_at
             FROM customers
             WHERE deleted_at IS NULL
             ORDER BY name COLLATE NOCASE ASC, id ASC
             LIMIT ?1 OFFSET ?2"
        }
        (Some(_), false) => {
            "SELECT id, name, phone, email, address, notes, created_at, updated_at, archived_at
             FROM customers
             WHERE archived_at IS NULL AND deleted_at IS NULL
               AND (
                 name LIKE ?3 ESCAPE '\\'
                 OR IFNULL(phone, '') LIKE ?3 ESCAPE '\\'
                 OR IFNULL(email, '') LIKE ?3 ESCAPE '\\'
               )
             ORDER BY name COLLATE NOCASE ASC, id ASC
             LIMIT ?1 OFFSET ?2"
        }
        (Some(_), true) => {
            "SELECT id, name, phone, email, address, notes, created_at, updated_at, archived_at
             FROM customers
             WHERE deleted_at IS NULL
               AND (name LIKE ?3 ESCAPE '\\'
                OR IFNULL(phone, '') LIKE ?3 ESCAPE '\\'
                OR IFNULL(email, '') LIKE ?3 ESCAPE '\\')
             ORDER BY name COLLATE NOCASE ASC, id ASC
             LIMIT ?1 OFFSET ?2"
        }
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = match &pattern {
        None => stmt
            .query_map(params![limit, offset], map_customer)?
            .collect::<Result<Vec<_>, _>>()?,
        Some(p) => stmt
            .query_map(params![limit, offset, p], map_customer)?
            .collect::<Result<Vec<_>, _>>()?,
    };

    Ok((rows, total))
}

fn map_customer(row: &rusqlite::Row<'_>) -> rusqlite::Result<Customer> {
    Ok(Customer {
        id: row.get(0)?,
        name: row.get(1)?,
        phone: row.get(2)?,
        email: row.get(3)?,
        address: row.get(4)?,
        notes: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        archived_at: row.get(8)?,
    })
}
