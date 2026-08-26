use rusqlite::{Connection, OptionalExtension, params};

use crate::domain::customers::types::Customer;
use crate::domain::customers::validation::ValidatedCustomerInput;
use crate::error::AppError;

pub fn insert_customer(
    conn: &Connection,
    input: &ValidatedCustomerInput,
    now: &str,
) -> Result<Customer, AppError> {
    conn.execute(
        "INSERT INTO customers (
            name, phone, email, address, notes, created_at, updated_at, archived_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL)",
        params![
            input.name,
            input.phone,
            input.email,
            input.address,
            input.notes,
            now,
            now
        ],
    )?;
    let id = conn.last_insert_rowid();
    get_customer_by_id(conn, id)?.ok_or(AppError::Internal {
        message: "customer missing after insert".into(),
    })
}

pub fn update_customer(
    conn: &Connection,
    id: i64,
    input: &ValidatedCustomerInput,
    now: &str,
) -> Result<Customer, AppError> {
    let updated = conn.execute(
        "UPDATE customers SET
            name = ?1,
            phone = ?2,
            email = ?3,
            address = ?4,
            notes = ?5,
            updated_at = ?6
         WHERE id = ?7 AND archived_at IS NULL",
        params![
            input.name,
            input.phone,
            input.email,
            input.address,
            input.notes,
            now,
            id
        ],
    )?;
    if updated == 0 {
        // Distinguish missing vs already archived
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

pub fn get_customer_by_id(conn: &Connection, id: i64) -> Result<Option<Customer>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, phone, email, address, notes, created_at, updated_at, archived_at
         FROM customers WHERE id = ?1",
    )?;
    let customer = stmt
        .query_row(params![id], map_customer)
        .optional()?;
    Ok(customer)
}

pub fn set_archived_at(
    conn: &Connection,
    id: i64,
    archived_at: Option<&str>,
    now: &str,
) -> Result<Customer, AppError> {
    let updated = conn.execute(
        "UPDATE customers SET archived_at = ?1, updated_at = ?2 WHERE id = ?3",
        params![archived_at, now, id],
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
    let pattern = search
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            let escaped = s
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_");
            format!("%{escaped}%")
        });

    let total: i64 = match (&pattern, include_archived) {
        (None, false) => conn.query_row(
            "SELECT COUNT(*) FROM customers WHERE archived_at IS NULL",
            [],
            |row| row.get(0),
        )?,
        (None, true) => conn.query_row("SELECT COUNT(*) FROM customers", [], |row| row.get(0))?,
        (Some(p), false) => conn.query_row(
            "SELECT COUNT(*) FROM customers
             WHERE archived_at IS NULL
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
             WHERE name LIKE ?1 ESCAPE '\\'
                OR IFNULL(phone, '') LIKE ?1 ESCAPE '\\'
                OR IFNULL(email, '') LIKE ?1 ESCAPE '\\'",
            params![p],
            |row| row.get(0),
        )?,
    };

    let sql = match (&pattern, include_archived) {
        (None, false) => {
            "SELECT id, name, phone, email, address, notes, created_at, updated_at, archived_at
             FROM customers
             WHERE archived_at IS NULL
             ORDER BY name COLLATE NOCASE ASC, id ASC
             LIMIT ?1 OFFSET ?2"
        }
        (None, true) => {
            "SELECT id, name, phone, email, address, notes, created_at, updated_at, archived_at
             FROM customers
             ORDER BY name COLLATE NOCASE ASC, id ASC
             LIMIT ?1 OFFSET ?2"
        }
        (Some(_), false) => {
            "SELECT id, name, phone, email, address, notes, created_at, updated_at, archived_at
             FROM customers
             WHERE archived_at IS NULL
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
             WHERE name LIKE ?3 ESCAPE '\\'
                OR IFNULL(phone, '') LIKE ?3 ESCAPE '\\'
                OR IFNULL(email, '') LIKE ?3 ESCAPE '\\'
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
