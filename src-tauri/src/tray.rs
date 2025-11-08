use crate::constants::{
    HELP_WINDOW_HEIGHT, HELP_WINDOW_WIDTH, SETTINGS_WINDOW_HEIGHT, SETTINGS_WINDOW_WIDTH,
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, WebviewUrl, WebviewWindowBuilder,
};

pub fn init(app_handle: &AppHandle) -> tauri::Result<()> {
    let help_i = MenuItem::with_id(app_handle, "help", "Help", true, None::<&str>)?;
    let settings_i = MenuItem::with_id(app_handle, "settings", "Settings", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app_handle, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app_handle, &[&settings_i, &help_i, &quit_i])?;

    let _tray = TrayIconBuilder::new()
        .icon(app_handle.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "help" => {
                if let Err(e) = open_help_window(app) {
                    log::error!("Failed to open help window: {}", e);
                }
            }
            "settings" => {
                if let Err(e) = open_settings_window(app) {
                    log::error!("Failed to open settings window: {}", e);
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {
                log::warn!("menu item {:?} not handled", event.id);
            }
        })
        .build(app_handle)?;
    Ok(())
}

pub fn open_settings_window(app: &AppHandle) -> tauri::Result<()> {
    // Check if settings window already exists
    if let Some(window) = app.get_webview_window("settings") {
        window.set_focus()?;
        return Ok(());
    }

    // Create new settings window
    let window =
        WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
            .title("Settings - emojiq")
            .inner_size(SETTINGS_WINDOW_WIDTH, SETTINGS_WINDOW_HEIGHT)
            .resizable(false)
            .center()
            .focused(true)
            .build()?;

    // Explicitly set focus to ensure it gets it
    window.set_focus()?;

    Ok(())
}

pub fn open_help_window(app: &AppHandle) -> tauri::Result<()> {
    // Reuse existing help window if present
    if let Some(window) = app.get_webview_window("help") {
        window.set_focus()?;
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(app, "help", WebviewUrl::App("help.html".into()))
        .title("Shortcuts - emojiq")
        .inner_size(HELP_WINDOW_WIDTH, HELP_WINDOW_HEIGHT)
        .resizable(false)
        .always_on_top(true)
        .center()
        .focused(true)
        .build()?;

    window.set_focus()?;
    Ok(())
}
