mod command;
mod constants;
pub mod emoji_manager;
mod errors;
mod panel;
mod permissions;
mod positioning;
mod settings;
mod tray;

use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

use crate::emoji_manager::EmojiManager;
use crate::settings::SettingsManager;

/// Application state containing shared resources
#[derive(Debug)]
pub struct AppState {
    pub emoji_manager: Arc<EmojiManager>,
    pub settings_manager: Arc<SettingsManager>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_macos_permissions::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_nspanel::init())
        .invoke_handler(tauri::generate_handler![
            command::show_panel,
            command::hide_panel,
            command::type_emoji,
            command::reset_accessibility_cache,
            command::get_emojis,
            command::get_keywords,
            command::increment_usage,
            command::get_settings,
            command::update_settings,
            command::open_settings,
        ])
        .setup(|app| {
            // Set activation policy to Accessory to prevent the app icon from showing on the dock
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Initialize emoji manager with ranks under Application Support
            let ranks_file_path: PathBuf = {
                let mut dir = app.path().app_data_dir()?;
                if let Err(e) = std::fs::create_dir_all(&dir) {
                    println!("Failed to create Application Support directory: {}", e);
                }
                dir.push(constants::DEFAULT_RANKS_FILE);
                dir
            };

            let emoji_manager = Arc::new(EmojiManager::new(
                PathBuf::from(constants::DEFAULT_EMOJI_FILE),
                ranks_file_path,
            ));
            if let Err(e) = emoji_manager.initialize() {
                println!("Warning: Failed to initialize emoji manager: {}", e);
            }

            // Initialize settings manager with settings file under Application Support
            let settings_file_path: PathBuf = {
                let mut dir = app.path().app_data_dir()?;
                dir.push(constants::DEFAULT_SETTINGS_FILE);
                dir
            };

            let settings_manager = Arc::new(SettingsManager::new(settings_file_path));
            if let Err(e) = settings_manager.initialize() {
                println!("Warning: Failed to initialize settings manager: {}", e);
            }

            // Check accessibility permissions at startup only if needed for the current mode
            let settings_manager_clone = settings_manager.clone();
            tauri::async_runtime::spawn(async move {
                // Check if emoji mode requires accessibility permission
                if let Ok(settings) = settings_manager_clone.get() {
                    let needs_permission = settings.emoji_mode != settings::EmojiMode::CopyOnly;
                    if needs_permission {
                        match permissions::ensure_accessibility_permission().await {
                            Ok(_) => (),
                            Err(e) => {
                                println!("⚠️  Accessibility permission issue: {}", e);
                                println!("   App will work for browsing emojis, but pasting may not work until permission is granted.");
                            }
                        }
                    }
                }
            });

            let app_state = AppState {
                emoji_manager,
                settings_manager,
            };
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
