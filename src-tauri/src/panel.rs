#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::{AppHandle, Manager, WebviewWindow};
use tauri_nspanel::{tauri_panel, CollectionBehavior, StyleMask, WebviewWindowExt};

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
    panel.hide();

    // Start with normal style mask to allow focus
    // We'll apply nonactivating_panel only when hiding for fullscreen compatibility
    panel.set_style_mask(StyleMask::empty().into());

    // Allow panel to display over fullscreen windows and join all spaces
    panel.set_collection_behavior(
        CollectionBehavior::new()
            .full_screen_auxiliary()
            .can_join_all_spaces()
            .into(),
    );

    panel.set_corner_radius(12.0); // without it - panel edges are not rounded, only window edges are rounded

    // panel.set_transparent(true); // works without it
    // panel.set_works_when_modal(true); // why?
    // panel.set_level(PanelLevel::Floating.into()); // why?
    // panel.set_movable_by_window_background(true); // not working

    // Ensures the panel cannot activate the App
    // panel.set_style_mask(NSWindowStyleMaskNonActivatingPanel); // should be adapted but why?

    // Create and attach event handler
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
