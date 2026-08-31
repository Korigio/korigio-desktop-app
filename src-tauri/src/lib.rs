mod commands;
mod db;
mod domain;
mod error;
mod menu;
mod paths;
mod sync_net;

use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::db::{Db, DbState};
use crate::paths::AppPaths;
use crate::sync_net::SyncRuntime;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|err| std::io::Error::other(format!("app data dir unavailable: {err}")))?;

            let paths = AppPaths::from_app_data_dir(app_data_dir).map_err(|err| {
                eprintln!("failed to prepare app data paths: {err:?}");
                std::io::Error::other(err.to_string())
            })?;

            let database = Db::open(paths).map_err(|err| {
                eprintln!("failed to open database: {err:?}");
                std::io::Error::other(err.to_string())
            })?;

            let db = Arc::new(Mutex::new(database));
            let runtime = SyncRuntime::new();
            runtime.start(db.clone(), app.handle().clone());
            app.manage(DbState(db));
            app.manage(runtime);

            if let Err(err) = menu::install_app_menu(app.handle()) {
                eprintln!("failed to install application menu: {err}");
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_status,
            commands::db_health,
            commands::get_locale_settings,
            commands::set_locale_preference,
            commands::get_theme_settings,
            commands::set_theme_preference,
            commands::get_auto_backup_settings,
            commands::set_auto_backup_settings,
            commands::get_shop_settings,
            commands::set_shop_settings,
            commands::get_sync_interval,
            commands::set_sync_interval,
            commands::list_companies,
            commands::get_company,
            commands::create_company,
            commands::update_company,
            commands::archive_company,
            commands::unarchive_company,
            commands::set_default_company,
            commands::attach_company_logo,
            commands::clear_company_logo,
            commands::resolve_company_logo_path,
            commands::list_customers,
            commands::get_customer,
            commands::create_customer,
            commands::update_customer,
            commands::archive_customer,
            commands::unarchive_customer,
            commands::list_devices,
            commands::get_device,
            commands::create_device,
            commands::update_device,
            commands::archive_device,
            commands::unarchive_device,
            commands::list_repairs,
            commands::get_repair,
            commands::create_repair,
            commands::update_repair,
            commands::complete_repair_diagnosis,
            commands::list_repair_documents,
            commands::upload_repair_document,
            commands::delete_repair_document,
            commands::open_repair_document,
            commands::confirm_repair_intake,
            commands::confirm_customer_approval,
            commands::confirm_repair_parts_received,
            commands::complete_repair_protocol,
            commands::confirm_repair_summary,
            commands::complete_repair_pickup,
            commands::assign_repair,
            commands::take_over_repair,
            commands::get_home_dashboard,
            commands::list_diagnosis_templates,
            commands::get_diagnosis_template,
            commands::create_diagnosis_template,
            commands::update_diagnosis_template,
            commands::delete_diagnosis_template,
            commands::get_repair_diagnosis,
            commands::upsert_repair_diagnosis,
            commands::list_repair_images,
            commands::attach_repair_images,
            commands::update_repair_image,
            commands::delete_repair_image,
            commands::resolve_repair_image_path,
            commands::create_backup,
            commands::validate_backup,
            commands::restore_backup,
            commands::list_local_backups,
            commands::run_auto_backup_if_due,
            commands::get_repair_print_report,
            commands::get_repair_diagnosis_print_report,
            commands::get_repair_summary_print_report,
            commands::list_staff,
            commands::get_staff,
            commands::create_staff,
            commands::update_staff,
            commands::deactivate_staff,
            commands::reactivate_staff,
            commands::change_staff_role,
            commands::set_staff_pin,
            commands::sign_in_staff,
            commands::sign_out_staff,
            commands::get_current_session,
            commands::get_team,
            commands::create_team,
            commands::leave_team,
            commands::get_team_pin,
            commands::list_nearby_teams,
            commands::list_team_members,
            commands::create_team_invite,
            commands::list_team_invites,
            commands::revoke_team_invite,
            commands::join_team,
            commands::list_team_devices,
            commands::remove_team_device,
            commands::rename_this_device,
            commands::list_presence,
            commands::get_sync_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Korigio");
}
