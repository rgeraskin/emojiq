use crate::emoji_manager;
use crate::permissions::{ensure_accessibility_permission, reset_permission_cache};
use crate::positioning;
use crate::{restore_previous_app, store_previous_app};
use enigo::{Enigo, Keyboard, Settings};
use tauri::{AppHandle, Manager};
use tauri_nspanel::{ManagerExt, StyleMask, WebviewWindowExt};

#[tauri::command]
pub fn show_panel(handle: AppHandle) -> Result<(), String> {
    // Store the currently active application before showing our panel
    store_previous_app();

    // Get the window first, then convert to panel (more reliable)
    if let Some(window) = handle.get_webview_window("main") {
        // Position panel BEFORE converting to panel and showing to prevent visible redrawing
        if let Err(e) = positioning::position_window_at_cursor(&window) {
            println!(
                "Warning: Failed to position panel at cursor: {}. Using default positioning.",
                e
            );
        }

        let panel = match window.to_panel::<crate::panel::EmojiqPanel>() {
            Ok(panel) => panel,
            Err(e) => return Err(format!("Failed to convert window to panel: {:?}", e)),
        };

        // Skip style mask change to avoid panic
        // panel.set_style_mask(StyleMask::empty().titled().closable().into());

        // Show panel after positioning is complete
        panel.show_and_make_key();

        // Multiple attempts to ensure focus
        panel.make_key_and_order_front();

        // Force the panel to become key and focused
        panel.make_key_and_order_front();

        // Set focus to allow ESC key detection
        match window.set_focus() {
            Ok(_) => println!("Successfully set focus on window for ESC detection"),
            Err(e) => println!("Failed to set focus on window: {:?}", e),
        }

        // Debug focus state
        match window.is_focused() {
            Ok(focused) => println!("Window focus state: {}", focused),
            Err(e) => println!("Failed to check focus state: {:?}", e),
        }

        // Try to force visual focus by making it the key window again
        panel.make_key_and_order_front();

        Ok(())
    } else {
        Err("Failed to get main window".to_string())
    }
}

#[tauri::command]
pub fn hide_panel(handle: AppHandle) -> Result<(), String> {
    // Get the window first, then convert to panel (more reliable)
    if let Some(window) = handle.get_webview_window("main") {
        let panel = match window.to_panel::<crate::panel::EmojiqPanel>() {
            Ok(panel) => panel,
            Err(e) => return Err(format!("Failed to convert window to panel: {:?}", e)),
        };
        if panel.is_visible() {
            println!("Hiding panel via command");
            panel.hide();
            // Restore nonactivating_panel for fullscreen compatibility
            panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());

            // Restore focus to the previously active application
            restore_previous_app();
        } else {
            println!("Panel is already hidden");
        }
        Ok(())
    } else {
        Err("Failed to get main window".to_string())
    }
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

    // Restore focus to the previously active application
    restore_previous_app();

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
