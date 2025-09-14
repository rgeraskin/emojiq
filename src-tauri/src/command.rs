use enigo::{Enigo, Keyboard, Settings};
use serde::{Deserialize, Serialize};
use std::thread;
use std::time::Duration;
use tauri::AppHandle;
use tauri_nspanel::ManagerExt;
use tauri_plugin_macos_permissions::{
    check_accessibility_permission, request_accessibility_permission,
};

#[derive(Deserialize, Serialize)]
struct EmojiData {
    emoji: String,
    description: String,
    category: String,
    aliases: Vec<String>,
    tags: Vec<String>,
    unicode_version: String,
    ios_version: String,
}

// #[tauri::command]
// pub fn get_emojis() -> Result<Vec<String>, String> {
//     // Read the emoji.json file from the src-python directory
//     let emoji_file_path = "src-python/emoji.json";

//     match fs::read_to_string(emoji_file_path) {
//         Ok(contents) => match serde_json::from_str::<Vec<EmojiData>>(&contents) {
//             Ok(emoji_data) => {
//                 let emojis: Vec<String> = emoji_data.into_iter().map(|emoji| emoji.emoji).collect();
//                 Ok(emojis)
//             }
//             Err(e) => Err(format!("Failed to parse JSON: {}", e)),
//         },
//         Err(e) => Err(format!("Failed to read emoji file: {}", e)),
//     }
// }

#[tauri::command]
pub fn show_panel(handle: AppHandle) {
    let panel = handle.get_webview_panel("main").unwrap();
    panel.show_and_make_key();
}

#[tauri::command]
pub fn hide_panel(handle: AppHandle) {
    let panel = handle.get_webview_panel("main").unwrap();
    panel.hide();
}

#[tauri::command]
pub fn close_panel(app_handle: AppHandle) {
    app_handle
        .get_webview_panel("main")
        .ok()
        .and_then(|panel| panel.to_window())
        .map(|window| window.close());
}

#[tauri::command]
pub async fn type_emoji(_: AppHandle, emoji: String) -> Result<(), String> {
    // request accessibility permission
    let authorized = check_accessibility_permission().await;
    if !authorized {
        println!("requesting accessibility permission");
        // request_accessibility_permission().await;
    } else {
        println!("accessibility permission already granted");
    }

    thread::sleep(Duration::from_millis(200));
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    // write text
    enigo.text(&emoji).unwrap();

    Ok(())
}
