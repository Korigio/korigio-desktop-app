//! Native application menu (desktop).

use tauri::{
    AppHandle, Emitter, Manager, Runtime,
    menu::{AboutMetadata, MenuBuilder, MenuItemBuilder, SubmenuBuilder},
};

/// Event name listened to by the React shell.
pub const OPEN_SETTINGS_EVENT: &str = "open-settings";
pub const OPEN_REPAIR_INTAKE_EVENT: &str = "open-repair-intake";

const SETTINGS_MENU_ID: &str = "settings";
const REPAIR_INTAKE_MENU_ID: &str = "repair-intake";

pub fn install_app_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let settings = MenuItemBuilder::with_id(SETTINGS_MENU_ID, "Settings…")
        .accelerator("CmdOrCtrl+,")
        .build(app)?;

    let repair_intake = MenuItemBuilder::with_id(REPAIR_INTAKE_MENU_ID, "New repair intake…")
        .accelerator("CmdOrCtrl+Shift+N")
        .build(app)?;

    #[cfg(target_os = "macos")]
    let app_submenu = {
        SubmenuBuilder::new(app, "Repair Manager")
            .about(Some(AboutMetadata {
                name: Some("Repair Manager".into()),
                ..Default::default()
            }))
            .separator()
            .item(&settings)
            .item(&repair_intake)
            .separator()
            .services()
            .separator()
            .hide()
            .hide_others()
            .show_all()
            .separator()
            .quit()
            .build()?
    };

    let file_submenu = {
        #[cfg(target_os = "macos")]
        {
            SubmenuBuilder::new(app, "File").close_window().build()?
        }
        #[cfg(not(target_os = "macos"))]
        {
            SubmenuBuilder::new(app, "File")
                .item(&repair_intake)
                .item(&settings)
                .separator()
                .quit()
                .build()?
        }
    };

    let edit_submenu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    let window_submenu = SubmenuBuilder::new(app, "Window")
        .minimize()
        .separator()
        .close_window()
        .build()?;

    #[cfg(target_os = "macos")]
    let menu = MenuBuilder::new(app)
        .items(&[&app_submenu, &file_submenu, &edit_submenu, &window_submenu])
        .build()?;

    #[cfg(not(target_os = "macos"))]
    let menu = MenuBuilder::new(app)
        .items(&[&file_submenu, &edit_submenu, &window_submenu])
        .build()?;

    app.set_menu(menu)?;

    app.on_menu_event(|app, event| {
        let menu_id = event.id().as_ref();
        let event_name = if menu_id == SETTINGS_MENU_ID {
            Some(OPEN_SETTINGS_EVENT)
        } else if menu_id == REPAIR_INTAKE_MENU_ID {
            Some(OPEN_REPAIR_INTAKE_EVENT)
        } else {
            None
        };

        let Some(event_name) = event_name else {
            return;
        };

        let dom_event = if menu_id == SETTINGS_MENU_ID {
            "rm:open-settings"
        } else {
            "rm:open-repair-intake"
        };

        // Prefer targeting the main webview; fall back to app-wide emit.
        if let Some(window) = app
            .get_webview_window("main")
            .or_else(|| app.webview_windows().into_values().next())
        {
            let _ = window.emit(event_name, ());
            // DOM fallback if the event channel is not yet subscribed.
            let _ = window.eval(&format!(
                "window.dispatchEvent(new CustomEvent('{dom_event}'));"
            ));
            let _ = window.set_focus();
        } else {
            let _ = app.emit(event_name, ());
        }
    });

    Ok(())
}
