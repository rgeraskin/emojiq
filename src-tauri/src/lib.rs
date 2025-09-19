mod command;
pub mod emoji_manager;
mod panel;
mod permissions;
mod positioning;
mod tray;

use std::sync::{Arc, Mutex};
use tauri::Manager;

use tauri_nspanel::{ManagerExt, StyleMask, WebviewWindowExt};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

// Global storage for the previously active application
lazy_static::lazy_static! {
    static ref PREVIOUS_APP: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_macos_permissions::init())
        .plugin(tauri_nspanel::init())
        .invoke_handler(tauri::generate_handler![
            command::show_panel,
            command::hide_panel,
            command::close_panel,
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
                    // Handle Cmd+K to toggle panel
                    if event.state == ShortcutState::Pressed
                        && shortcut.matches(Modifiers::SUPER, Code::KeyK)
                    {
                        // Get the window first, then convert to panel (more reliable)
                        if let Some(window) = app.app_handle().get_webview_window("main") {
                            if let Ok(panel) = window.to_panel::<panel::EmojiqPanel>() {
                            if panel.is_visible() {
                                panel.hide();
                                // Restore nonactivating_panel for fullscreen compatibility
                                panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());
                            } else {
                                // Store the currently active application before showing our panel
                                store_previous_app();

                                // Position panel BEFORE showing to prevent visible redrawing
                                if let Err(e) = positioning::position_window_at_cursor(&window) {
                                    println!("Warning: Failed to position panel at cursor: {}. Using default positioning.", e);
                                }
                                println!("Skipping style mask change to avoid panic...");
                                // Skip style mask change - it's causing the panic
                                // panel.set_style_mask(StyleMask::empty().titled().closable().into());
                                println!("Style mask change skipped");

                                println!("Showing panel...");
                                // Show panel after positioning is complete
                                panel.show_and_make_key();
                                println!("Panel shown successfully");

                                println!("Making panel key and order front...");
                                // Multiple attempts to ensure focus
                                panel.make_key_and_order_front();
                                println!("make_key_and_order_front completed");

                                println!("Setting focus...");
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

                                println!("Final make_key_and_order_front...");
                                // Try to force visual focus by making it the key window again
                                panel.make_key_and_order_front();
                                println!("Final make_key_and_order_front completed");

                                println!("Panel showing completed successfully!");

                                // ESC will be handled via JavaScript since webview now has focus
                            }
                            } else {
                                eprintln!("Failed to convert window to panel: {:?}", "conversion error");
                            }
                        } else {
                            eprintln!("Failed to get main window");
                        }
                    }
                })
                .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Function to store the currently active application
#[cfg(target_os = "macos")]
pub fn store_previous_app() {
    // Use AppleScript to get the frontmost application
    use std::process::Command;

    let output = Command::new("osascript")
        .arg("-e")
        .arg("tell application \"System Events\" to get bundle identifier of first application process whose frontmost is true")
        .output();

    if let Ok(output) = output {
        if let Ok(bundle_id) = String::from_utf8(output.stdout) {
            let bundle_id = bundle_id.trim().to_string();
            if !bundle_id.is_empty() && bundle_id != "com.emojiq.app" {
                if let Ok(mut previous_app) = PREVIOUS_APP.lock() {
                    *previous_app = Some(bundle_id.clone());
                    println!("Stored previous app: {}", bundle_id);
                }
            }
        }
    }
}

// Function to restore focus to the previously active application
#[cfg(target_os = "macos")]
pub fn restore_previous_app() {
    if let Ok(previous_app) = PREVIOUS_APP.lock() {
        if let Some(bundle_id) = previous_app.as_ref() {
            println!("Restoring focus to: {}", bundle_id);

            // Use AppleScript to activate the application
            use std::process::Command;

            let script = format!("tell application id \"{}\" to activate", bundle_id);
            let _output = Command::new("osascript").arg("-e").arg(&script).output();
        } else {
            println!("No previous app stored");
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn store_previous_app() {
    // No-op for non-macOS platforms
}

#[cfg(not(target_os = "macos"))]
pub fn restore_previous_app() {
    // No-op for non-macOS platforms
}
