#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use crate::constants::*;
use crate::errors::EmojiError;
use crate::positioning::{position_window_at_cursor, restore_previous_app, store_previous_app};
use tauri::{AppHandle, Manager, WebviewWindow};
use tauri_nspanel::{tauri_panel, CollectionBehavior, ManagerExt, StyleMask, WebviewWindowExt};

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
    let window: WebviewWindow = app_handle.get_webview_window("main").unwrap();
    let panel = window.to_panel::<EmojiqPanel>().unwrap();
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
    handler.window_did_resign_key(move |_notification| {
        println!("Panel lost focus, hiding panel");
        panel_for_handler.hide();
        // Restore nonactivating_panel for fullscreen compatibility
        panel_for_handler.set_style_mask(StyleMask::empty().nonactivating_panel().into());

        // No need to unregister ESC shortcut since we're not using global ESC
    });

    panel.set_event_handler(Some(handler.as_ref()));

    Ok(())
}

pub fn hide_panel(handle: AppHandle) -> Result<(), String> {
    let panel = handle
        .get_webview_panel("main")
        .map_err(|e| EmojiError::Panel(format!("Failed to get main panel: {:?}", e)).to_string())?;

    if panel.is_visible() {
        println!("Panel is visible, hiding panel via command");
        panel.hide();

        // Restore focus to the previously active application
        restore_previous_app();
    } else {
        println!("Panel is already hidden, why are we trying to hide it?");
    }
    Ok(())
}

pub fn show_panel(handle: AppHandle) -> Result<(), String> {
    // Store the currently active application before showing our panel
    store_previous_app();

    // Get the window first, then convert to panel (more reliable)
    if let Some(window) = handle.get_webview_window("main") {
        // Position panel BEFORE showing the panel
        if let Err(e) = position_window_at_cursor(&window) {
            println!(
                "Warning: Failed to position panel at cursor: {}. Using default positioning.",
                e
            );
        }

        // Show panel after positioning is complete
        let panel = handle.get_webview_panel("main").map_err(|e| {
            EmojiError::Panel(format!("Failed to get main panel: {:?}", e)).to_string()
        })?;
        panel.show_and_make_key();

        // Debug focus state
        // match window.is_focused() {
        //     Ok(focused) => println!("Window focus state: {}", focused),
        //     Err(e) => println!("Failed to check focus state: {:?}", e),
        // }

        Ok(())
    } else {
        Err(EmojiError::Panel("Failed to get main window".to_string()).to_string())
    }
}

pub fn toggle_panel(handle: AppHandle) -> Result<(), String> {
    let panel = handle
        .get_webview_panel("main")
        .map_err(|e| EmojiError::Panel(format!("Failed to get main panel: {:?}", e)).to_string())?;

    if panel.is_visible() {
        let _ = hide_panel(handle);
    } else {
        let _ = show_panel(handle);
    }
    Ok(())
}
