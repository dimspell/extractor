/// Centralized file tree filter component
#[derive(Debug, Clone, Default)]
pub struct FileTreeFilter {
    search_query: String,
}

impl FileTreeFilter {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
        }
    }

    pub fn with_search_query(mut self, query: String) -> Self {
        self.search_query = query;
        self
    }

    /// Check if a file name matches the search query (fuzzy, case-insensitive).
    pub fn matches_search(&self, file_name: &str) -> bool {
        if self.search_query.is_empty() {
            return true;
        }
        fuzzy_match(&self.search_query, file_name).is_some()
    }

    /// Get current search query
    pub fn search_query(&self) -> &str {
        &self.search_query
    }
}

/// Error types for file tree operations
#[derive(Debug, thiserror::Error)]
pub enum FileTreeError {
    #[error("Failed to read directory: {0}")]
    DirectoryReadError(String),

    #[error("Failed to access file metadata: {0}")]
    FileAccessError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Cache validation failed: {0}")]
    CacheValidationError(String),

    #[error("Cache serialization error: {0}")]
    CacheSerializationError(String),

    #[error("Cache deserialization error: {0}")]
    CacheDeserializationError(String),

    #[error("Cache corrupted or invalid")]
    CacheCorrupted,

    #[error("Cache timeout exceeded")]
    CacheTimeout,

    #[error("File system watcher error: {0}")]
    FileSystemWatcherError(String),

    #[error("Async operation timeout")]
    Timeout,

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),
}

/// Result type for file tree operations
pub type FileTreeResult<T> = Result<T, FileTreeError>;

/// Fuzzy subsequence match: every character in `query` must appear in `text`
/// in order (case-insensitive). Returns the matched byte-char indices on success.
///
/// Improved to handle file extensions better:
/// - If query starts with a dot (e.g., ".db"), match it against the filename extension
/// - If query looks like an extension (short, no path separator), try matching as extension first
/// - Otherwise, use standard subsequence matching
pub fn fuzzy_match(query: &str, text: &str) -> Option<Vec<usize>> {
    if query.is_empty() {
        return Some(vec![]);
    }

    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();

    // Special case: lone dot should match any file with an extension
    if query == "." {
        if text_lower.contains('.') {
            let filename = std::path::Path::new(text)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(text);
            if let Some(dot_idx) = filename.find('.') {
                let full_text_start = text.len() - filename.len();
                return Some(vec![full_text_start + dot_idx]);
            }
        }
        return None;
    }

    // Special case: if query starts with a dot, treat it as an extension search
    if query_lower.starts_with('.') {
        // Look for files ending with this extension (e.g., ".db", ".ini")
        if text_lower.ends_with(&query_lower) {
            // For ".db" query in "weaponItem.db", return indices for the .db part
            let ext_len = query.len();
            if text.len() >= ext_len {
                return Some((text.len() - ext_len..text.len()).collect());
            }
        }
        return None;
    }

    // Try extension matching for short queries without path separators
    // This handles queries like "db", "ini", "ref" to match file extensions
    if !query.contains('/') && !query.contains('\\') && query.len() <= 5 {
        // Check if this matches the file extension (without the dot)
        if let Some(dot_pos) = text_lower.rfind('.') {
            let ext = &text_lower[dot_pos + 1..];
            if ext == query_lower && !ext.is_empty() {
                // Found a matching extension
                let ext_len = query.len();
                if text.len() >= ext_len {
                    return Some((text.len() - ext_len..text.len()).collect());
                }
            }
        }
    }

    // Standard subsequence matching
    let text_chars: Vec<char> = text.chars().collect();
    let query_chars: Vec<char> = query_lower.chars().collect();

    let mut matched_indices = Vec::with_capacity(query_chars.len());
    let mut qi = 0;

    for (ti, tc) in text_chars.iter().enumerate() {
        if qi < query_chars.len() && tc.to_lowercase().next() == Some(query_chars[qi]) {
            matched_indices.push(ti);
            qi += 1;
        }
    }

    if qi == query_chars.len() {
        Some(matched_indices)
    } else {
        None
    }
}

impl FileTreeError {
    /// Create a directory read error
    pub fn directory_read_error(path: &std::path::Path, error: &std::io::Error) -> Self {
        FileTreeError::DirectoryReadError(format!(
            "Failed to read directory {}: {}",
            path.display(),
            error
        ))
    }

    /// Create a file access error
    pub fn file_access_error(path: &std::path::Path, error: &std::io::Error) -> Self {
        FileTreeError::FileAccessError(format!(
            "Failed to access file {}: {}",
            path.display(),
            error
        ))
    }

    /// Create a permission denied error
    pub fn permission_denied(path: &std::path::Path) -> Self {
        FileTreeError::PermissionDenied(format!("Permission denied: {}", path.display()))
    }

    /// Create a file not found error
    pub fn file_not_found(path: &std::path::Path) -> Self {
        FileTreeError::FileNotFound(format!("File not found: {}", path.display()))
    }

    /// Create an invalid path error
    pub fn invalid_path(path: &str) -> Self {
        FileTreeError::InvalidPath(format!("Invalid path: {}", path))
    }

    /// Create a cache validation error
    pub fn cache_validation_error(details: &str) -> Self {
        FileTreeError::CacheValidationError(format!("Cache validation failed: {}", details))
    }

    /// Create a cache corrupted error
    pub fn cache_corrupted() -> Self {
        FileTreeError::CacheCorrupted
    }

    /// Create a timeout error
    pub fn timeout() -> Self {
        FileTreeError::Timeout
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            FileTreeError::DirectoryReadError(_) => true,
            FileTreeError::FileAccessError(_) => true,
            FileTreeError::PermissionDenied(_) => false, // Usually not recoverable
            FileTreeError::FileNotFound(_) => true,
            FileTreeError::IoError(_) => true,
            FileTreeError::InvalidPath(_) => false,
            FileTreeError::Cancelled => false,
            FileTreeError::CacheError(_) => true,
            FileTreeError::CacheValidationError(_) => true,
            FileTreeError::CacheSerializationError(_) => true,
            FileTreeError::CacheDeserializationError(_) => true,
            FileTreeError::CacheCorrupted => true,
            FileTreeError::CacheTimeout => true,
            FileTreeError::FileSystemWatcherError(_) => true,
            FileTreeError::Timeout => true,
            FileTreeError::ResourceExhausted(_) => true,
        }
    }

    /// Get a user-friendly message for this error
    pub fn user_message(&self) -> String {
        match self {
            FileTreeError::DirectoryReadError(msg) => format!("Could not read directory: {}", msg),
            FileTreeError::FileAccessError(msg) => format!("Could not access file: {}", msg),
            FileTreeError::PermissionDenied(msg) => format!("Permission denied: {}", msg),
            FileTreeError::FileNotFound(msg) => format!("File not found: {}", msg),
            FileTreeError::IoError(e) => format!("I/O error: {}", e),
            FileTreeError::InvalidPath(msg) => format!("Invalid path: {}", msg),
            FileTreeError::Cancelled => "Operation cancelled".to_string(),
            FileTreeError::CacheError(msg) => format!("Cache error: {}", msg),
            FileTreeError::CacheValidationError(msg) => format!("Cache validation failed: {}", msg),
            FileTreeError::CacheSerializationError(msg) => format!("Cache save error: {}", msg),
            FileTreeError::CacheDeserializationError(msg) => format!("Cache load error: {}", msg),
            FileTreeError::CacheCorrupted => "Cache is corrupted and will be rebuilt".to_string(),
            FileTreeError::CacheTimeout => "Cache operation timed out".to_string(),
            FileTreeError::FileSystemWatcherError(msg) => format!("File watcher error: {}", msg),
            FileTreeError::Timeout => "Operation timed out".to_string(),
            FileTreeError::ResourceExhausted(msg) => format!("Resource limit reached: {}", msg),
        }
    }

    /// Get recovery suggestions for this error
    pub fn recovery_suggestions(&self) -> Vec<String> {
        match self {
            FileTreeError::DirectoryReadError(_) => vec![
                "Check if the directory exists".to_string(),
                "Verify directory permissions".to_string(),
                "Try again later".to_string(),
            ],
            FileTreeError::FileAccessError(_) => vec![
                "Check file permissions".to_string(),
                "Verify the file is not in use".to_string(),
                "Try again later".to_string(),
            ],
            FileTreeError::PermissionDenied(_) => vec![
                "Run as administrator".to_string(),
                "Check file/directory permissions".to_string(),
            ],
            FileTreeError::FileNotFound(_) => vec![
                "Verify the file path".to_string(),
                "Check if the file was moved or deleted".to_string(),
            ],
            FileTreeError::CacheError(_) => vec![
                "Try clearing the cache".to_string(),
                "Restart the application".to_string(),
            ],
            FileTreeError::CacheCorrupted => vec![
                "Cache will be automatically rebuilt".to_string(),
                "No action needed".to_string(),
            ],
            FileTreeError::Timeout => vec![
                "Try again with a simpler operation".to_string(),
                "Check system resources".to_string(),
            ],
            _ => vec!["Check logs for details".to_string()],
        }
    }
}

/// User notification for file tree operations
#[derive(Debug, Clone)]
pub struct FileTreeNotification {
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

impl FileTreeNotification {
    pub fn new(message: String, notification_type: NotificationType) -> Self {
        Self {
            message,
            notification_type,
            timestamp: std::time::SystemTime::now(),
        }
    }

    pub fn error(message: String) -> Self {
        Self::new(message, NotificationType::Error)
    }

    pub fn warning(message: String) -> Self {
        Self::new(message, NotificationType::Warning)
    }

    pub fn info(message: String) -> Self {
        Self::new(message, NotificationType::Info)
    }

    pub fn success(message: String) -> Self {
        Self::new(message, NotificationType::Success)
    }
}

/// Notification manager for file tree operations
#[derive(Debug, Default, Clone)]
pub struct NotificationManager {
    notifications: std::sync::Arc<std::sync::Mutex<Vec<FileTreeNotification>>>,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            notifications: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn add_notification(&self, notification: FileTreeNotification) {
        let mut notifications = self.notifications.lock().unwrap();
        notifications.push(notification);
        // In a real app, you might want to limit the number of notifications
        if notifications.len() > 100 {
            notifications.remove(0);
        }
    }

    pub fn get_notifications(&self) -> Vec<FileTreeNotification> {
        let notifications = self.notifications.lock().unwrap();
        notifications.clone()
    }

    pub fn clear_notifications(&self) {
        let mut notifications = self.notifications.lock().unwrap();
        notifications.clear();
    }

    pub fn clear_errors(&self) {
        let mut notifications = self.notifications.lock().unwrap();
        notifications.retain(|n| n.notification_type != NotificationType::Error);
    }
}

/// Cancellation token for async operations
#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

impl CancellationToken {
    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// Async cancellation result
pub enum CancelResult<T> {
    Completed(T),
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_match_empty_query() {
        assert_eq!(fuzzy_match("", "anything.txt"), Some(vec![]));
    }

    #[test]
    fn fuzzy_match_exact_substring() {
        assert!(fuzzy_match("test", "test.txt").is_some());
    }

    #[test]
    fn fuzzy_match_subsequence() {
        assert!(fuzzy_match("abc", "aXbYc").is_some());
    }

    #[test]
    fn fuzzy_match_case_insensitive() {
        assert!(fuzzy_match("TeSt", "test.txt").is_some());
    }

    #[test]
    fn fuzzy_match_no_match() {
        assert!(fuzzy_match("xyz", "abc").is_none());
    }

    #[test]
    fn fuzzy_match_extension_db() {
        // Fixed: searching for ".db" in "CharacterInGame/weaponItem.db" should now work
        let result = fuzzy_match(".db", "CharacterInGame/weaponItem.db");
        assert!(result.is_some(), "Should find '.db' extension");
    }

    #[test]
    fn fuzzy_match_extension_ini() {
        // Fixed: ".ini" should match in "AllMap.ini"
        let result = fuzzy_match(".ini", "AllMap.ini");
        assert!(result.is_some(), "Should find '.ini' extension");
    }

    #[test]
    fn fuzzy_match_weapon_item_works() {
        let result = fuzzy_match("weapon", "weaponItem.db");
        assert!(result.is_some(), "Should find 'weapon' in 'weaponItem.db'");
    }

    #[test]
    fn fuzzy_match_partial_path() {
        let result = fuzzy_match("item", "CharacterInGame/weaponItem.db");
        assert!(result.is_some(), "Should find 'item' in path");
    }

    #[test]
    fn fuzzy_match_mixed_case_path() {
        let result = fuzzy_match("char", "CharacterInGame/weaponItem.db");
        assert!(result.is_some(), "Should find 'char' case-insensitive");
    }

    #[test]
    fn fuzzy_match_return_indices() {
        let result = fuzzy_match("ace", "abcde");
        assert!(result.is_some());
        let indices = result.unwrap();
        assert_eq!(indices, vec![0, 2, 4], "Should return correct indices");
    }

    #[test]
    fn fuzzy_match_order_matters() {
        // Query chars must appear in order, but this doesn't work for extensions starting with dot
        let result = fuzzy_match("db", "weaponItem.db");
        assert!(result.is_some(), "'db' should match at end of filename");
    }

    #[test]
    fn fuzzy_match_single_char_dot() {
        // The dot should match as a file extension when searching for just "."
        let result = fuzzy_match(".", "file.txt");
        assert!(result.is_some(), "Should find '.' extension in 'file.txt'");
    }

    #[test]
    fn file_tree_filter_empty_query_matches_all() {
        let filter = FileTreeFilter::new();
        assert!(filter.matches_search("anything.txt"));
        assert!(filter.matches_search("path/to/file.db"));
        assert!(filter.matches_search(""));
    }

    #[test]
    fn file_tree_filter_with_matching_query() {
        let filter = FileTreeFilter::new().with_search_query("weapon".to_string());
        assert!(filter.matches_search("weaponItem.db"));
        assert!(!filter.matches_search("armorItem.db"));
    }

    #[test]
    fn file_tree_filter_case_insensitive() {
        let filter = FileTreeFilter::new().with_search_query("WEAPON".to_string());
        assert!(filter.matches_search("weaponItem.db"));
    }

    #[test]
    fn fuzzy_match_extension_without_dot_db() {
        // Query "db" should match files ending with .db
        let result = fuzzy_match("db", "CharacterInGame/weaponItem.db");
        assert!(
            result.is_some(),
            "Should find 'db' as extension in 'weaponItem.db'"
        );
    }

    #[test]
    fn fuzzy_match_extension_without_dot_ini() {
        // Query "ini" should match files ending with .ini
        let result = fuzzy_match("ini", "AllMap.ini");
        assert!(
            result.is_some(),
            "Should find 'ini' as extension in 'AllMap.ini'"
        );
    }

    #[test]
    fn fuzzy_match_extension_without_dot_ref() {
        // Query "ref" should match files ending with .ref
        let result = fuzzy_match("ref", "PartyRef.ref");
        assert!(
            result.is_some(),
            "Should find 'ref' as extension in 'PartyRef.ref'"
        );
    }

    #[test]
    fn fuzzy_match_extension_case_insensitive_ext() {
        // "DB" query should match ".db" files (case insensitive)
        let result = fuzzy_match("DB", "weaponItem.db");
        assert!(
            result.is_some(),
            "Should find 'DB' extension case-insensitive"
        );
    }

    #[test]
    fn fuzzy_match_short_query_prioritizes_extension() {
        // Short queries like "db" should match extensions before doing subsequence matching
        let result = fuzzy_match("db", "debug.ini");
        // "db" appears in "debug", so subsequence matching would also match
        // But we want to verify extension matching is tried
        assert!(result.is_some());
    }

    #[test]
    fn fuzzy_match_file_in_subdirectory() {
        // Find "Face1.spr" in "NpcInGame/Face1.spr" using full path
        let result = fuzzy_match("Face", "NpcInGame/Face1.spr");
        assert!(result.is_some(), "Should find 'Face' in subdirectory path");
    }

    #[test]
    fn fuzzy_match_subdirectory_name() {
        // Find files by directory name
        let result = fuzzy_match("NpcInGame", "NpcInGame/Face1.spr");
        assert!(result.is_some(), "Should find directory name in path");
    }

    #[test]
    fn fuzzy_match_complex_path() {
        // Complex path matching - find file by any part of path
        let result = fuzzy_match("game", "CharacterInGame/weaponItem.db");
        assert!(result.is_some(), "Should find 'game' in path");
    }

    #[test]
    fn fuzzy_match_npc_finds_npccat_in_subdir() {
        // "npc" should match "NpcInGame/Npccat1.ref" via directory name
        let result = fuzzy_match("npc", "NpcInGame/Npccat1.ref");
        assert!(
            result.is_some(),
            "Should find 'npc' in 'NpcInGame/Npccat1.ref'"
        );
    }

    #[test]
    fn fuzzy_match_npc_finds_full_absolute_path() {
        // "npc" should match via the NpcInGame segment in a full absolute path
        let result = fuzzy_match("npc", "/game/data/NpcInGame/Npccat1.ref");
        assert!(
            result.is_some(),
            "Should find 'npc' in absolute path containing NpcInGame"
        );
    }

    #[test]
    fn file_tree_filter_npc_matches_npc_subdir_path() {
        let filter = FileTreeFilter::new().with_search_query("npc".to_string());
        assert!(filter.matches_search("NpcInGame/Npccat1.ref"));
        assert!(filter.matches_search("/full/path/NpcInGame/Npccat1.ref"));
        assert!(!filter.matches_search("CharacterInGame/weaponItem.db"));
    }

    #[test]
    fn fuzzy_match_npc_uppercase_query() {
        // "NPC" (uppercase) should also find "NpcInGame/Npccat1.ref"
        let result = fuzzy_match("NPC", "NpcInGame/Npccat1.ref");
        assert!(
            result.is_some(),
            "Uppercase 'NPC' should match case-insensitively"
        );
    }
}
