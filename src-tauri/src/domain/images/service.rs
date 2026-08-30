use std::fs;
use std::path::{Component, Path, PathBuf};

use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageFormat};

use crate::db::repository::now_utc_rfc3339;
use crate::db::Db;
use crate::domain::images::constants::THUMB_MAX_EDGE;
use crate::domain::images::repository;
use crate::domain::images::types::{
    AttachRepairImagesInput, ImageVariant, RepairImage, ResolveRepairImagePathResult,
    UpdateRepairImageInput,
};
use crate::domain::images::validation::{
    ensure_room_for_attachments, validate_attach_input, validate_update_input,
};
use crate::domain::repairs;
use crate::domain::sync::{self, begin_write, WriteContext};
use crate::error::AppError;
use crate::paths::AppPaths;

fn record_image(
    conn: &rusqlite::Connection,
    image: &RepairImage,
    ctx: &WriteContext,
) -> Result<(), AppError> {
    let payload = serde_json::to_value(image).map_err(|err| AppError::Internal {
        message: format!("serialize repair image: {err}"),
    })?;
    sync::record_upsert(conn, "repair_images", &image.id, payload, ctx)?;
    Ok(())
}

pub fn list_repair_images(db: &Db, repair_id: String) -> Result<Vec<RepairImage>, AppError> {
    let repair_id = crate::domain::ids::parse_entity_id_field(&repair_id, "repairId")?;
    let _repair = repairs::get_repair(db.conn(), repair_id.clone())?;
    repository::list_by_repair_id(db.conn(), &repair_id)
}

pub fn attach_repair_images(
    db: &Db,
    input: AttachRepairImagesInput,
) -> Result<Vec<RepairImage>, AppError> {
    let validated = validate_attach_input(&input)?;
    let _repair = repairs::get_repair(db.conn(), validated.repair_id.clone())?;

    let current = repository::count_by_repair_id(db.conn(), &validated.repair_id)?;
    ensure_room_for_attachments(current, validated.source_paths.len())?;

    let mut sort_order = repository::next_sort_order(db.conn(), &validated.repair_id)?;
    let created_at = now_utc_rfc3339()?;
    let mut attached = Vec::with_capacity(validated.source_paths.len());

    for source in &validated.source_paths {
        let image = store_one_image(db, &validated.repair_id, source, sort_order, &created_at)?;
        attached.push(image);
        sort_order += 1;
    }

    Ok(attached)
}

pub fn update_repair_image(
    db: &Db,
    id: String,
    input: UpdateRepairImageInput,
) -> Result<RepairImage, AppError> {
    let id = crate::domain::ids::parse_entity_id(&id).map_err(|_| AppError::NotFound)?;
    let validated = validate_update_input(&input)?;
    let ctx = begin_write(db.conn())?;
    let image = repository::update(db.conn(), &id, &validated, &ctx)?;
    record_image(db.conn(), &image, &ctx)?;
    Ok(image)
}

pub fn delete_repair_image(db: &Db, id: String) -> Result<(), AppError> {
    let id = crate::domain::ids::parse_entity_id(&id).map_err(|_| AppError::NotFound)?;
    let existing = repository::get_by_id(db.conn(), &id)?.ok_or(AppError::NotFound)?;
    let ctx = begin_write(db.conn())?;
    repository::delete(db.conn(), &id, &ctx)?;
    sync::record_delete(
        db.conn(),
        "repair_images",
        &id,
        serde_json::json!({ "id": id }),
        &ctx,
    )?;
    remove_relative_file(db.paths(), &existing.original_path);
    if let Some(thumb) = &existing.thumb_path {
        remove_relative_file(db.paths(), thumb);
    }
    Ok(())
}

pub fn resolve_repair_image_path(
    db: &Db,
    id: String,
    variant: ImageVariant,
) -> Result<ResolveRepairImagePathResult, AppError> {
    let id = crate::domain::ids::parse_entity_id(&id).map_err(|_| AppError::NotFound)?;
    let existing = repository::get_by_id(db.conn(), &id)?.ok_or(AppError::NotFound)?;
    let relative = match variant {
        ImageVariant::Original => existing.original_path.as_str(),
        ImageVariant::Thumb => existing.thumb_path.as_deref().ok_or(AppError::NotFound)?,
    };

    let absolute = resolve_safe_absolute(db.paths(), relative)?;
    Ok(ResolveRepairImagePathResult {
        absolute_path: absolute.to_string_lossy().into_owned(),
    })
}

fn store_one_image(
    db: &Db,
    repair_id: &str,
    source: &Path,
    sort_order: i64,
    created_at: &str,
) -> Result<RepairImage, AppError> {
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_else(|| "jpg".into());
    let bytes = fs::read(source)?;
    let hash = crate::domain::sync::blobs::write_blob_bytes(
        &db.paths().root,
        db.conn(),
        &bytes,
        "image",
        created_at,
    )?;
    crate::domain::sync::blobs::record_local_blob(
        db.conn(),
        &hash,
        "image",
        bytes.len() as i64,
        created_at,
    )?;
    let original_rel = format!("images/{repair_id}/{hash}.{ext}");
    let thumb_rel = format!("thumbs/{repair_id}/{hash}.jpg");

    crate::domain::sync::blobs::copy_to_display_path(&db.paths().root, &hash, &original_rel)?;
    let original_abs = db.paths().root.join(&original_rel);
    let thumb_abs = db.paths().root.join(&thumb_rel);
    if let Some(parent) = thumb_abs.parent() {
        fs::create_dir_all(parent)?;
    }
    generate_thumbnail(&original_abs, &thumb_abs)?;
    if let Ok(thumb_bytes) = fs::read(&thumb_abs) {
        if let Ok(thumb_hash) = crate::domain::sync::blobs::write_blob_bytes(
            &db.paths().root,
            db.conn(),
            &thumb_bytes,
            "thumb",
            created_at,
        ) {
            let _ = crate::domain::sync::blobs::record_local_blob(
                db.conn(),
                &thumb_hash,
                "thumb",
                thumb_bytes.len() as i64,
                created_at,
            );
        }
    }

    let ctx = begin_write(db.conn())?;
    let image = repository::insert(
        db.conn(),
        repair_id,
        &original_rel,
        Some(&thumb_rel),
        &hash,
        sort_order,
        created_at,
        &ctx,
    )?;
    record_image(db.conn(), &image, &ctx)?;
    Ok(image)
}

fn generate_thumbnail(source: &Path, dest: &Path) -> Result<(), AppError> {
    let img = image::open(source).map_err(|err| AppError::Validation {
        field: Some("sourcePaths".into()),
        message: format!("Could not read image: {err}"),
    })?;
    let thumb = resize_max_edge(img, THUMB_MAX_EDGE);
    thumb
        .save_with_format(dest, ImageFormat::Jpeg)
        .map_err(|err| AppError::Internal {
            message: format!("failed to write thumbnail: {err}"),
        })?;
    Ok(())
}

fn resize_max_edge(img: DynamicImage, max_edge: u32) -> DynamicImage {
    let (w, h) = img.dimensions();
    if w <= max_edge && h <= max_edge {
        return img;
    }
    img.resize(max_edge, max_edge, FilterType::Triangle)
}

fn remove_relative_file(paths: &AppPaths, relative: &str) {
    let absolute = paths.root.join(relative);
    match fs::remove_file(&absolute) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            eprintln!(
                "failed to remove image file path_kind={} err={err}",
                if relative.starts_with("thumbs/") {
                    "thumb"
                } else {
                    "original"
                }
            );
        }
    }
}

/// Resolve a stored relative path to an absolute path, only if it stays under images/ or thumbs/.
pub fn resolve_safe_absolute(paths: &AppPaths, relative: &str) -> Result<PathBuf, AppError> {
    if relative.is_empty()
        || Path::new(relative).is_absolute()
        || has_parent_dir_component(relative)
    {
        return Err(AppError::Validation {
            field: Some("path".into()),
            message: "Image path is invalid.".into(),
        });
    }

    let under_images = relative.starts_with("images/") || relative.starts_with("images\\");
    let under_thumbs = relative.starts_with("thumbs/") || relative.starts_with("thumbs\\");
    if !under_images && !under_thumbs {
        return Err(AppError::Validation {
            field: Some("path".into()),
            message: "Image path is invalid.".into(),
        });
    }

    let candidate = paths.root.join(relative);
    let canonical = candidate.canonicalize().map_err(|_| AppError::Validation {
        field: Some("path".into()),
        message: "Image file was not found.".into(),
    })?;

    let allowed_roots = [
        paths.images.canonicalize().ok(),
        paths.thumbs.canonicalize().ok(),
    ];
    let allowed = allowed_roots
        .iter()
        .flatten()
        .any(|root| canonical.starts_with(root));
    if !allowed {
        return Err(AppError::Validation {
            field: Some("path".into()),
            message: "Image path is invalid.".into(),
        });
    }

    Ok(canonical)
}

fn has_parent_dir_component(relative: &str) -> bool {
    Path::new(relative)
        .components()
        .any(|c| matches!(c, Component::ParentDir))
}
