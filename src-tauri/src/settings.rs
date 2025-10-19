use crate::constants::{
    DEFAULT_GLOBAL_HOTKEY, DEFAULT_MAX_TOP_EMOJIS, DEFAULT_PLACE_UNDER_MOUSE, DEFAULT_SCALE_FACTOR,
    DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH,
};
use crate::errors::EmojiError;
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
    #[serde(default = "default_place_under_mouse")]
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
    DEFAULT_GLOBAL_HOTKEY.to_string()
}

fn default_window_width() -> f64 {
    DEFAULT_WINDOW_WIDTH
}

fn default_window_height() -> f64 {
    DEFAULT_WINDOW_HEIGHT
}

fn default_max_top_emojis() -> usize {
    DEFAULT_MAX_TOP_EMOJIS
}

fn default_scale_factor() -> f64 {
    DEFAULT_SCALE_FACTOR
}

fn default_place_under_mouse() -> bool {
    DEFAULT_PLACE_UNDER_MOUSE
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            global_hotkey: default_global_hotkey(),
            place_under_mouse: default_place_under_mouse(),
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
    pub fn initialize(&self) -> Result<(), EmojiError> {
        if self.settings_file_path.exists() {
            self.load()?;
        } else {
            // Create default settings file
            self.save()?;
        }
        Ok(())
    }

    /// Load settings from file
    fn load(&self) -> Result<(), EmojiError> {
        let content = fs::read_to_string(&self.settings_file_path)?;
        let loaded_settings: Settings = serde_json::from_str(&content)?;

        let mut settings = self
            .settings
            .lock()
            .map_err(|e| EmojiError::Lock(format!("Failed to lock settings: {}", e)))?;
        *settings = loaded_settings;

        Ok(())
    }

    /// Save settings to file
    pub fn save(&self) -> Result<(), EmojiError> {
        let settings = self
            .settings
            .lock()
            .map_err(|e| EmojiError::Lock(format!("Failed to lock settings: {}", e)))?;

        let json = serde_json::to_string_pretty(&*settings)?;
        fs::write(&self.settings_file_path, json)?;

        Ok(())
    }

    /// Get current settings
    pub fn get(&self) -> Result<Settings, EmojiError> {
        let settings = self
            .settings
            .lock()
            .map_err(|e| EmojiError::Lock(format!("Failed to lock settings: {}", e)))?;
        Ok(settings.clone())
    }

    /// Update settings
    pub fn update(&self, new_settings: Settings) -> Result<(), EmojiError> {
        {
            let mut settings = self
                .settings
                .lock()
                .map_err(|e| EmojiError::Lock(format!("Failed to lock settings: {}", e)))?;
            *settings = new_settings;
        }
        self.save()?;
        Ok(())
    }

    /// Get a specific setting value
    pub fn get_place_under_mouse(&self) -> Result<bool, EmojiError> {
        let settings = self.get()?;
        Ok(settings.place_under_mouse)
    }

    /// Set the place_under_mouse setting
    pub fn set_place_under_mouse(&self, value: bool) -> Result<(), EmojiError> {
        let mut settings = self.get()?;
        settings.place_under_mouse = value;
        self.update(settings)?;
        Ok(())
    }

    /// Update window size in settings
    pub fn update_window_size(&self, width: f64, height: f64) -> Result<(), EmojiError> {
        let mut settings = self.get()?;
        settings.window_width = width;
        settings.window_height = height;
        self.update(settings)?;
        Ok(())
    }
}
