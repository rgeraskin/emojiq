mod command;
pub mod emoji_manager;
mod panel;
mod permissions;
mod tray;

use tauri::Manager;
use tauri_nspanel::ManagerExt;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

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
        // Register a global shortcut (⌘+K) to toggle the visibility of the spotlight panel
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcut(Shortcut::new(Some(Modifiers::SUPER), Code::KeyK))
                .unwrap()
                .with_handler(|app, shortcut, event| {
                    if event.state == ShortcutState::Pressed
                        && shortcut.matches(Modifiers::SUPER, Code::KeyK)
                    {
                        if let Ok(panel) = app.app_handle().get_webview_panel("main") {
                            if panel.is_visible() {
                                panel.hide();
                            } else {
                                panel.show_and_make_key();
                            }
                        } else {
                            eprintln!("Failed to get main panel");
                        }
                    }
                })
                .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
