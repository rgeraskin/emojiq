use crate::constants::FOCUS_RESTORATION_DELAY_MS;
use crate::panel;
use crate::permissions::{ensure_accessibility_permission, reset_permission_cache};
use crate::settings::{EmojiMode, Settings as AppSettings};
use crate::tray;
use crate::AppState;
use enigo::{Enigo, Keyboard, Settings};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_nspanel::ManagerExt;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub fn show_panel(handle: AppHandle) -> Result<(), String> {
    panel::show_panel(handle)
}

#[tauri::command]
pub fn hide_panel(handle: AppHandle) -> Result<(), String> {
    panel::hide_panel(handle)
}

/// Copy emoji to clipboard
async fn copy_emoji(handle: &AppHandle, emoji: &str) -> Result<(), String> {
    handle
        .clipboard()
        .write_text(emoji)
        .map_err(|e| format!("Failed to copy emoji to clipboard: {}", e))
}

/// Paste emoji to the previously focused window
async fn paste_emoji(emoji: &str) -> Result<(), String> {
    // Panel is already hidden and focus to the previously active application is being restored
    // Short delay to allow focus restoration to complete (offload blocking sleep)
    let delay = std::time::Duration::from_millis(FOCUS_RESTORATION_DELAY_MS);
    let _ = tauri::async_runtime::spawn_blocking(move || std::thread::sleep(delay)).await;

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to initialize Enigo: {}", e))?;

    enigo
        .text(emoji)
        .map_err(|e| format!("Failed to type emoji: {}", e))
}

#[tauri::command]
pub async fn type_emoji(
    handle: AppHandle,
    state: State<'_, AppState>,
    emoji: String,
) -> Result<(), String> {
    // Get current settings to determine emoji mode
    let settings = state.settings_manager.get().map_err(|e| e.to_string())?;

    match settings.emoji_mode {
        EmojiMode::PasteOnly => {
            ensure_accessibility_permission()
                .await
                .map_err(|e| e.to_string())?;
            paste_emoji(&emoji).await?;
        }
        EmojiMode::CopyOnly => {
            copy_emoji(&handle, &emoji).await?;
        }
        EmojiMode::PasteAndCopy => {
            ensure_accessibility_permission()
                .await
                .map_err(|e| e.to_string())?;
            copy_emoji(&handle, &emoji).await?;
            paste_emoji(&emoji).await?;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn reset_accessibility_cache() {
    reset_permission_cache();
}

// Emoji manager commands
#[tauri::command]
pub fn get_emojis(state: State<AppState>, filter_word: String) -> Result<Vec<String>, String> {
    let settings = state.settings_manager.get().map_err(|e| e.to_string())?;
    state
        .emoji_manager
        .get_emojis(&filter_word, settings.max_top_emojis)
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

// Settings commands
#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<AppSettings, String> {
    state.settings_manager.get()
}

#[tauri::command]
pub fn update_settings(
    handle: AppHandle,
    state: State<AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    state.settings_manager.update(settings)?;

    // Notify main window to refresh emoji list if it exists
    if let Some(main_window) = handle.get_webview_window("main") {
        let _ = main_window.emit("settings-changed", ());
    }

    Ok(())
}

#[tauri::command]
pub fn open_settings(handle: AppHandle, state: State<AppState>) -> Result<(), String> {
    // Set flag to indicate we're opening settings
    state
        .opening_settings
        .store(true, std::sync::atomic::Ordering::Release);

    // Hide the main panel first if it's open to avoid focus conflicts
    if let Ok(panel) = handle.get_webview_panel("main") {
        if panel.is_visible() {
            panel.hide();
            // Small delay to ensure panel is fully hidden before opening settings
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    let result = tray::open_settings_window(&handle).map_err(|e| e.to_string());

    // Clear the flag after settings window is opened
    state
        .opening_settings
        .store(false, std::sync::atomic::Ordering::Release);

    result
}

#[tauri::command]
pub fn save_window_size(state: State<AppState>, width: f64, height: f64) -> Result<(), String> {
    state.settings_manager.update_window_size(width, height)
}

// #[tauri::command]
// pub fn close_panel(app_handle: AppHandle) {
//     app_handle
//         .get_webview_panel("main")
//         .ok()
//         .and_then(|panel| panel.to_window())
//         .map(|window| window.close());
// }
