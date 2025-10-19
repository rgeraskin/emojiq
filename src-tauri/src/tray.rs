use crate::constants::{SETTINGS_WINDOW_HEIGHT, SETTINGS_WINDOW_WIDTH};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, WebviewUrl, WebviewWindowBuilder,
};

pub fn init(app_handle: &AppHandle) -> tauri::Result<()> {
    let settings_i = MenuItem::with_id(app_handle, "settings", "Settings", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app_handle, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app_handle, &[&settings_i, &quit_i])?;

    let _tray = TrayIconBuilder::new()
        .icon(app_handle.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => {
                if let Err(e) = open_settings_window(app) {
                    println!("Failed to open settings window: {}", e);
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {
                println!("menu item {:?} not handled", event.id);
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
