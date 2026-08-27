use std::fs;
use std::path::{Component, Path, PathBuf};

use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageFormat};
use uuid::Uuid;

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
use crate::error::AppError;
use crate::paths::AppPaths;

pub fn list_repair_images(db: &Db, repair_id: i64) -> Result<Vec<RepairImage>, AppError> {
    if repair_id <= 0 {
        return Err(AppError::Validation {
            field: Some("repairId".into()),
            message: "Repair is required.".into(),
        });
    }
    let _repair = repairs::get_repair(db.conn(), repair_id)?;
    repository::list_by_repair_id(db.conn(), repair_id)
}

pub fn attach_repair_images(
    db: &Db,
    input: AttachRepairImagesInput,
) -> Result<Vec<RepairImage>, AppError> {
    let validated = validate_attach_input(&input)?;
    let _repair = repairs::get_repair(db.conn(), validated.repair_id)?;

    let current = repository::count_by_repair_id(db.conn(), validated.repair_id)?;
    ensure_room_for_attachments(current, validated.source_paths.len())?;

    let mut sort_order = repository::next_sort_order(db.conn(), validated.repair_id)?;
    let created_at = now_utc_rfc3339()?;
    let mut attached = Vec::with_capacity(validated.source_paths.len());

    for source in &validated.source_paths {
        let image = store_one_image(db, validated.repair_id, source, sort_order, &created_at)?;
        attached.push(image);
        sort_order += 1;
    }

    Ok(attached)
}

pub fn update_repair_image(
    db: &Db,
    id: i64,
    input: UpdateRepairImageInput,
) -> Result<RepairImage, AppError> {
    if id <= 0 {
        return Err(AppError::NotFound);
    }
    let validated = validate_update_input(&input)?;
    repository::update(db.conn(), id, &validated)
}

pub fn delete_repair_image(db: &Db, id: i64) -> Result<(), AppError> {
    if id <= 0 {
        return Err(AppError::NotFound);
    }
    let existing = repository::get_by_id(db.conn(), id)?.ok_or(AppError::NotFound)?;
    repository::delete(db.conn(), id)?;
    remove_relative_file(db.paths(), &existing.original_path);
    if let Some(thumb) = &existing.thumb_path {
        remove_relative_file(db.paths(), thumb);
    }
    Ok(())
}

pub fn resolve_repair_image_path(
    db: &Db,
    id: i64,
    variant: ImageVariant,
) -> Result<ResolveRepairImagePathResult, AppError> {
    if id <= 0 {
        return Err(AppError::NotFound);
    }
    let existing = repository::get_by_id(db.conn(), id)?.ok_or(AppError::NotFound)?;
    let relative = match variant {
        ImageVariant::Original => existing.original_path.as_str(),
        ImageVariant::Thumb => existing
            .thumb_path
            .as_deref()
            .ok_or(AppError::NotFound)?,
    };

    let absolute = resolve_safe_absolute(db.paths(), relative)?;
    Ok(ResolveRepairImagePathResult {
        absolute_path: absolute.to_string_lossy().into_owned(),
    })
}

fn store_one_image(
    db: &Db,
    repair_id: i64,
    source: &Path,
    sort_order: i64,
    created_at: &str,
) -> Result<RepairImage, AppError> {
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_else(|| "jpg".into());
    let file_stem = Uuid::new_v4().to_string();
    let original_rel = format!("images/{repair_id}/{file_stem}.{ext}");
    let thumb_rel = format!("thumbs/{repair_id}/{file_stem}.jpg");

    let original_abs = db.paths().root.join(&original_rel);
    let thumb_abs = db.paths().root.join(&thumb_rel);

    if let Some(parent) = original_abs.parent() {
        fs::create_dir_all(parent)?;
    }
    if let Some(parent) = thumb_abs.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(source, &original_abs)?;
    generate_thumbnail(&original_abs, &thumb_abs)?;

    repository::insert(
        db.conn(),
        repair_id,
        &original_rel,
        Some(&thumb_rel),
        sort_order,
        created_at,
    )
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
    let allowed = allowed_roots.iter().flatten().any(|root| canonical.starts_with(root));
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
