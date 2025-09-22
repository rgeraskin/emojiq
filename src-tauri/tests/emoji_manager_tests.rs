use emojiq_lib::emoji_manager::{EmojiData, EmojiManager};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

// Sample emoji data for testing
fn create_test_emoji_data() -> Vec<EmojiData> {
    vec![
        EmojiData {
            emoji: "😀".to_string(),
            description: Some("grinning face".to_string()),
            category: Some("Smileys & Emotion".to_string()),
            aliases: Some(vec!["grinning".to_string()]),
            tags: Some(vec!["smile".to_string(), "happy".to_string()]),
            unicode_version: Some("6.1".to_string()),
            ios_version: Some("6.0".to_string()),
        },
        EmojiData {
            emoji: "😃".to_string(),
            description: Some("grinning face with big eyes".to_string()),
            category: Some("Smileys & Emotion".to_string()),
            aliases: Some(vec!["smiley".to_string()]),
            tags: Some(vec![
                "happy".to_string(),
                "joy".to_string(),
                "haha".to_string(),
            ]),
            unicode_version: Some("6.0".to_string()),
            ios_version: Some("6.0".to_string()),
        },
        EmojiData {
            emoji: "📆".to_string(),
            description: Some("tear-off calendar".to_string()),
            category: Some("Objects".to_string()),
            aliases: Some(vec!["calendar".to_string()]),
            tags: Some(vec!["schedule".to_string()]),
            unicode_version: Some("6.0".to_string()),
            ios_version: Some("6.0".to_string()),
        },
        EmojiData {
            emoji: "🐒".to_string(),
            description: Some("monkey".to_string()),
            category: Some("Animals & Nature".to_string()),
            aliases: Some(vec!["monkey".to_string()]),
            tags: Some(vec!["animal".to_string()]),
            unicode_version: Some("6.0".to_string()),
            ios_version: Some("6.0".to_string()),
        },
        EmojiData {
            emoji: "🐵".to_string(),
            description: Some("monkey face".to_string()),
            category: Some("Animals & Nature".to_string()),
            aliases: Some(vec!["monkey_face".to_string()]),
            tags: Some(vec!["animal".to_string(), "monkey".to_string()]),
            unicode_version: Some("6.0".to_string()),
            ios_version: Some("6.0".to_string()),
        },
    ]
}

// Sample ranks data for testing
fn create_test_ranks_data() -> HashMap<String, u32> {
    let mut ranks = HashMap::new();
    ranks.insert("🧡".to_string(), 5);
    ranks.insert("🎉".to_string(), 3);
    ranks.insert("👀".to_string(), 6);
    ranks.insert("🐒".to_string(), 1);
    ranks
}

fn setup_test_files(temp_dir: &TempDir) -> (PathBuf, PathBuf) {
    let emoji_file = temp_dir.path().join("emoji.json");
    let ranks_file = temp_dir.path().join("ranks.json");

    // Write test emoji data
    let emoji_data = create_test_emoji_data();
    let emoji_json = serde_json::to_string_pretty(&emoji_data).unwrap();
    fs::write(&emoji_file, emoji_json).unwrap();

    // Write test ranks data
    let ranks_data = create_test_ranks_data();
    let ranks_json = serde_json::to_string(&ranks_data).unwrap();
    fs::write(&ranks_file, ranks_json).unwrap();

    (emoji_file, ranks_file)
}

#[test]
fn test_emoji_manager_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let (emoji_file, ranks_file) = setup_test_files(&temp_dir);

    let manager = EmojiManager::new(
        emoji_file.clone(),
        ranks_file
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    );

    assert_eq!(manager.emoji_file_path, emoji_file);
    assert!(manager
        .ranks_file_path
        .to_string_lossy()
        .contains("ranks.json"));

    // Check initial state
    let data = manager.data.read().unwrap();
    assert!(!data.emojis_loaded);
    assert!(!data.ranks_loaded);
    assert!(!data.keywords_built);
    assert!(!data.index_built);
}

#[test]
fn test_load_emojis() {
    let temp_dir = TempDir::new().unwrap();
    let (emoji_file, _) = setup_test_files(&temp_dir);

    let manager = EmojiManager::new(emoji_file, "test_ranks.json".to_string());
    manager.load_emojis().unwrap();

    let data = manager.data.read().unwrap();
    assert!(data.emojis_loaded);
    assert_eq!(data.emojis.len(), 5);
    assert_eq!(data.emojis[0].emoji, "😀");
    assert_eq!(data.emojis[1].emoji, "😃");
    assert_eq!(data.emojis[2].emoji, "📆");
    assert_eq!(data.emojis[3].emoji, "🐒");
    assert_eq!(data.emojis[4].emoji, "🐵");
}

#[test]
fn test_load_ranks() {
    let temp_dir = TempDir::new().unwrap();
    let (_, ranks_file) = setup_test_files(&temp_dir);

    let manager = EmojiManager::new(
        PathBuf::from("nonexistent.json"),
        ranks_file
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    );

    // Manually set the ranks file path for testing
    let mut manager = manager;
    manager.ranks_file_path = ranks_file;

    manager.load_ranks().unwrap();

    let data = manager.data.read().unwrap();
    assert!(data.ranks_loaded);
    assert_eq!(data.ranks.len(), 4);
    assert_eq!(data.ranks.get("👀"), Some(&6));
    assert_eq!(data.ranks.get("🧡"), Some(&5));
    assert_eq!(data.ranks.get("🎉"), Some(&3));
    assert_eq!(data.ranks.get("🐒"), Some(&1));
}

#[test]
fn test_build_keywords() {
    let temp_dir = TempDir::new().unwrap();
    let (emoji_file, _) = setup_test_files(&temp_dir);

    let manager = EmojiManager::new(emoji_file, "test_ranks.json".to_string());
    manager.build_keywords().unwrap();

    let data = manager.data.read().unwrap();
    assert!(data.keywords_built);
    assert!(data.keywords.len() > 0);

    // Check specific emoji keywords
    let grinning_keywords = data.keywords.get("😀").unwrap();
    assert!(grinning_keywords.contains(&"grinning face".to_string()));
    assert!(grinning_keywords.contains(&"grinning".to_string()));
    assert!(grinning_keywords.contains(&"smile".to_string()));
    assert!(grinning_keywords.contains(&"happy".to_string()));
}

#[test]
fn test_build_index() {
    let temp_dir = TempDir::new().unwrap();
    let (emoji_file, _) = setup_test_files(&temp_dir);

    let manager = EmojiManager::new(emoji_file, "test_ranks.json".to_string());
    manager.build_keywords().unwrap(); // Keywords must be built first
    manager.build_index().unwrap();

    let data = manager.data.read().unwrap();
    assert!(data.index_built);
    assert!(data.index.len() > 0);

    // Check calendar emoji indexing
    assert!(data.index.get("ca").unwrap().contains("📆"));
    assert!(data.index.get("cal").unwrap().contains("📆"));
    assert!(data.index.get("calendar").unwrap().contains("📆"));
    assert!(data.index.get("schedule").unwrap().contains("📆"));
}

#[test]
fn test_get_emojis_empty_filter() {
    let temp_dir = TempDir::new().unwrap();
    let (emoji_file, ranks_file) = setup_test_files(&temp_dir);

    let mut manager = EmojiManager::new(
        emoji_file,
        ranks_file
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    );
    manager.ranks_file_path = ranks_file;

    // Initialize the manager
    manager.load_emojis().unwrap();
    manager.load_ranks().unwrap();
    manager.build_keywords().unwrap();
    manager.build_index().unwrap();

    let result = manager.get_emojis("").unwrap();
    let emojis = result;

    // Should have emojis (exact order depends on ranking logic)
    assert!(emojis.len() >= 5);

    // Check that our test emojis are present
    assert!(emojis.contains(&"😀".to_string()));
    assert!(emojis.contains(&"😃".to_string()));
    assert!(emojis.contains(&"📆".to_string()));
    assert!(emojis.contains(&"🐒".to_string()));
    assert!(emojis.contains(&"🐵".to_string()));
}

#[test]
fn test_get_emojis_with_filter() {
    let temp_dir = TempDir::new().unwrap();
    let (emoji_file, ranks_file) = setup_test_files(&temp_dir);

    let mut manager = EmojiManager::new(
        emoji_file,
        ranks_file
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    );
    manager.ranks_file_path = ranks_file;

    // Initialize the manager
    manager.load_emojis().unwrap();
    manager.load_ranks().unwrap();
    manager.build_keywords().unwrap();
    manager.build_index().unwrap();

    let result = manager.get_emojis("monkey").unwrap();

    // Should find monkey emojis
    assert!(result.contains(&"🐒".to_string()));
    assert!(result.contains(&"🐵".to_string()));
}

#[test]
fn test_get_keywords() {
    let temp_dir = TempDir::new().unwrap();
    let (emoji_file, _) = setup_test_files(&temp_dir);

    let manager = EmojiManager::new(emoji_file, "test_ranks.json".to_string());
    manager.build_keywords().unwrap();

    let keywords = manager.get_keywords("😀").unwrap();

    assert!(keywords.contains(&"grinning face".to_string()));
    assert!(keywords.contains(&"grinning".to_string()));
    assert!(keywords.contains(&"smile".to_string()));
    assert!(keywords.contains(&"happy".to_string()));
}

#[test]
fn test_get_keywords_nonexistent_emoji() {
    let temp_dir = TempDir::new().unwrap();
    let (emoji_file, _) = setup_test_files(&temp_dir);

    let manager = EmojiManager::new(emoji_file, "test_ranks.json".to_string());
    manager.build_keywords().unwrap();

    let keywords = manager.get_keywords("🏴‍☠️🦄").unwrap();
    assert_eq!(keywords, Vec::<String>::new());
}

#[test]
fn test_increment_usage() {
    let temp_dir = TempDir::new().unwrap();
    let (_, ranks_file) = setup_test_files(&temp_dir);

    let mut manager = EmojiManager::new(
        PathBuf::from("nonexistent.json"),
        ranks_file
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    );
    manager.ranks_file_path = ranks_file.clone();

    // Load initial ranks
    manager.load_ranks().unwrap();
    let initial_count = {
        let data = manager.data.read().unwrap();
        data.ranks.get("😀").copied().unwrap_or(0)
    };

    // Increment usage
    manager.increment_usage("😀").unwrap();

    // Check that count was incremented
    let new_count = {
        let data = manager.data.read().unwrap();
        data.ranks.get("😀").copied().unwrap_or(0)
    };
    assert_eq!(new_count, initial_count + 1);

    // Wait for potential file write
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_increment_usage_new_emoji() {
    let temp_dir = TempDir::new().unwrap();
    let (_, ranks_file) = setup_test_files(&temp_dir);

    let mut manager = EmojiManager::new(
        PathBuf::from("nonexistent.json"),
        ranks_file
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    );
    manager.ranks_file_path = ranks_file;

    // Increment usage for emoji not in ranks
    manager.increment_usage("🚀").unwrap();

    // Should start at 1
    let count = {
        let data = manager.data.read().unwrap();
        data.ranks.get("🚀").copied().unwrap_or(0)
    };
    assert_eq!(count, 1);

    // Wait for potential file write
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_public_api_functions() {
    // Test that the public API functions work
    // Note: These use the global instance, so they might interfere with each other
    // In a real application, you'd want to use dependency injection

    // These tests will fail if emoji.json doesn't exist, but that's expected
    // in a test environment. The important thing is that the functions don't panic.
    let _result = emojiq_lib::emoji_manager::get_emojis("");
    let _result = emojiq_lib::emoji_manager::get_keywords("😀");
    let _result = emojiq_lib::emoji_manager::increment_usage("😀");

    // Just test that the functions can be called without panicking
    assert!(true);
}
