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
            can_become_main_window: false,
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

    // Prevent panel from activating the app (required for fullscreen display)
    panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());

    // Allow panel to display over fullscreen windows and join all spaces
    panel.set_collection_behavior(
        CollectionBehavior::new()
            .full_screen_auxiliary()
            .can_join_all_spaces()
            .into(),
    );

    panel.set_corner_radius(12.0); // without it - panel edges are not rounded, only window edges are rounded

    // panel.set_transparent(true); // works without it
    // panel.set_works_when_modal(true); // hz?
    // panel.set_level(PanelLevel::Floating.into()); // hz?
    // panel.set_movable_by_window_background(true); // not working

    // Ensures the panel cannot activate the App
    // panel.set_style_mask(NSWindowStyleMaskNonActivatingPanel); // should be adapted but hz why?

    // Print panel info
    println!("Panel class name: {:?}", panel.as_panel().class().name());
    println!("Panel can become key?: {}", panel.can_become_key_window());
    println!("Panel can become main?: {}", panel.can_become_main_window());
    println!("Panel is floating?: {}", panel.is_floating_panel());

    // Create and attach event handler
    let handler = MiniPanelEventHandler::new();

    let panel_for_handler = panel.clone();
    handler.window_did_resign_key(move |_notification| {
        panel_for_handler.hide();
    });

    panel.set_event_handler(Some(handler.as_ref()));

    Ok(())
}
