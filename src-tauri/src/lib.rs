mod command;
pub mod constants;
pub mod emoji_manager;
mod errors;
mod hotkey;
mod panel;
mod permissions;
mod positioning;
mod settings;
mod tray;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
// time utilities not needed here anymore
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::emoji_manager::EmojiManager;
use crate::settings::SettingsManager;

/// Application state containing shared resources
#[derive(Debug)]
pub struct AppState {
    pub emoji_manager: Arc<EmojiManager>,
    pub settings_manager: Arc<SettingsManager>,
    pub opening_settings: Arc<AtomicBool>,
    pub opening_help: Arc<AtomicBool>,
    pub current_shortcut: Arc<Mutex<Shortcut>>,
    pub shortcut_pressed: Arc<AtomicBool>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .filter_module("enigo", log::LevelFilter::Warn)
        .init();

    log::info!("Starting EmojiQ application");

    // Use default shortcut initially; will re-register after SettingsManager loads
    let default_hotkey = constants::DEFAULT_GLOBAL_HOTKEY.to_string();
    let shortcut = match hotkey::parse_hotkey(&default_hotkey) {
        Ok(s) => s,
        Err(e) => {
            log::warn!(
                "Failed to parse default hotkey '{}': {}. Fallback to default.",
                default_hotkey,
                e
            );
            hotkey::parse_hotkey(constants::DEFAULT_GLOBAL_HOTKEY).unwrap()
        }
    };

    log::info!(
        "Registering global hotkey: {}",
        constants::DEFAULT_GLOBAL_HOTKEY
    );

    tauri::Builder::default()
        .plugin(tauri_plugin_macos_permissions::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_nspanel::init())
        // Initialize global shortcut plugin FIRST with a single global handler
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if let Some(state) = app.try_state::<crate::AppState>() {
                        match event.state {
                            ShortcutState::Pressed => {
                                state.shortcut_pressed.store(true, Ordering::Relaxed);
                            }
                            ShortcutState::Released => {
                                let was_pressed = state
                                    .shortcut_pressed
                                    .swap(false, Ordering::Relaxed);
                                if !was_pressed {
                                    log::warn!("Global handler: Ignoring duplicate release");
                                    return;
                                }
                                let handle = app.app_handle();
                                let _ = panel::toggle_panel(handle.clone());
                            }
                        }
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            command::show_panel,
            command::hide_panel,
            command::type_emoji,
            command::reset_accessibility_cache,
            command::get_emojis,
            command::get_keywords,
            command::increment_usage,
            command::remove_emoji_rank,
            command::reset_emoji_ranks,
            command::get_settings,
            command::update_settings,
            command::open_settings,
            command::save_window_size,
            command::reregister_hotkey,
            command::open_help,
            command::close_help,
        ])
        .setup(move |app| {
            // Set activation policy to Accessory to prevent the app icon from showing on the dock
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Initialize emoji manager with ranks under Application Support
            let ranks_file_path: PathBuf = {
                let mut dir = app.path().app_data_dir()?;
                if let Err(e) = std::fs::create_dir_all(&dir) {
                    log::error!("Failed to create Application Support directory: {}", e);
                }
                dir.push(constants::DEFAULT_RANKS_FILE);
                dir
            };

            let emoji_manager = Arc::new(EmojiManager::new(
                PathBuf::from(constants::DEFAULT_EMOJI_FILE),
                ranks_file_path,
            ));
            if let Err(e) = emoji_manager.initialize() {
                log::warn!("Failed to initialize emoji manager: {}", e);
            }

            // Initialize settings manager with settings file under Application Support
            let settings_file_path: PathBuf = {
                let mut dir = app.path().app_data_dir()?;
                dir.push(constants::DEFAULT_SETTINGS_FILE);
                dir
            };

            // Determine if this is the first launch (settings file absent)
            let first_launch = !settings_file_path.exists();

            let settings_manager = Arc::new(SettingsManager::new(settings_file_path));
            if let Err(e) = settings_manager.initialize() {
                log::warn!("Failed to initialize settings manager: {}", e);
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
                                log::warn!("Accessibility permission issue: {}", e);
                                log::info!("App will work for browsing emojis, but pasting may not work until permission is granted.");
                            }
                        }
                    }
                }
            });

            let app_state = AppState {
                emoji_manager,
                settings_manager,
                opening_settings: Arc::new(AtomicBool::new(false)),
                opening_help: Arc::new(AtomicBool::new(false)),
                current_shortcut: Arc::new(Mutex::new(shortcut.clone())),
                shortcut_pressed: Arc::new(AtomicBool::new(false)),
            };
            app.manage(app_state);

            let app_handle = app.app_handle();

            panel::init(&app_handle)?;
            tray::init(&app_handle)?;

            // Register initial global shortcut (single central handler already set by plugin)
            if let Err(e) = app_handle.global_shortcut().register(shortcut.clone()) {
                log::error!("Failed to register initial hotkey: {}", e);
            }

            // Show Help window on first launch
            if first_launch {
                if let Err(e) = tray::open_help_window(&app_handle) {
                    log::error!("Failed to open Help window on first launch: {}", e);
                }
            }

            // After settings manager has loaded, re-register to saved hotkey if different
            {
                let handle_clone = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    // Read desired hotkey from settings
                    if let Some(state) = handle_clone.try_state::<crate::AppState>() {
                        if let Ok(settings) = state.settings_manager.get() {
                            if settings.global_hotkey != constants::DEFAULT_GLOBAL_HOTKEY {
                                // Parse new shortcut
                                if let Ok(new_shortcut) = crate::hotkey::parse_hotkey(&settings.global_hotkey) {
                                    // Unregister all, wait, register new
                                    if let Err(e) = handle_clone.global_shortcut().unregister_all() {
                                        log::error!("Failed to unregister shortcuts: {}", e);
                                        return;
                                    }
                                    let delay = std::time::Duration::from_millis(
                                        crate::constants::HOTKEY_UNREGISTER_WAIT_MS,
                                    );
                                    let _ = tauri::async_runtime::spawn_blocking(move || std::thread::sleep(delay)).await;
                                    if let Err(e) = handle_clone.global_shortcut().register(new_shortcut.clone()) {
                                        log::error!("Failed to register saved hotkey: {}", e);
                                        return;
                                    }
                                    if let Ok(mut guard) = state.current_shortcut.lock() {
                                        *guard = new_shortcut;
                                    }
                                    log::info!(
                                        "Hotkey re-registered to saved setting: {}",
                                        settings.global_hotkey
                                    );
                                }
                            }
                        }
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
