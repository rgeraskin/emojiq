use crate::constants::{
    FOCUS_RESTORATION_DELAY_MS, HOTKEY_UNREGISTER_WAIT_MS, MAX_SCALE_FACTOR, MAX_TOP_EMOJIS_LIMIT,
    MIN_SCALE_FACTOR,
};
use crate::errors::EmojiError;
use crate::hotkey;
use crate::panel;
use crate::permissions::{ensure_accessibility_permission, reset_permission_cache};
use crate::settings::{EmojiMode, Settings as AppSettings};
use crate::tray;
use crate::AppState;
use enigo::{Enigo, Keyboard, Settings};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

#[tauri::command]
pub fn show_panel(handle: AppHandle) -> Result<(), EmojiError> {
    panel::show_panel(handle)
}

#[tauri::command]
pub fn hide_panel(handle: AppHandle) -> Result<(), EmojiError> {
    panel::hide_panel(handle)
}

/// Copy emoji to clipboard
async fn copy_emoji(handle: &AppHandle, emoji: &str) -> Result<(), EmojiError> {
    handle
        .clipboard()
        .write_text(emoji)
        .map_err(|e| EmojiError::Tauri(format!("Failed to copy emoji to clipboard: {}", e)))
}

/// Paste emoji to the previously focused window
async fn paste_emoji(emoji: &str) -> Result<(), EmojiError> {
    // Panel is already hidden and focus to the previously active application is being restored
    // Short delay to allow focus restoration to complete (offload blocking sleep)
    let delay = std::time::Duration::from_millis(FOCUS_RESTORATION_DELAY_MS);
    let _ = tauri::async_runtime::spawn_blocking(move || std::thread::sleep(delay)).await;

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| EmojiError::Tauri(format!("Failed to initialize Enigo: {}", e)))?;

    enigo
        .text(emoji)
        .map_err(|e| EmojiError::Tauri(format!("Failed to type emoji: {}", e)))
}

#[tauri::command]
pub async fn type_emoji(
    handle: AppHandle,
    state: State<'_, AppState>,
    emoji: String,
) -> Result<(), EmojiError> {
    // Get current settings to determine emoji mode
    let settings = state.settings_manager.get()?;

    match settings.emoji_mode {
        EmojiMode::PasteOnly => {
            ensure_accessibility_permission().await?;
            paste_emoji(&emoji).await?;
        }
        EmojiMode::CopyOnly => {
            copy_emoji(&handle, &emoji).await?;
        }
        EmojiMode::PasteAndCopy => {
            ensure_accessibility_permission().await?;
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
pub fn get_emojis(state: State<AppState>, filter_word: String) -> Result<Vec<String>, EmojiError> {
    let settings = state.settings_manager.get()?;
    state
        .emoji_manager
        .get_emojis(&filter_word, settings.max_top_emojis)
}

#[tauri::command]
pub fn get_keywords(state: State<AppState>, emoji: String) -> Result<Vec<String>, EmojiError> {
    state.emoji_manager.get_keywords(&emoji)
}

#[tauri::command]
pub fn increment_usage(state: State<AppState>, emoji: String, amount: Option<u32>) -> Result<(), EmojiError> {
    state.emoji_manager.increment_usage(&emoji, amount)
}

#[tauri::command]
pub fn remove_emoji_rank(
    handle: AppHandle,
    state: State<AppState>,
    emoji: String,
) -> Result<(), EmojiError> {
    state.emoji_manager.remove_emoji_rank(&emoji)?;

    // Notify main window to refresh emoji list if it exists
    if let Some(main_window) = handle.get_webview_window("main") {
        let _ = main_window.emit("settings-changed", ());
    }

    Ok(())
}

#[tauri::command]
pub fn reset_emoji_ranks(handle: AppHandle, state: State<AppState>) -> Result<(), EmojiError> {
    state.emoji_manager.reset_ranks()?;

    // Notify main window to refresh emoji list if it exists
    if let Some(main_window) = handle.get_webview_window("main") {
        let _ = main_window.emit("settings-changed", ());
    }

    Ok(())
}

// Settings commands
#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<AppSettings, EmojiError> {
    state.settings_manager.get()
}

#[tauri::command]
pub async fn update_settings(
    handle: AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), EmojiError> {
    // Sanitize settings: clamp scale factor and max_top_emojis, validate hotkey
    let mut new_settings = settings;

    // Clamp values
    if new_settings.scale_factor < MIN_SCALE_FACTOR {
        new_settings.scale_factor = MIN_SCALE_FACTOR;
    } else if new_settings.scale_factor > MAX_SCALE_FACTOR {
        new_settings.scale_factor = MAX_SCALE_FACTOR;
    }

    if new_settings.max_top_emojis > MAX_TOP_EMOJIS_LIMIT {
        new_settings.max_top_emojis = MAX_TOP_EMOJIS_LIMIT;
    }

    // Validate hotkey string by parsing
    if let Err(e) = crate::hotkey::parse_hotkey(&new_settings.global_hotkey) {
        return Err(EmojiError::InvalidInput(format!(
            "Invalid hotkey '{}': {}",
            new_settings.global_hotkey, e
        )));
    }

    // Check if hotkey has changed
    let old_settings = state.settings_manager.get()?;
    let hotkey_changed = old_settings.global_hotkey != new_settings.global_hotkey;

    if hotkey_changed {
        log::info!(
            "Hotkey changed from '{}' to '{}'",
            old_settings.global_hotkey,
            new_settings.global_hotkey
        );
    }

    state.settings_manager.update(new_settings.clone())?;

    // Notify main window to refresh emoji list if it exists
    if let Some(main_window) = handle.get_webview_window("main") {
        let _ = main_window.emit("settings-changed", ());
    }

    // Re-register hotkey if it changed
    if hotkey_changed {
        log::info!("Hotkey changed, re-registering...");
        if let Err(e) = reregister_hotkey(handle.clone(), state).await {
            log::error!("Failed to re-register hotkey: {}", e);
        }
    }

    Ok(())
}

#[tauri::command]
pub fn open_settings(handle: AppHandle, state: State<'_, AppState>) -> Result<(), EmojiError> {
    // Set flag to indicate we're opening settings
    state
        .opening_settings
        .store(true, std::sync::atomic::Ordering::Release);

    // Open settings window first to ensure UI remains visible; then hide panel
    let result = tray::open_settings_window(&handle).map_err(|e| EmojiError::Tauri(e.to_string()));

    // Clear the flag after settings window is opened/attempted
    state
        .opening_settings
        .store(false, std::sync::atomic::Ordering::Release);

    result
}

#[tauri::command]
pub fn open_help(handle: AppHandle, state: State<'_, AppState>) -> Result<(), EmojiError> {
    // Signal that we're opening help to prevent focus restoration to previous app
    state
        .opening_help
        .store(true, std::sync::atomic::Ordering::Release);

    let result = tray::open_help_window(&handle).map_err(|e| EmojiError::Tauri(e.to_string()));

    // Clear the flag after attempting to open help
    state
        .opening_help
        .store(false, std::sync::atomic::Ordering::Release);

    result
}

#[tauri::command]
pub fn close_help(handle: AppHandle) -> Result<(), EmojiError> {
    if let Some(win) = handle.get_webview_window("help") {
        win.close()
            .map_err(|e| EmojiError::Tauri(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub fn save_window_size(state: State<AppState>, width: f64, height: f64) -> Result<(), EmojiError> {
    state.settings_manager.update_window_size(width, height)
}

#[tauri::command]
pub async fn reregister_hotkey(
    handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), EmojiError> {
    log::info!("Re-registering hotkey...");

    // Make sure panel is hidden before re-registering
    log::debug!("Ensuring panel is hidden before re-registration...");
    let _ = hide_panel(handle.clone());

    // Get the new hotkey from settings
    let settings = state.settings_manager.get()?;
    let new_hotkey_str = settings.global_hotkey.clone();

    // Parse the new hotkey
    let new_shortcut = hotkey::parse_hotkey(&new_hotkey_str).map_err(|e| {
        EmojiError::InvalidInput(format!(
            "Failed to parse hotkey '{}': {}",
            new_hotkey_str, e
        ))
    })?;

    // Unregister ALL shortcuts to ensure clean state
    log::debug!("Unregistering all shortcuts");
    handle
        .global_shortcut()
        .unregister_all()
        .map_err(|e| EmojiError::Tauri(format!("Failed to unregister shortcuts: {}", e)))?;

    // Longer delay to ensure OS processes the unregistration
    log::debug!("Waiting for unregistration to complete...");
    let delay = std::time::Duration::from_millis(HOTKEY_UNREGISTER_WAIT_MS);
    let _ = tauri::async_runtime::spawn_blocking(move || std::thread::sleep(delay)).await;

    // Register the new shortcut (single global handler will handle events)
    log::debug!("Registering new hotkey: {}", new_hotkey_str);
    handle
        .global_shortcut()
        .register(new_shortcut)
        .map_err(|e| EmojiError::Tauri(format!("Failed to register new hotkey: {}", e)))?;

    // Update the stored shortcut
    {
        let mut current = state
            .current_shortcut
            .lock()
            .map_err(|e| EmojiError::Lock(format!("Failed to lock shortcut: {}", e)))?;
        *current = new_shortcut;
    }

    log::info!("Hotkey successfully re-registered to: {}", new_hotkey_str);
    Ok(())
}
