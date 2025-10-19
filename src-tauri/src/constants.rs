use std::time::Duration;

// Timing constants
pub const FOCUS_RESTORATION_DELAY_MS: u64 = 200;
pub const RANK_WRITE_DELAY_SECS: u64 = 2;
pub const SETTINGS_HIDE_BEFORE_OPEN_MS: u64 = 100;
pub const HOTKEY_UNREGISTER_WAIT_MS: u64 = 300;
// pub const ASYNC_WRITE_CHECK_DELAY_MS: u64 = 100; // Currently unused

// Search constants
pub const MIN_SEARCH_LENGTH: usize = 2;
pub const MAX_SEARCH_RESULTS: usize = 2000;
pub const MIN_KEYWORD_LENGTH: usize = 2;
pub const MAX_PREFIX_LENGTH: usize = 12; // Cap for prefix indexing to bound memory

// UI constants
pub const PANEL_CORNER_RADIUS: f64 = 12.0;

// File constants
pub const DEFAULT_EMOJI_FILE: &str = "src/emoji.json";
pub const DEFAULT_RANKS_FILE: &str = "ranks.json";
pub const DEFAULT_SETTINGS_FILE: &str = "settings.json";

// Settings defaults and limits
pub const DEFAULT_GLOBAL_HOTKEY: &str = "Cmd+Option+Space";
pub const DEFAULT_PLACE_UNDER_MOUSE: bool = true;
pub const DEFAULT_WINDOW_WIDTH: f64 = 338.0;
pub const DEFAULT_WINDOW_HEIGHT: f64 = 290.0;
pub const DEFAULT_MAX_TOP_EMOJIS: usize = 10;
pub const DEFAULT_SCALE_FACTOR: f64 = 1.0;

// Settings window dimensions
pub const SETTINGS_WINDOW_WIDTH: f64 = 400.0;
pub const SETTINGS_WINDOW_HEIGHT: f64 = 755.0;

pub const MIN_SCALE_FACTOR: f64 = 0.5;
pub const MAX_SCALE_FACTOR: f64 = 2.0;
pub const MAX_TOP_EMOJIS_LIMIT: usize = 50;

// App metadata
pub const APP_BUNDLE_IDENTIFIER: &str = "dev.rgeraskin.emojiq";

// Helper functions for common durations
pub const fn write_delay() -> Duration {
    Duration::from_secs(RANK_WRITE_DELAY_SECS)
}
