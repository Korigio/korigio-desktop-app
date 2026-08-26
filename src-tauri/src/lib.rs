mod commands;
mod db;
mod error;
mod paths;

use std::sync::Mutex;

use tauri::Manager;

use crate::db::{Db, DbState};
use crate::paths::AppPaths;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().map_err(|err| {
                std::io::Error::other(format!("app data dir unavailable: {err}"))
            })?;

            let paths = AppPaths::from_app_data_dir(app_data_dir).map_err(|err| {
                eprintln!("failed to prepare app data paths: {err:?}");
                std::io::Error::other(err.to_string())
            })?;

            let database = Db::open(paths).map_err(|err| {
                eprintln!("failed to open database: {err:?}");
                std::io::Error::other(err.to_string())
            })?;

            app.manage(DbState(Mutex::new(database)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_status,
            commands::db_health
        ])
        .run(tauri::generate_context!())
        .expect("error while running Repair Manager");
}
