use std::time::Duration;

// Timing constants
pub const FOCUS_RESTORATION_DELAY_MS: u64 = 200;
pub const RANK_WRITE_DELAY_SECS: u64 = 2;
// pub const ASYNC_WRITE_CHECK_DELAY_MS: u64 = 100; // Currently unused

// Search constants
pub const MIN_SEARCH_LENGTH: usize = 2;
pub const MAX_SEARCH_RESULTS: usize = 2000;
pub const MAX_TOP_EMOJIS: usize = 10;
pub const MIN_KEYWORD_LENGTH: usize = 2;
pub const MAX_PREFIX_LENGTH: usize = 12; // Cap for prefix indexing to bound memory

// UI constants
pub const PANEL_CORNER_RADIUS: f64 = 12.0;

// File constants
pub const DEFAULT_EMOJI_FILE: &str = "src/emoji.json";
pub const DEFAULT_RANKS_FILE: &str = "ranks.json";

// App metadata
pub const APP_BUNDLE_IDENTIFIER: &str = "dev.rgeraskin.emojiq";

// Helper functions for common durations
pub const fn write_delay() -> Duration {
    Duration::from_secs(RANK_WRITE_DELAY_SECS)
}
