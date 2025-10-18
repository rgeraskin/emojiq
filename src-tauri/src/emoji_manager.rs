use crate::constants::*;
use crate::errors::{EmojiError, LockResultExt};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, RwLock,
};
use std::thread;
use std::time::Instant;

/// Emoji data structure matching the JSON format
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmojiData {
    pub emoji: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub unicode_version: Option<String>,
    pub ios_version: Option<String>,
}

/// Data structure for emoji manager
#[derive(Debug, Default)]
pub struct EmojiManagerData {
    pub emojis: Vec<EmojiData>,
    pub ranks: HashMap<String, u32>,
    pub keywords: HashMap<String, Arc<Vec<String>>>,
    pub index: HashMap<String, Vec<usize>>,
    // Loading flags
    pub emojis_loaded: bool,
    pub ranks_loaded: bool,
    pub keywords_built: bool,
    pub index_built: bool,
}

/// Thread-safe emoji manager with caching and efficient search
#[derive(Debug)]
pub struct EmojiManager {
    pub emoji_file_path: PathBuf,
    pub ranks_file_path: PathBuf,

    // Consolidated data storage with RwLock for better read performance
    pub data: Arc<RwLock<EmojiManagerData>>,

    // Threading for file operations
    pending_writes: Arc<Mutex<bool>>,
    last_write_time: Arc<Mutex<Instant>>,
    write_delay: std::time::Duration,
    write_worker_active: Arc<std::sync::atomic::AtomicBool>,

    // Initialization synchronization (retryable)
    init_lock: Mutex<()>,
    init_success: AtomicBool,
}

impl Default for EmojiManager {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let ranks_file_path = home_dir.join(DEFAULT_RANKS_FILE);
        Self::new(DEFAULT_EMOJI_FILE.into(), ranks_file_path)
    }
}

impl EmojiManager {
    /// Create a new EmojiManager with explicit file paths
    pub fn new(emoji_file_path: PathBuf, ranks_file_path: PathBuf) -> Self {
        Self {
            emoji_file_path,
            ranks_file_path,
            data: Arc::new(RwLock::new(EmojiManagerData::default())),
            pending_writes: Arc::new(Mutex::new(false)),
            last_write_time: Arc::new(Mutex::new(Instant::now())),
            write_delay: write_delay(),
            write_worker_active: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            init_lock: Mutex::new(()),
            init_success: AtomicBool::new(false),
        }
    }

    /// Initialize all data structures at startup (retryable on failure)
    pub fn initialize(&self) -> Result<(), EmojiError> {
        if self.init_success.load(Ordering::Acquire) {
            return Ok(());
        }

        let _guard = self.init_lock.lock().map_lock_err()?;
        if self.init_success.load(Ordering::Acquire) {
            return Ok(());
        }

        println!("Initializing emoji manager...");

        self.load_emojis()?;
        self.load_ranks()?;
        self.build_keywords()?;
        self.build_index()?;

        self.init_success.store(true, Ordering::Release);
        println!("Emoji manager initialized successfully");
        Ok(())
    }

    /// Load emoji data from JSON file
    pub fn load_emojis(&self) -> Result<(), EmojiError> {
        // Check if already loaded (read lock is cheaper)
        {
            let data = self.data.read().map_lock_err()?;
            if data.emojis_loaded {
                return Ok(());
            }
        }

        // Use embedded emoji data for production builds, fallback to file system for development
        let content = if cfg!(debug_assertions) {
            // Development: try to read from file system first, fallback to embedded
            match fs::read_to_string(&self.emoji_file_path) {
                Ok(content) => content,
                Err(_) => {
                    println!("Could not read emoji file from filesystem, using embedded data");
                    include_str!("emoji.json").to_string()
                }
            }
        } else {
            // Production: always use embedded data
            include_str!("emoji.json").to_string()
        };

        let emoji_data: Vec<EmojiData> = serde_json::from_str(&content)?;

        // Update with write lock
        {
            let mut data = self.data.write().map_lock_err()?;
            data.emojis = emoji_data;
            data.emojis_loaded = true;
            println!("Loaded {} emojis", data.emojis.len());
        }

        Ok(())
    }

    /// Load usage ranks from file
    pub fn load_ranks(&self) -> Result<(), EmojiError> {
        // Check if already loaded (read lock is cheaper)
        {
            let data = self.data.read().map_lock_err()?;
            if data.ranks_loaded {
                return Ok(());
            }
        }

        let ranks_data = if self.ranks_file_path.exists() {
            let content = fs::read_to_string(&self.ranks_file_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        // Update with write lock
        {
            let mut data = self.data.write().map_lock_err()?;
            data.ranks = ranks_data;
            data.ranks_loaded = true;
            println!("Loaded {} ranks", data.ranks.len());
        }

        Ok(())
    }

    /// Build keyword mappings
    pub fn build_keywords(&self) -> Result<(), EmojiError> {
        // Check if already built (read lock is cheaper)
        {
            let data = self.data.read().map_lock_err()?;
            if data.keywords_built {
                println!("Keywords already built, skipping");
                return Ok(());
            }
        }

        self.load_emojis()?;

        let mut keywords_map = HashMap::new();

        // Build keywords with read lock
        {
            let data = self.data.read().map_lock_err()?;

            for emoji_data in data.emojis.iter() {
                let emoji = &emoji_data.emoji;
                let description = emoji_data
                    .description
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .replace('_', " ");

                let aliases = emoji_data.aliases.as_deref().unwrap_or(&[]);
                let tags = emoji_data.tags.as_deref().unwrap_or(&[]);

                // Create keyword list with description first
                let mut keywords = vec![description.clone()];

                // Add aliases and tags, sorted by length for better matching
                let mut all_keywords: Vec<String> = aliases
                    .iter()
                    .chain(tags.iter())
                    .map(|kw| kw.to_lowercase().replace('_', " "))
                    .collect();

                // Sort by length
                all_keywords.sort_by_key(|k| k.len());

                // Remove duplicates and description variants
                let mut seen = HashSet::new();
                seen.insert(description);

                for keyword in all_keywords {
                    if !seen.contains(&keyword) {
                        keywords.push(keyword.clone());
                        seen.insert(keyword);
                    }
                }

                // Use Arc to avoid cloning when accessing keywords
                keywords_map.insert(emoji.clone(), Arc::new(keywords));
            }
        }

        // Update with write lock
        {
            let mut data = self.data.write().map_lock_err()?;
            data.keywords = keywords_map;
            data.keywords_built = true;
            println!("Built keywords for {} emojis", data.keywords.len());
        }

        Ok(())
    }

    /// Build search index
    pub fn build_index(&self) -> Result<(), EmojiError> {
        // Check if already built (read lock is cheaper)
        {
            let data = self.data.read().map_lock_err()?;
            if data.index_built {
                println!("Index already built, skipping");
                return Ok(());
            }
        }

        let mut index_map = HashMap::new();

        // Helper function to index keywords using emoji indices for better memory efficiency
        let index_keyword =
            |keywords: &[String], emoji_idx: usize, index: &mut HashMap<String, Vec<usize>>| {
                let filtered_keywords: Vec<&String> = keywords
                    .iter()
                    .filter(|k| k.len() >= MIN_KEYWORD_LENGTH)
                    .collect();

                for keyword in filtered_keywords {
                    // Index full keyword
                    index
                        .entry(keyword.clone())
                        .or_insert_with(Vec::new)
                        .push(emoji_idx);

                    // Index prefixes for partial matching (min length MIN_KEYWORD_LENGTH)
                    let chars: Vec<char> = keyword.chars().collect();
                    let cap = MAX_PREFIX_LENGTH.min(chars.len());
                    for i in MIN_KEYWORD_LENGTH..=cap {
                        let prefix: String = chars[..i].iter().collect();
                        index.entry(prefix).or_insert_with(Vec::new).push(emoji_idx);
                    }
                }
            };

        // Build inverted index: keyword -> vec of emoji indices (with read lock)
        {
            let data = self.data.read().map_lock_err()?;

            for (emoji_idx, emoji_data) in data.emojis.iter().enumerate() {
                let emoji = &emoji_data.emoji;
                if let Some(emoji_keywords) = data.keywords.get(emoji) {
                    index_keyword(emoji_keywords, emoji_idx, &mut index_map);

                    // Also index individual words from multi-word keywords
                    for keyword in emoji_keywords.iter() {
                        let words: Vec<String> = keyword
                            .replace('-', " ")
                            .split_whitespace()
                            .map(|s| s.to_string())
                            .collect();

                        if words.len() > 1 {
                            index_keyword(&words, emoji_idx, &mut index_map);
                        }
                    }
                }
            }

            // Remove duplicates and sort indices for better cache locality
            for indices in index_map.values_mut() {
                indices.sort_unstable();
                indices.dedup();
            }
        }

        // Update with write lock
        {
            let mut data = self.data.write().map_lock_err()?;
            data.index = index_map;
            data.index_built = true;
            println!("Built index for {} matches", data.index.len());
        }

        Ok(())
    }

    /// Get top emojis from ranks data
    fn get_top_emojis_from_ranks(&self, ranks: &HashMap<String, u32>, limit: usize) -> Vec<String> {
        if ranks.is_empty() {
            println!("No ranks found");
            return Vec::new();
        }

        let mut emoji_ranks: Vec<(&String, &u32)> = ranks.iter().collect();

        // Sort by count in descending order (highest usage first)
        emoji_ranks.sort_by_key(|(_, &count)| std::cmp::Reverse(count));

        emoji_ranks
            .into_iter()
            .take(limit)
            .map(|(emoji, _)| emoji.clone())
            .collect()
    }

    /// Emoji ordering by usage frequency
    fn order_emojis_by_usage(&self, emojis: Vec<String>, max_top_emojis: usize) -> Vec<String> {
        let data = match self.data.read() {
            Ok(data) => data,
            Err(_) => {
                println!("Failed to acquire read lock for ranks, returning emojis as-is");
                return emojis;
            }
        };

        if data.ranks.is_empty() {
            println!("No ranks found, returning emojis as-is");
            return emojis;
        }

        // Get top most used emojis
        let top_emojis = self.get_top_emojis_from_ranks(&data.ranks, max_top_emojis);

        // Create HashSets for O(1) lookups instead of O(n) contains() calls
        let top_emojis_set: HashSet<&String> = top_emojis.iter().collect();
        let emojis_set: HashSet<&String> = emojis.iter().collect();

        let mut result = Vec::with_capacity(emojis.len());

        // 1. Add top 10 emojis first (in usage order)
        for top_emoji in &top_emojis {
            // Check if this top emoji exists in our emoji list using O(1) lookup
            if emojis_set.contains(top_emoji) {
                result.push(top_emoji.clone());
            }
        }

        // 2. Add all other emojis in their original order
        for emoji in &emojis {
            if !top_emojis_set.contains(emoji) {
                result.push(emoji.clone());
            }
        }
        result
    }

    /// Schedule a batched write operation
    fn schedule_write(&self) {
        {
            if let Ok(mut pending) = self.pending_writes.lock() {
                *pending = true;
            }
            if let Ok(mut last_write) = self.last_write_time.lock() {
                *last_write = Instant::now();
            }
        }

        // Only spawn a worker if one is not already active
        if self
            .write_worker_active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_err()
        {
            return; // writer already active; it will pick up the updated timestamp
        }

        // Clone necessary data for the async task
        let pending_writes = Arc::clone(&self.pending_writes);
        let last_write_time = Arc::clone(&self.last_write_time);
        let data = Arc::clone(&self.data);
        let ranks_file_path = self.ranks_file_path.clone();
        let write_delay = self.write_delay;
        let write_worker_active = Arc::clone(&self.write_worker_active);

        // Schedule write in separate thread
        thread::spawn(move || {
            thread::sleep(write_delay);

            let should_write = {
                let pending = match pending_writes.lock() {
                    Ok(guard) => *guard,
                    Err(_) => {
                        println!("Pending writes lock poisoned, assuming should write");
                        true
                    }
                };
                let last_write_elapsed = match last_write_time.lock() {
                    Ok(guard) => guard.elapsed(),
                    Err(_) => {
                        println!("Last write time lock poisoned, assuming should write");
                        write_delay
                    }
                };
                pending && last_write_elapsed >= write_delay
            };

            if should_write {
                let ranks_data = {
                    match data.read() {
                        Ok(data) => data.ranks.clone(),
                        Err(_) => {
                            eprintln!("Failed to acquire read lock for ranks during write");
                            write_worker_active.store(false, Ordering::Release);
                            return;
                        }
                    }
                };

                match serde_json::to_string(&ranks_data) {
                    Ok(json_content) => {
                        if let Err(e) = fs::write(&ranks_file_path, json_content) {
                            eprintln!("Failed to write ranks: {}", e);
                        } else {
                            if let Ok(mut pending) = pending_writes.lock() {
                                *pending = false;
                            }
                            println!("Wrote usage ranks to file");
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to serialize ranks: {}", e);
                    }
                }
            }
            // Mark worker inactive regardless of outcome to allow future scheduling
            write_worker_active.store(false, Ordering::Release);
        });
    }

    /// Get filtered emojis as array with optimized memory usage and result limits
    pub fn get_emojis(
        &self,
        filter_word: &str,
        max_top_emojis: usize,
    ) -> Result<Vec<String>, EmojiError> {
        println!("get_emojis called with filter: '{}'", filter_word);
        let filter_word = filter_word.trim().to_lowercase();

        let emoji_list: Vec<String> = if filter_word.len() < MIN_SEARCH_LENGTH {
            println!("Getting all emojis (filter too short)");
            // Return all emojis when filter is too short, limited by MAX_SEARCH_RESULTS to avoid overwhelming UI
            let data = self.data.read().map_lock_err()?;

            data.emojis
                .iter()
                .take(MAX_SEARCH_RESULTS)
                .map(|e| e.emoji.clone())
                .collect()
        } else {
            println!("Getting emojis for filter word: '{}'", filter_word);
            // Index is already built at startup, now using emoji indices
            let data = self.data.read().map_lock_err()?;

            if let Some(emoji_indices) = data.index.get(&filter_word) {
                emoji_indices
                    .iter()
                    .take(MAX_SEARCH_RESULTS) // Limit results for better performance
                    .filter_map(|&idx| data.emojis.get(idx))
                    .map(|emoji_data| emoji_data.emoji.clone())
                    .collect()
            } else {
                Vec::new()
            }
        };

        // Order emojis by usage frequency (skip if max_top_emojis is 0)
        let ordered_emojis = if max_top_emojis == 0 {
            emoji_list
        } else {
            self.order_emojis_by_usage(emoji_list, max_top_emojis)
        };

        println!("Returning {} emojis", ordered_emojis.len());
        Ok(ordered_emojis)
    }

    /// Get keywords for an emoji as array
    pub fn get_keywords(&self, emoji: &str) -> Result<Vec<String>, EmojiError> {
        // Keywords are already built at startup
        let data = self.data.read().map_lock_err()?;

        // Clone the Arc contents to maintain API compatibility while being more efficient internally
        let emoji_keywords = data
            .keywords
            .get(emoji)
            .map(|arc_keywords| (**arc_keywords).clone())
            .unwrap_or_default();
        Ok(emoji_keywords)
    }

    /// Increment usage count for an emoji
    pub fn increment_usage(&self, emoji: &str) -> Result<(), EmojiError> {
        println!("Incrementing usage for emoji: '{}'", emoji);

        // Ranks are already loaded at startup
        {
            let mut data = self.data.write().map_lock_err()?;
            let count = data.ranks.entry(emoji.to_string()).or_insert(0);
            *count += 1;
        }

        // Schedule batched write
        self.schedule_write();

        Ok(())
    }
}
