use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EmojiMode {
    /// Only paste emoji to last focused window on emoji select (requires accessibility permission)
    PasteOnly,
    /// Only copy emoji to clipboard on select (no accessibility permission required)
    CopyOnly,
    /// Both paste to last focused window and copy to clipboard (requires accessibility permission)
    PasteAndCopy,
}

impl Default for EmojiMode {
    fn default() -> Self {
        Self::PasteOnly
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    /// Global hotkey to open the main panel (e.g., "Cmd+Option+Space")
    #[serde(default = "default_global_hotkey")]
    pub global_hotkey: String,
    /// Whether to place the main panel under the mouse cursor when shown
    pub place_under_mouse: bool,
    /// Emoji selection mode
    #[serde(default)]
    pub emoji_mode: EmojiMode,
    /// Last window width (for persistence)
    #[serde(default = "default_window_width")]
    pub window_width: f64,
    /// Last window height (for persistence)
    #[serde(default = "default_window_height")]
    pub window_height: f64,
    /// Maximum number of most used emojis to show first
    #[serde(default = "default_max_top_emojis")]
    pub max_top_emojis: usize,
    /// Scale factor for UI elements (0.5 to 2.0)
    #[serde(default = "default_scale_factor")]
    pub scale_factor: f64,
}

fn default_global_hotkey() -> String {
    "Cmd+Option+Space".to_string()
}

fn default_window_width() -> f64 {
    338.0
}

fn default_window_height() -> f64 {
    290.0
}

fn default_max_top_emojis() -> usize {
    10
}

fn default_scale_factor() -> f64 {
    1.0
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            global_hotkey: default_global_hotkey(),
            place_under_mouse: true,
            emoji_mode: EmojiMode::default(),
            window_width: default_window_width(),
            window_height: default_window_height(),
            max_top_emojis: default_max_top_emojis(),
            scale_factor: default_scale_factor(),
        }
    }
}

/// Manager for application settings with file persistence
#[derive(Debug)]
pub struct SettingsManager {
    settings: Arc<Mutex<Settings>>,
    settings_file_path: PathBuf,
}

impl SettingsManager {
    /// Create a new settings manager with the given file path
    pub fn new(settings_file_path: PathBuf) -> Self {
        Self {
            settings: Arc::new(Mutex::new(Settings::default())),
            settings_file_path,
        }
    }

    /// Initialize settings by loading from file or creating default
    pub fn initialize(&self) -> Result<(), String> {
        if self.settings_file_path.exists() {
            self.load()?;
        } else {
            // Create default settings file
            self.save()?;
        }
        Ok(())
    }

    /// Load settings from file
    fn load(&self) -> Result<(), String> {
        let content = fs::read_to_string(&self.settings_file_path)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;

        let loaded_settings: Settings = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse settings file: {}", e))?;

        let mut settings = self
            .settings
            .lock()
            .map_err(|e| format!("Failed to lock settings: {}", e))?;
        *settings = loaded_settings;

        Ok(())
    }

    /// Save settings to file
    pub fn save(&self) -> Result<(), String> {
        let settings = self
            .settings
            .lock()
            .map_err(|e| format!("Failed to lock settings: {}", e))?;

        let json = serde_json::to_string_pretty(&*settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(&self.settings_file_path, json)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;

        Ok(())
    }

    /// Get current settings
    pub fn get(&self) -> Result<Settings, String> {
        let settings = self
            .settings
            .lock()
            .map_err(|e| format!("Failed to lock settings: {}", e))?;
        Ok(settings.clone())
    }

    /// Update settings
    pub fn update(&self, new_settings: Settings) -> Result<(), String> {
        {
            let mut settings = self
                .settings
                .lock()
                .map_err(|e| format!("Failed to lock settings: {}", e))?;
            *settings = new_settings;
        }
        self.save()?;
        Ok(())
    }

    /// Get a specific setting value
    pub fn get_place_under_mouse(&self) -> Result<bool, String> {
        let settings = self.get()?;
        Ok(settings.place_under_mouse)
    }

    /// Set the place_under_mouse setting
    pub fn set_place_under_mouse(&self, value: bool) -> Result<(), String> {
        let mut settings = self.get()?;
        settings.place_under_mouse = value;
        self.update(settings)?;
        Ok(())
    }

    /// Update window size in settings
    pub fn update_window_size(&self, width: f64, height: f64) -> Result<(), String> {
        let mut settings = self.get()?;
        settings.window_width = width;
        settings.window_height = height;
        self.update(settings)?;
        Ok(())
    }
}
