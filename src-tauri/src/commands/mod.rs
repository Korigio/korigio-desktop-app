//! Thin Tauri command adapters. Business logic will live under `domain/` in later phases.

#[tauri::command]
pub fn app_status() -> String {
    "Repair Manager backend ready".to_string()
}
