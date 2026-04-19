use super::*;
use crate::indexation_service::IndexationService;
use tempfile::tempdir;

#[test]
fn test_cache_serialization_deserialization() {
    // Create test cache data
    let mut cache = FileIndexCache {
        game_path: PathBuf::from("/test/game"),
        last_indexed: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        files: Vec::new(),
    };

    // Add test files
    cache.files.push(CachedFileInfo {
        path: PathBuf::from("/test/game/monster.db"),
        name: "monster.db".to_string(),
        is_directory: false,
        file_type: "db".to_string(),
        icon: "🗃️".to_string(),
        modified_time: 1234567890,
        sprite_metadata: None,
    });

    cache.files.push(CachedFileInfo {
        path: PathBuf::from("/test/game/sprite.spr"),
        name: "sprite.spr".to_string(),
        is_directory: false,
        file_type: "spr".to_string(),
        icon: "🎨".to_string(),
        modified_time: 1234567891,
        sprite_metadata: Some(SpriteMetadata {
            sequence_count: 3,
            frame_counts: vec![10, 15, 8],
        }),
    });

    // Test serialization
    let _cache_manager = FileIndexCacheManager::new().unwrap();
    let temp_dir = tempdir().unwrap();
    let _test_cache_path = temp_dir.path().join("test.cache");

    // Save cache to temp location for test
    let test_cache_manager = FileIndexCacheManager {
        cache_dir: temp_dir.path().to_path_buf(),
    };
    test_cache_manager.save_cache(&cache).unwrap();

    // Load cache back
    let loaded_cache = test_cache_manager.load_cache().unwrap();
    assert!(loaded_cache.is_some());

    let loaded_cache = loaded_cache.unwrap();
    assert_eq!(loaded_cache.game_path, cache.game_path);
    assert_eq!(loaded_cache.files.len(), 2);

    // Verify first file
    let file1 = &loaded_cache.files[0];
    assert_eq!(file1.name, "monster.db");
    assert!(!file1.is_directory);
    assert_eq!(file1.file_type, "db");
    assert_eq!(file1.icon, "🗃️");
    assert!(file1.sprite_metadata.is_none());

    // Verify second file (sprite)
    let file2 = &loaded_cache.files[1];
    assert_eq!(file2.name, "sprite.spr");
    assert!(!file2.is_directory);
    assert_eq!(file2.file_type, "spr");
    assert_eq!(file2.icon, "🎨");

    let sprite_meta = file2.sprite_metadata.as_ref().unwrap();
    assert_eq!(sprite_meta.sequence_count, 3);
    assert_eq!(sprite_meta.frame_counts, vec![10, 15, 8]);
}

#[test]
fn test_cache_validation() {
    let _cache_manager = FileIndexCacheManager::new().unwrap();

    // Create test cache
    let cache = FileIndexCache {
        game_path: PathBuf::from("/test/game"),
        last_indexed: FileIndexCacheManager::current_timestamp(),
        files: Vec::new(),
    };

    // Test validation with same path (should be valid)
    assert!(IndexationService::validate_sprite_cache(
        &cache,
        &PathBuf::from("/test/game")
    ));

    // Test validation with different path (should be invalid)
    assert!(!IndexationService::validate_sprite_cache(
        &cache,
        &PathBuf::from("/different/game")
    ));
}

#[test]
fn test_cache_invalidation_by_time() {
    let _cache_manager = FileIndexCacheManager::new().unwrap();

    // Create cache that's 31 days old (should be invalid)
    let old_timestamp = FileIndexCacheManager::current_timestamp() - (31 * 24 * 60 * 60);

    let cache = FileIndexCache {
        game_path: PathBuf::from("/test/game"),
        last_indexed: old_timestamp,
        files: Vec::new(),
    };

    assert!(!IndexationService::validate_sprite_cache(
        &cache,
        &PathBuf::from("/test/game")
    ));
}

#[test]
fn test_cache_directory_creation() {
    let cache_manager = FileIndexCacheManager::new().unwrap();

    // Verify cache directory was created
    assert!(cache_manager.cache_dir.exists());
    assert!(cache_manager.cache_dir.is_dir());

    // Verify cache file path
    let cache_path = cache_manager.get_cache_path();
    assert_eq!(cache_path.extension().unwrap(), "cache");
}

#[test]
fn test_cache_deletion() {
    let cache_manager = FileIndexCacheManager::new().unwrap();

    // Create a test cache
    let cache = FileIndexCache {
        game_path: PathBuf::from("/test/game"),
        last_indexed: FileIndexCacheManager::current_timestamp(),
        files: Vec::new(),
    };

    // Save it
    cache_manager.save_cache(&cache).unwrap();
    assert!(cache_manager.get_cache_path().exists());

    // Delete it
    cache_manager.delete_cache().unwrap();
    assert!(!cache_manager.get_cache_path().exists());
}

#[test]
fn test_sprite_metadata_structure() {
    let metadata = SpriteMetadata {
        sequence_count: 5,
        frame_counts: vec![12, 8, 15, 10, 6],
    };

    assert_eq!(metadata.sequence_count, 5);
    assert_eq!(metadata.frame_counts.len(), 5);
    assert_eq!(metadata.frame_counts[0], 12);
    assert_eq!(metadata.frame_counts[2], 15);
}

#[test]
fn test_file_icon_mapping() {
    assert_eq!(
        IndexationService::get_file_icon(&PathBuf::from("test.db")),
        "🗃️"
    );
    assert_eq!(
        IndexationService::get_file_icon(&PathBuf::from("test.ini")),
        "📄"
    );
    assert_eq!(
        IndexationService::get_file_icon(&PathBuf::from("test.ref")),
        "📋"
    );
    assert_eq!(
        IndexationService::get_file_icon(&PathBuf::from("test.spr")),
        "🎨"
    );
    assert_eq!(
        IndexationService::get_file_icon(&PathBuf::from("test.snf")),
        "🔊"
    );
    assert_eq!(
        IndexationService::get_file_icon(&PathBuf::from("test.unknown")),
        "📎"
    );
}
