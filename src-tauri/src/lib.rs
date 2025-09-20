mod command;
mod constants;
pub mod emoji_manager;
mod panel;
mod permissions;
mod positioning;
mod tray;

use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

use crate::emoji_manager::EmojiManager;

/// Application state containing shared resources
#[derive(Debug)]
pub struct AppState {
    pub emoji_manager: Arc<EmojiManager>,
}

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

            // Initialize emoji manager and add to app state
            let emoji_manager = Arc::new(EmojiManager::default());
            if let Err(e) = emoji_manager.initialize() {
                println!("Warning: Failed to initialize emoji manager: {}", e);
            }

            let app_state = AppState { emoji_manager };
            app.manage(app_state);

            panel::init(app.app_handle())?;
            tray::init(app.app_handle())?;

            Ok(())
        })
        // Register global shortcuts (only Option+Cmd+Space, ESC will be registered dynamically)
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcut(Shortcut::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::Space))
                .unwrap()
                .with_handler(|app, shortcut, event| {
                    let handle = app.app_handle();
                    // Handle Option+Cmd+Space to toggle panel
                    if event.state == ShortcutState::Pressed
                        && shortcut.matches(Modifiers::SUPER | Modifiers::ALT, Code::Space)
                    {
                        let _ = panel::toggle_panel(handle.clone());
                    }
                })
                .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
