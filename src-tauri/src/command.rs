use crate::emoji_manager;
use crate::panel;
use crate::permissions::{ensure_accessibility_permission, reset_permission_cache};
use enigo::{Enigo, Keyboard, Settings};
use tauri::AppHandle;

#[tauri::command]
pub fn show_panel(handle: AppHandle) -> Result<(), String> {
    panel::show_panel(handle)
}

#[tauri::command]
pub fn hide_panel(handle: AppHandle) -> Result<(), String> {
    panel::hide_panel(handle)
}

#[tauri::command]
pub async fn type_emoji(_: AppHandle, emoji: String) -> Result<(), String> {
    // Ensure accessibility permission is granted (uses caching)
    ensure_accessibility_permission().await?;

    // Panel show already hidden and focus to the previously active application is restored
    // Short delay to allow focus restoration to complete
    std::thread::sleep(std::time::Duration::from_millis(200));

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

// Emoji manager commands
#[tauri::command]
pub fn get_emojis(filter_word: String) -> Result<Vec<String>, String> {
    emoji_manager::get_emojis(&filter_word)
}

#[tauri::command]
pub fn get_keywords(emoji: String) -> Result<Vec<String>, String> {
    emoji_manager::get_keywords(&emoji)
}

#[tauri::command]
pub fn increment_usage(emoji: String) -> Result<(), String> {
    emoji_manager::increment_usage(&emoji)
}

// #[tauri::command]
// pub fn close_panel(app_handle: AppHandle) {
//     app_handle
//         .get_webview_panel("main")
//         .ok()
//         .and_then(|panel| panel.to_window())
//         .map(|window| window.close());
// }
