use crate::permissions::{ensure_accessibility_permission, reset_permission_cache};
use enigo::{Enigo, Keyboard, Settings};
use tauri::AppHandle;
use tauri_nspanel::ManagerExt;

#[tauri::command]
pub fn show_panel(handle: AppHandle) -> Result<(), String> {
    let panel = handle
        .get_webview_panel("main")
        .map_err(|e| format!("Failed to get main panel: {:?}", e))?;
    panel.show_and_make_key();
    Ok(())
}

#[tauri::command]
pub fn hide_panel(handle: AppHandle) -> Result<(), String> {
    let panel = handle
        .get_webview_panel("main")
        .map_err(|e| format!("Failed to get main panel: {:?}", e))?;
    panel.hide();
    Ok(())
}

#[tauri::command]
pub fn close_panel(app_handle: AppHandle) {
    app_handle
        .get_webview_panel("main")
        .ok()
        .and_then(|panel| panel.to_window())
        .map(|window| window.close());
}

#[tauri::command]
pub async fn type_emoji(_: AppHandle, emoji: String) -> Result<(), String> {
    // Ensure accessibility permission is granted (uses caching)
    ensure_accessibility_permission().await?;

    // Small delay to ensure the panel is hidden before typing
    std::thread::sleep(std::time::Duration::from_millis(100));

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to initialize Enigo: {}", e))?;

    // Write emoji text
    enigo
        .text(&emoji)
        .map_err(|e| format!("Failed to type emoji: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn reset_accessibility_cache() {
    reset_permission_cache();
}
