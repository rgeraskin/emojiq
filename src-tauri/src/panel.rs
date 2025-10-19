#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use crate::constants::*;
use crate::errors::EmojiError;
use crate::positioning::{position_window_at_cursor, restore_previous_app, store_previous_app};
use tauri::{AppHandle, Manager, WebviewWindow};
use tauri_nspanel::{tauri_panel, CollectionBehavior, ManagerExt, StyleMask, WebviewWindowExt};

/// Helper function to check if settings window is visible and focus it if so
/// Returns true if settings was visible and focused
fn try_focus_settings(handle: &AppHandle) -> bool {
    if let Some(window) = handle.get_webview_window("settings") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.set_focus();
            return true;
        }
    }
    false
}

// Define custom panel class and event handler
tauri_panel! {
    panel!(EmojiqPanel {
        config: {
            can_become_key_window: true,
            can_become_main_window: true,
            is_floating_panel: true
        }
    })

    panel_event!(MiniPanelEventHandler {
        window_did_become_key(notification: &NSNotification) -> (),
        window_did_resign_key(notification: &NSNotification) -> ()
    })
}

pub fn init(app_handle: &AppHandle) -> tauri::Result<()> {
    let window: WebviewWindow = app_handle
        .get_webview_window("main")
        .ok_or_else(|| tauri::Error::WindowNotFound)?;

    // Restore window size from settings
    if let Some(state) = app_handle.try_state::<crate::AppState>() {
        if let Ok(settings) = state.settings_manager.get() {
            let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
                width: settings.window_width,
                height: settings.window_height,
            }));
        }
    }

    let panel = window.to_panel::<EmojiqPanel>().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to convert to panel: {:?}", e),
        )
    })?;
    let _ = hide_panel(app_handle.clone());

    // Prevent panel from activating the app (required for fullscreen display)
    panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());

    // Allow panel to display over fullscreen windows and join all spaces
    panel.set_collection_behavior(
        CollectionBehavior::new()
            .full_screen_auxiliary()
            .can_join_all_spaces()
            .into(),
    );

    // without it - panel edges are not rounded, only window edges are rounded
    panel.set_corner_radius(PANEL_CORNER_RADIUS);

    // panel.set_transparent(true); // works without it
    // panel.set_works_when_modal(true); // why?
    // panel.set_level(PanelLevel::Floating.into()); // why?
    // panel.set_movable_by_window_background(true); // not working

    // Ensures the panel cannot activate the App
    // panel.set_style_mask(NSWindowStyleMaskNonActivatingPanel); // should be adapted but why?

    // Create and attach event handler (panel retains the handler internally)
    let handler = MiniPanelEventHandler::new();

    // Handle focus loss - hide panel and restore nonactivating_panel style
    let panel_for_handler = panel.clone();
    let handle_for_handler = app_handle.clone();
    handler.window_did_resign_key(move |_notification| {
        log::debug!("Panel lost focus, hiding panel");
        panel_for_handler.hide();
        // Restore nonactivating_panel for fullscreen compatibility
        panel_for_handler.set_style_mask(StyleMask::empty().nonactivating_panel().into());

        // Check if we're opening settings - if so, don't restore focus yet
        if let Some(state) = handle_for_handler.try_state::<crate::AppState>() {
            if state
                .opening_settings
                .load(std::sync::atomic::Ordering::Acquire)
            {
                return;
            }
        }

        // Try to focus settings window if it's open, otherwise restore previous app
        if !try_focus_settings(&handle_for_handler) {
            restore_previous_app();
        }
    });

    panel.set_event_handler(Some(handler.as_ref()));

    Ok(())
}

pub fn hide_panel(handle: AppHandle) -> Result<(), EmojiError> {
    let panel = handle
        .get_webview_panel("main")
        .map_err(|e| EmojiError::Panel(format!("Failed to get main panel: {:?}", e)))?;

    if panel.is_visible() {
        log::debug!("Panel is visible, hiding panel via command");
        panel.hide();
    } else {
        log::warn!("Panel is already hidden, why are we trying to hide it?");
    }
    Ok(())
}

pub fn show_panel(handle: AppHandle) -> Result<(), EmojiError> {
    // Check if settings window is currently open and focused
    // If so, don't store the previous app - we'll return focus to settings instead
    let settings_is_open = handle
        .get_webview_window("settings")
        .map(|w| w.is_visible().unwrap_or(false) && w.is_focused().unwrap_or(false))
        .unwrap_or(false);

    // Only store the previous app if settings window is not currently focused
    if !settings_is_open {
        store_previous_app();
    }

    // Get the window first, then convert to panel (more reliable)
    if let Some(window) = handle.get_webview_window("main") {
        // Check settings to determine if we should position at cursor
        let should_position_at_cursor = {
            use tauri::Manager;
            match handle.try_state::<crate::AppState>() {
                Some(state) => state
                    .settings_manager
                    .get_place_under_mouse()
                    .unwrap_or(true),
                None => true, // Default to true if we can't get state
            }
        };

        // Position panel BEFORE showing the panel (if enabled)
        if should_position_at_cursor {
            if let Err(e) = position_window_at_cursor(&window) {
                log::warn!(
                    "Failed to position panel at cursor: {}. Using default positioning.",
                    e
                );
            }
        } else {
            log::debug!("Positioning at cursor disabled, using default positioning");
        }

        // Show panel after positioning is complete
        let panel = handle
            .get_webview_panel("main")
            .map_err(|e| EmojiError::Panel(format!("Failed to get main panel: {:?}", e)))?;
        panel.show_and_make_key();

        Ok(())
    } else {
        Err(EmojiError::Panel("Failed to get main window".to_string()))
    }
}

pub fn toggle_panel(handle: AppHandle) -> Result<(), EmojiError> {
    let panel = handle
        .get_webview_panel("main")
        .map_err(|e| EmojiError::Panel(format!("Failed to get main panel: {:?}", e)))?;

    let is_visible = panel.is_visible();
    log::debug!("toggle_panel called, panel is_visible: {}", is_visible);

    if is_visible {
        let _ = hide_panel(handle);
    } else {
        let _ = show_panel(handle);
    }
    Ok(())
}
