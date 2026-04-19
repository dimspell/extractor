// File index cache module
use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// File index cache data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIndexCache {
    pub game_path: PathBuf,
    pub last_indexed: u64, // Unix timestamp
    pub files: Vec<CachedFileInfo>,
}

/// Cached file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedFileInfo {
    pub path: PathBuf,
    pub name: String,
    pub is_directory: bool,
    pub file_type: String,                       // Extension or type
    pub icon: String,                            // Emoji icon
    pub modified_time: u64,                      // Unix timestamp
    pub sprite_metadata: Option<SpriteMetadata>, // Sprite-specific metadata
}

/// Sprite file metadata for caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteMetadata {
    pub sequence_count: usize,
    pub frame_counts: Vec<usize>,
}

/// Cache error types
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("Cache corrupted or invalid")]
    Corrupted,
    #[error("Cache directory not accessible")]
    DirectoryAccess,
    #[error("File access error")]
    FileAccess,
}

/// Maximum cache size in bytes (50MB)
const MAX_CACHE_SIZE: u64 = 50 * 1024 * 1024;

/// Cache manager
#[derive(Debug, Clone)]
pub struct FileIndexCacheManager {
    cache_dir: PathBuf,
}

impl FileIndexCacheManager {
    /// Create new cache manager
    pub fn new() -> Result<Self, CacheError> {
        let cache_dir = Self::get_cache_directory()?;
        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self { cache_dir })
    }

    /// Get cache directory using dirs crate
    fn get_cache_directory() -> Result<PathBuf, CacheError> {
        let cache_dir = dirs::cache_dir().ok_or(CacheError::DirectoryAccess)?;
        Ok(cache_dir.join("dispel-gui"))
    }

    /// Get cache file path
    fn get_cache_path(&self) -> PathBuf {
        self.cache_dir.join("file_index.cache")
    }

    /// Save cache to disk
    pub fn save_cache(&self, cache: &FileIndexCache) -> Result<(), CacheError> {
        let cache_data = serialize(cache)?;
        std::fs::write(self.get_cache_path(), cache_data)?;
        Ok(())
    }

    /// Load cache from disk
    pub fn load_cache(&self) -> Result<Option<FileIndexCache>, CacheError> {
        let cache_path = self.get_cache_path();
        if !cache_path.exists() {
            return Ok(None); // No cache file
        }

        // Check cache size before loading
        self.enforce_size_limit()?;

        let cache_data = std::fs::read(cache_path)?;
        let cache = deserialize(&cache_data)?;
        Ok(Some(cache))
    }

    /// Delete cache file
    pub fn delete_cache(&self) -> Result<(), CacheError> {
        let cache_path = self.get_cache_path();
        if cache_path.exists() {
            std::fs::remove_file(cache_path)?;
        }
        Ok(())
    }

    /// Get current Unix timestamp
    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Calculate current cache size in bytes
    pub fn calculate_cache_size(&self) -> Result<u64, CacheError> {
        let cache_path = self.get_cache_path();

        if cache_path.exists() {
            let metadata = std::fs::metadata(&cache_path).map_err(|_| CacheError::FileAccess)?;
            Ok(metadata.len())
        } else {
            Ok(0)
        }
    }

    /// Check if cache exceeds maximum size
    pub fn is_cache_too_large(&self) -> Result<bool, CacheError> {
        let current_size = self.calculate_cache_size()?;
        Ok(current_size > MAX_CACHE_SIZE)
    }

    /// Clean up cache if it exceeds maximum size
    pub fn enforce_size_limit(&self) -> Result<(), CacheError> {
        if self.is_cache_too_large()? {
            // For now, simply delete the cache if it's too large
            // In future, could implement more sophisticated cleanup
            self.delete_cache()?;
            println!(
                "Cache size exceeded {}MB, cache cleared",
                MAX_CACHE_SIZE / 1024 / 1024
            );
        }
        Ok(())
    }

    /// Perform periodic cache maintenance
    /// This should be called during app initialization and periodically during use
    pub fn perform_periodic_cleanup(&self) -> Result<(), CacheError> {
        // Check and enforce size limits
        self.enforce_size_limit()?;

        // Additional cleanup logic could be added here:
        // - Remove old cache files from previous versions
        // - Clean up temporary files
        // - Validate cache integrity

        Ok(())
    }
}

/// Cache validation utilities
pub struct CacheValidator;

impl CacheValidator {
    /// Validate cache against current file system
    pub fn validate_cache(cache: &FileIndexCache, current_game_path: &Path) -> bool {
        // Check if game path matches
        if cache.game_path != current_game_path {
            return false;
        }

        // Check if cache is too old (30 days)
        let cache_age = FileIndexCacheManager::current_timestamp() - cache.last_indexed;
        if cache_age > 30 * 24 * 60 * 60 {
            // 30 days in seconds
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests;
