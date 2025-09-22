use crate::constants::*;
use crate::panel;
use crate::permissions::{ensure_accessibility_permission, reset_permission_cache};
use crate::AppState;
use enigo::{Enigo, Keyboard, Settings};
use tauri::{AppHandle, State};

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
    ensure_accessibility_permission()
        .await
        .map_err(|e| e.to_string())?;

    // Panel is already hidden and focus to the previously active application is being restored
    // Short delay to allow focus restoration to complete (offload blocking sleep)
    let delay = std::time::Duration::from_millis(FOCUS_RESTORATION_DELAY_MS);
    let _ = tauri::async_runtime::spawn_blocking(move || std::thread::sleep(delay)).await;

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
pub fn get_emojis(state: State<AppState>, filter_word: String) -> Result<Vec<String>, String> {
    state
        .emoji_manager
        .get_emojis(&filter_word)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_keywords(state: State<AppState>, emoji: String) -> Result<Vec<String>, String> {
    state
        .emoji_manager
        .get_keywords(&emoji)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn increment_usage(state: State<AppState>, emoji: String) -> Result<(), String> {
    state
        .emoji_manager
        .increment_usage(&emoji)
        .map_err(|e| e.to_string())
}

// #[tauri::command]
// pub fn close_panel(app_handle: AppHandle) {
//     app_handle
//         .get_webview_panel("main")
//         .ok()
//         .and_then(|panel| panel.to_window())
//         .map(|window| window.close());
// }
