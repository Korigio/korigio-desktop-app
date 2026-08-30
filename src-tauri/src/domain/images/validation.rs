use std::path::{Path, PathBuf};

use crate::domain::images::constants::{
    ALLOWED_EXTENSIONS, CAPTION_MAX_LEN, MAX_IMAGES_PER_REPAIR, MAX_IMAGE_BYTES,
};
use crate::domain::images::types::{AttachRepairImagesInput, UpdateRepairImageInput};
use crate::error::AppError;

#[derive(Debug)]
pub struct ValidatedAttachInput {
    pub repair_id: String,
    pub source_paths: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct ValidatedUpdateInput {
    pub caption: Option<Option<String>>,
    pub sort_order: Option<i64>,
}

pub fn validate_attach_input(
    input: &AttachRepairImagesInput,
) -> Result<ValidatedAttachInput, AppError> {
    let repair_id = crate::domain::ids::parse_entity_id_field(&input.repair_id, "repairId")?;
    if input.source_paths.is_empty() {
        return Err(AppError::Validation {
            field: Some("sourcePaths".into()),
            message: "At least one image path is required.".into(),
        });
    }

    let mut source_paths = Vec::with_capacity(input.source_paths.len());
    for (index, raw) in input.source_paths.iter().enumerate() {
        let path = PathBuf::from(raw);
        validate_source_image(&path, index)?;
        source_paths.push(path);
    }

    Ok(ValidatedAttachInput {
        repair_id,
        source_paths,
    })
}

pub fn validate_source_image(path: &Path, index: usize) -> Result<(), AppError> {
    let field = format!("sourcePaths[{index}]");
    if !path.is_file() {
        return Err(AppError::Validation {
            field: Some(field),
            message: "Image file was not found.".into(),
        });
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    if !ALLOWED_EXTENSIONS.iter().any(|allowed| *allowed == ext) {
        return Err(AppError::Validation {
            field: Some(field),
            message: "Only JPEG and PNG images are supported.".into(),
        });
    }

    let meta = std::fs::metadata(path)?;
    if meta.len() > MAX_IMAGE_BYTES {
        return Err(AppError::Validation {
            field: Some(field),
            message: format!(
                "Each image must be at most {} MB.",
                MAX_IMAGE_BYTES / (1024 * 1024)
            ),
        });
    }

    Ok(())
}

pub fn validate_update_input(
    input: &UpdateRepairImageInput,
) -> Result<ValidatedUpdateInput, AppError> {
    if input.caption.is_none() && input.sort_order.is_none() {
        return Err(AppError::Validation {
            field: None,
            message: "No changes were provided.".into(),
        });
    }

    let caption = match &input.caption {
        None => None,
        Some(None) => Some(None),
        Some(Some(raw)) => {
            let trimmed = raw.trim();
            if trimmed.chars().count() > CAPTION_MAX_LEN {
                return Err(AppError::Validation {
                    field: Some("caption".into()),
                    message: format!("Caption must be at most {CAPTION_MAX_LEN} characters."),
                });
            }
            if trimmed.is_empty() {
                Some(None)
            } else {
                Some(Some(trimmed.to_string()))
            }
        }
    };

    Ok(ValidatedUpdateInput {
        caption,
        sort_order: input.sort_order,
    })
}

pub fn ensure_room_for_attachments(current_count: i64, incoming: usize) -> Result<(), AppError> {
    let incoming = incoming as i64;
    if current_count + incoming > MAX_IMAGES_PER_REPAIR {
        return Err(AppError::Validation {
            field: Some("sourcePaths".into()),
            message: format!("A repair can have at most {MAX_IMAGES_PER_REPAIR} images."),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn rejects_bad_extension() {
        let mut file = NamedTempFile::new().expect("temp");
        writeln!(file, "not-an-image").unwrap();
        let path = file.path().with_extension("gif");
        std::fs::copy(file.path(), &path).unwrap();
        let err = validate_source_image(&path, 0).expect_err("gif");
        assert!(matches!(err, AppError::Validation { .. }));
        let _ = std::fs::remove_file(path);
    }
}
