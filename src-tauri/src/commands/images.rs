//! Thin repair-image IPC adapters.

use tauri::State;

use crate::db::DbState;
use crate::domain::images::{
    self, AttachRepairImagesInput, ImageVariant, RepairImage, ResolveRepairImagePathResult,
    UpdateRepairImageInput,
};
use crate::error::{AppError, CommandError};

fn lock_db<'a>(
    state: &'a State<'_, DbState>,
) -> Result<std::sync::MutexGuard<'a, crate::db::Db>, AppError> {
    state.0.lock().map_err(|_| AppError::Internal {
        message: "database lock poisoned".into(),
    })
}

#[tauri::command(rename_all = "camelCase")]
pub fn list_repair_images(
    state: State<'_, DbState>,
    repair_id: i64,
) -> Result<Vec<RepairImage>, CommandError> {
    let db = lock_db(&state)?;
    Ok(images::list_repair_images(&db, repair_id)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn attach_repair_images(
    state: State<'_, DbState>,
    input: AttachRepairImagesInput,
) -> Result<Vec<RepairImage>, CommandError> {
    let db = lock_db(&state)?;
    Ok(images::attach_repair_images(&db, input)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn update_repair_image(
    state: State<'_, DbState>,
    id: i64,
    input: UpdateRepairImageInput,
) -> Result<RepairImage, CommandError> {
    let db = lock_db(&state)?;
    Ok(images::update_repair_image(&db, id, input)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn delete_repair_image(state: State<'_, DbState>, id: i64) -> Result<(), CommandError> {
    let db = lock_db(&state)?;
    Ok(images::delete_repair_image(&db, id)?)
}

#[tauri::command(rename_all = "camelCase")]
pub fn resolve_repair_image_path(
    state: State<'_, DbState>,
    id: i64,
    variant: ImageVariant,
) -> Result<ResolveRepairImagePathResult, CommandError> {
    let db = lock_db(&state)?;
    Ok(images::resolve_repair_image_path(&db, id, variant)?)
}
