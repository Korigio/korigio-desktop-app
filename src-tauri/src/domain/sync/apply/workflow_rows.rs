use rusqlite::{params, Connection};
use serde_json::Value;

use super::team_rows::{document_type_slug, json_field};
use super::{i64_opt, s, s_req};
use crate::domain::sync::hlc::Hlc;
use crate::error::AppError;

pub(super) fn upsert_diagnosis_template(
    conn: &Connection,
    v: &Value,
    hlc: &Hlc,
) -> Result<(), AppError> {
    let body_json = json_field(v, "body", "bodyJson")?;
    conn.execute(
        "INSERT INTO diagnosis_templates (
            id, name, body_json, created_at, updated_at,
            hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name, body_json = excluded.body_json,
            updated_at = excluded.updated_at, hlc_wall_ms = excluded.hlc_wall_ms,
            hlc_counter = excluded.hlc_counter, origin_device_id = excluded.origin_device_id,
            updated_by_staff_id = excluded.updated_by_staff_id, deleted_at = excluded.deleted_at",
        params![
            s_req(v, "id")?,
            s_req(v, "name")?,
            body_json,
            s(v, "createdAt").unwrap_or_default(),
            s(v, "updatedAt").unwrap_or_default(),
            hlc.wall,
            hlc.counter,
            hlc.origin_device_id,
            s(v, "updatedByStaffId"),
            s(v, "deletedAt"),
        ],
    )?;
    Ok(())
}

pub(super) fn upsert_repair_diagnosis(
    conn: &Connection,
    v: &Value,
    hlc: &Hlc,
) -> Result<(), AppError> {
    let result_json = json_field(v, "result", "resultJson")?;
    conn.execute(
        "INSERT INTO repair_diagnosis (
            id, repair_id, template_id, result_json, created_at, updated_at,
            hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
         ON CONFLICT(id) DO UPDATE SET
            template_id = excluded.template_id, result_json = excluded.result_json,
            updated_at = excluded.updated_at, hlc_wall_ms = excluded.hlc_wall_ms,
            hlc_counter = excluded.hlc_counter, origin_device_id = excluded.origin_device_id,
            updated_by_staff_id = excluded.updated_by_staff_id, deleted_at = excluded.deleted_at",
        params![
            s_req(v, "id")?,
            s_req(v, "repairId")?,
            s(v, "templateId"),
            result_json,
            s(v, "createdAt").unwrap_or_default(),
            s(v, "updatedAt").unwrap_or_default(),
            hlc.wall,
            hlc.counter,
            hlc.origin_device_id,
            s(v, "updatedByStaffId"),
            s(v, "deletedAt"),
        ],
    )?;
    Ok(())
}

pub(super) fn upsert_repair_image(conn: &Connection, v: &Value, hlc: &Hlc) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO repair_images (
            id, repair_id, original_path, thumb_path, caption, sort_order, created_at,
            content_hash, updated_at, hlc_wall_ms, hlc_counter, origin_device_id,
            updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
         ON CONFLICT(id) DO UPDATE SET
            original_path = excluded.original_path, thumb_path = excluded.thumb_path,
            caption = excluded.caption, sort_order = excluded.sort_order,
            content_hash = excluded.content_hash, updated_at = excluded.updated_at,
            hlc_wall_ms = excluded.hlc_wall_ms, hlc_counter = excluded.hlc_counter,
            origin_device_id = excluded.origin_device_id,
            updated_by_staff_id = excluded.updated_by_staff_id, deleted_at = excluded.deleted_at",
        params![
            s_req(v, "id")?,
            s_req(v, "repairId")?,
            s_req(v, "originalPath")?,
            s(v, "thumbPath"),
            s(v, "caption"),
            i64_opt(v, "sortOrder").unwrap_or(0),
            s(v, "createdAt").unwrap_or_default(),
            s(v, "contentHash").unwrap_or_default(),
            s(v, "updatedAt").unwrap_or_else(|| s(v, "createdAt").unwrap_or_default()),
            hlc.wall,
            hlc.counter,
            hlc.origin_device_id,
            s(v, "updatedByStaffId"),
            s(v, "deletedAt"),
        ],
    )?;
    Ok(())
}

pub(super) fn upsert_repair_document(
    conn: &Connection,
    v: &Value,
    hlc: &Hlc,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO repair_documents (
            id, repair_id, document_type, file_path, original_filename, content_hash,
            created_at, updated_at, hlc_wall_ms, hlc_counter, origin_device_id,
            updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
         ON CONFLICT(id) DO UPDATE SET
            file_path = excluded.file_path, original_filename = excluded.original_filename,
            content_hash = excluded.content_hash, updated_at = excluded.updated_at,
            hlc_wall_ms = excluded.hlc_wall_ms, hlc_counter = excluded.hlc_counter,
            origin_device_id = excluded.origin_device_id,
            updated_by_staff_id = excluded.updated_by_staff_id, deleted_at = excluded.deleted_at",
        params![
            s_req(v, "id")?,
            s_req(v, "repairId")?,
            document_type_slug(v)?,
            s_req(v, "filePath")?,
            s(v, "originalFilename").unwrap_or_else(|| "document".into()),
            s(v, "contentHash").unwrap_or_default(),
            s(v, "createdAt").unwrap_or_default(),
            s(v, "updatedAt").unwrap_or_default(),
            hlc.wall,
            hlc.counter,
            hlc.origin_device_id,
            s(v, "updatedByStaffId"),
            s(v, "deletedAt"),
        ],
    )?;
    Ok(())
}

pub(super) fn upsert_content_blob(conn: &Connection, v: &Value) -> Result<(), AppError> {
    let hash = s(v, "contentHash")
        .or_else(|| s(v, "id"))
        .ok_or(AppError::Internal {
            message: "sync payload missing contentHash".into(),
        })?;
    let kind = s(v, "kind").unwrap_or_else(|| "document".into());
    let created_at = s(v, "createdAt").unwrap_or_default();
    crate::domain::sync::blobs::upsert_blob_row(
        conn,
        &hash,
        i64_opt(v, "byteSize").unwrap_or(0),
        &kind,
        &created_at,
    )
}
