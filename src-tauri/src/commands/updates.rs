//! Thin update-check and external-link IPC adapters.

use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use crate::domain::updates::{self, AppVersion, ReqwestLatestFetcher, UpdateCheck};
use crate::error::{AppError, CommandError};

#[tauri::command]
pub fn get_app_version() -> Result<AppVersion, CommandError> {
    Ok(updates::app_version())
}

#[tauri::command]
pub async fn check_app_update() -> Result<UpdateCheck, CommandError> {
    Ok(
        tokio::task::spawn_blocking(|| updates::check_app_update(&ReqwestLatestFetcher))
            .await
            .map_err(|_| AppError::network("join"))??,
    )
}

#[tauri::command(rename_all = "camelCase")]
pub fn open_external_url(app: AppHandle, url: String) -> Result<(), CommandError> {
    updates::validate_open_url(&url)?;
    app.opener().open_url(&url, None::<&str>).map_err(|_| {
        eprintln!("app_error code=internal reason=opener_failed");
        AppError::Internal {
            message: "opener failed".into(),
        }
    })?;
    Ok(())
}
