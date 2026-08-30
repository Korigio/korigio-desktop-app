use rusqlite::{params, Connection, OptionalExtension};

use crate::db::repository::like_pattern;
use crate::domain::diagnosis::types::{
    DiagnosisResult, DiagnosisTemplate, DiagnosisTemplateBody, RepairDiagnosis,
};
use crate::domain::diagnosis::validation::{ValidatedRepairDiagnosisInput, ValidatedTemplateInput};
use crate::domain::sync::WriteContext;
use crate::error::AppError;

pub fn insert_template(
    conn: &Connection,
    input: &ValidatedTemplateInput,
    now: &str,
    ctx: &WriteContext,
) -> Result<DiagnosisTemplate, AppError> {
    let body_json = serialize_body(&input.body)?;
    let id = crate::domain::ids::new_entity_id();
    conn.execute(
        "INSERT INTO diagnosis_templates (
            id, name, body_json, created_at, updated_at,
            hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL)",
        params![
            id,
            input.name,
            body_json,
            now,
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id
        ],
    )?;
    get_template_by_id(conn, &id)?.ok_or(AppError::Internal {
        message: "diagnosis template missing after insert".into(),
    })
}

pub fn update_template(
    conn: &Connection,
    id: &str,
    input: &ValidatedTemplateInput,
    now: &str,
    ctx: &WriteContext,
) -> Result<DiagnosisTemplate, AppError> {
    let body_json = serialize_body(&input.body)?;
    let updated = conn.execute(
        "UPDATE diagnosis_templates SET name = ?1, body_json = ?2, updated_at = ?3,
            hlc_wall_ms = ?4, hlc_counter = ?5, origin_device_id = ?6, updated_by_staff_id = ?7
         WHERE id = ?8 AND deleted_at IS NULL",
        params![
            input.name,
            body_json,
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
    get_template_by_id(conn, id)?.ok_or(AppError::NotFound)
}

pub fn get_template_by_id(
    conn: &Connection,
    id: &str,
) -> Result<Option<DiagnosisTemplate>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, body_json, updated_at FROM diagnosis_templates
         WHERE id = ?1 AND deleted_at IS NULL",
    )?;
    let row = stmt
        .query_row(params![id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .optional()?;

    match row {
        None => Ok(None),
        Some((id, name, body_json, updated_at)) => {
            Ok(Some(map_template(id, name, &body_json, updated_at)?))
        }
    }
}

pub fn delete_template(
    conn: &Connection,
    id: &str,
    now: &str,
    ctx: &WriteContext,
) -> Result<(), AppError> {
    let updated = conn.execute(
        "UPDATE diagnosis_templates SET deleted_at = ?1, updated_at = ?1,
            hlc_wall_ms = ?2, hlc_counter = ?3, origin_device_id = ?4, updated_by_staff_id = ?5
         WHERE id = ?6 AND deleted_at IS NULL",
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
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub fn count_repair_diagnosis_by_template(
    conn: &Connection,
    template_id: &str,
) -> Result<i64, AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM repair_diagnosis WHERE template_id = ?1",
        params![template_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

pub fn list_templates(
    conn: &Connection,
    search: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<(Vec<DiagnosisTemplate>, i64), AppError> {
    let pattern = like_pattern(search);

    let total: i64 = match &pattern {
        None => conn.query_row(
            "SELECT COUNT(*) FROM diagnosis_templates WHERE deleted_at IS NULL",
            [],
            |row| row.get(0),
        )?,
        Some(p) => conn.query_row(
            "SELECT COUNT(*) FROM diagnosis_templates
             WHERE deleted_at IS NULL AND name LIKE ?1 ESCAPE '\\'",
            params![p],
            |row| row.get(0),
        )?,
    };

    let sql = match &pattern {
        None => {
            "SELECT id, name, body_json, updated_at
             FROM diagnosis_templates
             WHERE deleted_at IS NULL
             ORDER BY name COLLATE NOCASE ASC, id ASC
             LIMIT ?1 OFFSET ?2"
        }
        Some(_) => {
            "SELECT id, name, body_json, updated_at
             FROM diagnosis_templates
             WHERE deleted_at IS NULL AND name LIKE ?3 ESCAPE '\\'
             ORDER BY name COLLATE NOCASE ASC, id ASC
             LIMIT ?1 OFFSET ?2"
        }
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = match &pattern {
        None => stmt
            .query_map(params![limit, offset], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?,
        Some(p) => stmt
            .query_map(params![limit, offset, p], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?,
    };

    let mut items = Vec::with_capacity(rows.len());
    for (id, name, body_json, updated_at) in rows {
        items.push(map_template(id, name, &body_json, updated_at)?);
    }
    Ok((items, total))
}

pub fn get_by_repair_id(
    conn: &Connection,
    repair_id: &str,
) -> Result<Option<RepairDiagnosis>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, repair_id, template_id, result_json
         FROM repair_diagnosis WHERE repair_id = ?1",
    )?;
    let row = stmt
        .query_row(params![repair_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .optional()?;

    match row {
        None => Ok(None),
        Some((id, repair_id, template_id, result_json)) => Ok(Some(map_repair_diagnosis(
            id,
            repair_id,
            template_id,
            &result_json,
        )?)),
    }
}

pub fn upsert_repair_diagnosis(
    conn: &Connection,
    input: &ValidatedRepairDiagnosisInput,
    ctx: &WriteContext,
) -> Result<RepairDiagnosis, AppError> {
    let result_json = serialize_result(&input.result)?;

    if let Some(existing) = get_by_repair_id(conn, &input.repair_id)? {
        let now = crate::db::repository::now_utc_rfc3339()?;
        conn.execute(
            "UPDATE repair_diagnosis SET template_id = ?1, result_json = ?2, updated_at = ?3,
                hlc_wall_ms = ?4, hlc_counter = ?5, origin_device_id = ?6, updated_by_staff_id = ?7
             WHERE id = ?8",
            params![
                input.template_id,
                result_json,
                now,
                ctx.hlc.wall,
                ctx.hlc.counter,
                ctx.hlc.origin_device_id,
                ctx.staff_id,
                existing.id
            ],
        )?;
        return get_by_repair_id(conn, &input.repair_id)?.ok_or(AppError::Internal {
            message: "repair diagnosis missing after update".into(),
        });
    }

    let id = crate::domain::ids::new_entity_id();
    let now = crate::db::repository::now_utc_rfc3339()?;
    conn.execute(
        "INSERT INTO repair_diagnosis (
            id, repair_id, template_id, result_json, created_at, updated_at,
            hlc_wall_ms, hlc_counter, origin_device_id, updated_by_staff_id, deleted_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, NULL)",
        params![
            id,
            input.repair_id,
            input.template_id,
            result_json,
            now,
            now,
            ctx.hlc.wall,
            ctx.hlc.counter,
            ctx.hlc.origin_device_id,
            ctx.staff_id
        ],
    )?;
    get_by_repair_id(conn, &input.repair_id)?.ok_or(AppError::Internal {
        message: "repair diagnosis missing after insert".into(),
    })
}

fn serialize_body(body: &DiagnosisTemplateBody) -> Result<String, AppError> {
    serde_json::to_string(body).map_err(|err| AppError::Internal {
        message: format!("failed to serialize diagnosis template body: {err}"),
    })
}

fn serialize_result(result: &DiagnosisResult) -> Result<String, AppError> {
    serde_json::to_string(result).map_err(|err| AppError::Internal {
        message: format!("failed to serialize diagnosis result: {err}"),
    })
}

fn map_template(
    id: String,
    name: String,
    body_json: &str,
    updated_at: String,
) -> Result<DiagnosisTemplate, AppError> {
    let body: DiagnosisTemplateBody =
        serde_json::from_str(body_json).map_err(|err| AppError::Internal {
            message: format!("failed to parse diagnosis template body: {err}"),
        })?;
    Ok(DiagnosisTemplate {
        id,
        name,
        body,
        updated_at,
    })
}

fn map_repair_diagnosis(
    id: String,
    repair_id: String,
    template_id: Option<String>,
    result_json: &str,
) -> Result<RepairDiagnosis, AppError> {
    let result: DiagnosisResult =
        serde_json::from_str(result_json).map_err(|err| AppError::Internal {
            message: format!("failed to parse diagnosis result: {err}"),
        })?;
    Ok(RepairDiagnosis {
        id,
        repair_id,
        template_id,
        result,
    })
}
