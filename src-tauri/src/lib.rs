mod command;
pub mod emoji_manager;
mod panel;
mod permissions;
mod positioning;
mod tray;

use tauri::Manager;

use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_macos_permissions::init())
        .plugin(tauri_nspanel::init())
        .invoke_handler(tauri::generate_handler![
            command::show_panel,
            command::hide_panel,
            command::type_emoji,
            command::reset_accessibility_cache,
            command::get_emojis,
            command::get_keywords,
            command::increment_usage,
        ])
        .setup(|app| {
            // Set activation policy to Accessory to prevent the app icon from showing on the dock
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Check accessibility permissions at startup
            tauri::async_runtime::spawn(async {
                match permissions::ensure_accessibility_permission().await {
                    Ok(_) => (),
                    Err(e) => {
                        println!("⚠️  Accessibility permission issue: {}", e);
                        println!("   App will work for browsing emojis, but pasting may not work until permission is granted.");
                    }
                }
            });

            // Initialize emoji manager
            if let Err(e) = emoji_manager::initialize_global_manager() {
                println!("Warning: Failed to initialize emoji manager: {}", e);
            }

            panel::init(app.app_handle())?;
            tray::init(app.app_handle())?;

            Ok(())
        })
        // Register global shortcuts (only Cmd+K, ESC will be registered dynamically)
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcut(Shortcut::new(Some(Modifiers::SUPER), Code::KeyK))
                .unwrap()
                .with_handler(|app, shortcut, event| {
                    let handle = app.app_handle();
                    // Handle Cmd+K to toggle panel
                    if event.state == ShortcutState::Pressed
                        && shortcut.matches(Modifiers::SUPER, Code::KeyK)
                    {
                        let _ = panel::toggle_panel(handle.clone());
                            // let panel = match handle.get_webview_panel("main") {
                            //     Ok(panel) => panel,
                            //     Err(e) => {
                            //         eprintln!("Failed to get main panel: {:?}", e);
                            //         return;
                            //     }
                            // };

                            // if panel.is_visible() {
                            //     let _ = command::hide_panel(handle.clone());
                            //     // Restore nonactivating_panel for fullscreen compatibility
                            //     // panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());
                            // } else {
                            //     let _ = command::show_panel(handle.clone());
                            //     // // Store the currently active application before showing our panel
                            //     // store_previous_app();

                            //     // // Position panel BEFORE showing to prevent visible redrawing
                            //     // if let Err(e) = positioning::position_window_at_cursor(&window) {
                            //     //     println!("Warning: Failed to position panel at cursor: {}. Using default positioning.", e);
                            //     // }
                            //     // println!("Skipping style mask change to avoid panic...");
                            //     // // Skip style mask change - it's causing the panic
                            //     // // panel.set_style_mask(StyleMask::empty().titled().closable().into());
                            //     // println!("Style mask change skipped");

                            //     // println!("Showing panel...");
                            //     // // Show panel after positioning is complete
                            //     // panel.show_and_make_key();
                            //     // println!("Panel shown successfully");

                            //     // println!("Making panel key and order front...");
                            //     // // Multiple attempts to ensure focus
                            //     // panel.make_key_and_order_front();
                            //     // println!("make_key_and_order_front completed");

                            //     // println!("Setting focus...");
                            //     // // Set focus to allow ESC key detection
                            //     // match window.set_focus() {
                            //     //     Ok(_) => println!("Successfully set focus on window for ESC detection"),
                            //     //     Err(e) => println!("Failed to set focus on window: {:?}", e),
                            //     // }

                            //     // // Debug focus state
                            //     // match window.is_focused() {
                            //     //     Ok(focused) => println!("Window focus state: {}", focused),
                            //     //     Err(e) => println!("Failed to check focus state: {:?}", e),
                            //     // }

                            //     // println!("Final make_key_and_order_front...");
                            //     // // Try to force visual focus by making it the key window again
                            //     // panel.make_key_and_order_front();
                            //     // println!("Final make_key_and_order_front completed");

                            //     // println!("Panel showing completed successfully!");

                            //     // // ESC will be handled via JavaScript since webview now has focus
                            // }
                    }
                })
                .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
