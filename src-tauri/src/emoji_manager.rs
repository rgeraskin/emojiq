use crate::constants::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, Once, RwLock};
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

    // Initialization synchronization
    init_once: Once,
}

impl Default for EmojiManager {
    fn default() -> Self {
        Self::new(DEFAULT_EMOJI_FILE.into(), DEFAULT_RANKS_FILE.to_string())
    }
}

impl EmojiManager {
    /// Create a new EmojiManager with custom file paths
    pub fn new(emoji_file_path: PathBuf, ranks_file_name: String) -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let ranks_file_path = home_dir.join(ranks_file_name);

        Self {
            emoji_file_path,
            ranks_file_path,
            data: Arc::new(RwLock::new(EmojiManagerData::default())),
            pending_writes: Arc::new(Mutex::new(false)),
            last_write_time: Arc::new(Mutex::new(Instant::now())),
            write_delay: write_delay(),
            init_once: Once::new(),
        }
    }

    /// Initialize all data structures at startup
    pub fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut init_result = Ok(());

        self.init_once.call_once(|| {
            println!("Initializing emoji manager...");

            // Load emojis first
            if let Err(e) = self.load_emojis() {
                init_result = Err(e);
                return;
            }

            // Load ranks
            if let Err(e) = self.load_ranks() {
                init_result = Err(e);
                return;
            }

            // Build keywords
            if let Err(e) = self.build_keywords() {
                init_result = Err(e);
                return;
            }

            // Build search index
            if let Err(e) = self.build_index() {
                init_result = Err(e);
                return;
            }

            println!("Emoji manager initialized successfully");
        });

        init_result
    }

    /// Load emoji data from JSON file
    pub fn load_emojis(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Check if already loaded (read lock is cheaper)
        {
            let data = self
                .data
                .read()
                .map_err(|_| "Failed to acquire read lock for emoji data")?;
            if data.emojis_loaded {
                return Ok(());
            }
        }

        let content = fs::read_to_string(&self.emoji_file_path)?;
        let emoji_data: Vec<EmojiData> = serde_json::from_str(&content)?;

        // Update with write lock
        {
            let mut data = self
                .data
                .write()
                .map_err(|_| "Failed to acquire write lock for emoji data")?;
            data.emojis = emoji_data;
            data.emojis_loaded = true;
            println!("Loaded {} emojis", data.emojis.len());
        }

        Ok(())
    }

    /// Load usage ranks from file
    pub fn load_ranks(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Check if already loaded (read lock is cheaper)
        {
            let data = self
                .data
                .read()
                .map_err(|_| "Failed to acquire read lock for ranks data")?;
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
            let mut data = self
                .data
                .write()
                .map_err(|_| "Failed to acquire write lock for ranks data")?;
            data.ranks = ranks_data;
            data.ranks_loaded = true;
            println!("Loaded {} ranks", data.ranks.len());
        }

        Ok(())
    }

    /// Build keyword mappings
    pub fn build_keywords(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Check if already built (read lock is cheaper)
        {
            let data = self
                .data
                .read()
                .map_err(|_| "Failed to acquire read lock for keywords data")?;
            if data.keywords_built {
                println!("Keywords already built, skipping");
                return Ok(());
            }
        }

        self.load_emojis()?;

        let mut keywords_map = HashMap::new();

        // Build keywords with read lock
        {
            let data = self
                .data
                .read()
                .map_err(|_| "Failed to acquire read lock for emoji data")?;

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
            let mut data = self
                .data
                .write()
                .map_err(|_| "Failed to acquire write lock for keywords data")?;
            data.keywords = keywords_map;
            data.keywords_built = true;
            println!("Built keywords for {} emojis", data.keywords.len());
        }

        Ok(())
    }

    /// Build search index
    pub fn build_index(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Check if already built (read lock is cheaper)
        {
            let data = self
                .data
                .read()
                .map_err(|_| "Failed to acquire read lock for index data")?;
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
                    for i in MIN_KEYWORD_LENGTH..=chars.len() {
                        let prefix: String = chars[..i].iter().collect();
                        index.entry(prefix).or_insert_with(Vec::new).push(emoji_idx);
                    }
                }
            };

        // Build inverted index: keyword -> vec of emoji indices (with read lock)
        {
            let data = self
                .data
                .read()
                .map_err(|_| "Failed to acquire read lock for keywords data")?;

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
            let mut data = self
                .data
                .write()
                .map_err(|_| "Failed to acquire write lock for index data")?;
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
    fn order_emojis_by_usage(&self, emojis: Vec<String>) -> Vec<String> {
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
        let top_emojis = self.get_top_emojis_from_ranks(&data.ranks, MAX_TOP_EMOJIS);

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

        // Clone necessary data for the async task
        let pending_writes = Arc::clone(&self.pending_writes);
        let last_write_time = Arc::clone(&self.last_write_time);
        let data = Arc::clone(&self.data);
        let ranks_file_path = self.ranks_file_path.clone();
        let write_delay = self.write_delay;

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
        });
    }

    /// Get filtered emojis as array with optimized memory usage and result limits
    pub fn get_emojis(&self, filter_word: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        println!("get_emojis called with filter: '{}'", filter_word);
        let filter_word = filter_word.trim().to_lowercase();

        let emoji_list: Vec<String> = if filter_word.len() < MIN_SEARCH_LENGTH {
            println!("Getting all emojis (filter too short)");
            // Return all emojis when filter is too short, limited by MAX_SEARCH_RESULTS to avoid overwhelming UI
            let data = self
                .data
                .read()
                .map_err(|_| "Failed to acquire read lock for emoji data")?;

            data.emojis
                .iter()
                .take(MAX_SEARCH_RESULTS)
                .map(|e| e.emoji.clone())
                .collect()
        } else {
            println!("Getting emojis for filter word: '{}'", filter_word);
            // Index is already built at startup, now using emoji indices
            let data = self
                .data
                .read()
                .map_err(|_| "Failed to acquire read lock for index data")?;

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

        // Order emojis by usage frequency
        let ordered_emojis = self.order_emojis_by_usage(emoji_list);

        println!("Returning {} emojis", ordered_emojis.len());
        Ok(ordered_emojis)
    }

    /// Get keywords for an emoji as array
    pub fn get_keywords(&self, emoji: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Keywords are already built at startup
        let data = self
            .data
            .read()
            .map_err(|_| "Failed to acquire read lock for keywords data")?;

        // Clone the Arc contents to maintain API compatibility while being more efficient internally
        let emoji_keywords = data
            .keywords
            .get(emoji)
            .map(|arc_keywords| (**arc_keywords).clone())
            .unwrap_or_default();
        Ok(emoji_keywords)
    }

    /// Increment usage count for an emoji
    pub fn increment_usage(&self, emoji: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("Incrementing usage for emoji: '{}'", emoji);

        // Ranks are already loaded at startup
        {
            let mut data = self
                .data
                .write()
                .map_err(|_| "Failed to acquire write lock for ranks data")?;
            let count = data.ranks.entry(emoji.to_string()).or_insert(0);
            *count += 1;
        }

        // Schedule batched write
        self.schedule_write();

        Ok(())
    }
}

// Public API functions
/// Get filtered emojis as array
pub fn get_emojis(manager: &EmojiManager, filter_word: &str) -> Result<Vec<String>, String> {
    manager.get_emojis(filter_word).map_err(|e| e.to_string())
}

/// Get keywords for an emoji as array
pub fn get_keywords(manager: &EmojiManager, emoji: &str) -> Result<Vec<String>, String> {
    manager.get_keywords(emoji).map_err(|e| e.to_string())
}

/// Increment usage count for an emoji
pub fn increment_usage(manager: &EmojiManager, emoji: &str) -> Result<(), String> {
    manager.increment_usage(emoji).map_err(|e| e.to_string())
}
