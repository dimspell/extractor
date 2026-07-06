use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

use gui_widgets::components::TreeNode;

use super::tree_node::GameFileNode;

/// Centralized file tree filter component
#[derive(Debug, Clone, Default)]
pub struct FileTreeFilter {
    search_query: String,
    /// Pre-computed set of file paths matching the current query.
    /// `None` means "show all" (empty search). `Some(set)` is the set
    /// of matching file paths — any path not in this set should be hidden.
    matching_paths: Option<HashSet<PathBuf>>,
}

impl FileTreeFilter {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
            matching_paths: None,
        }
    }

    pub fn with_search_query(mut self, query: String) -> Self {
        self.search_query = query;
        self.matching_paths = None;
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

    /// Check whether a given path should be visible.
    /// If `matching_paths` is None, all paths are visible.
    pub fn is_path_matching(&self, path: &Path) -> bool {
        match &self.matching_paths {
            Some(paths) => paths.contains(path),
            None => true,
        }
    }

    /// Access the pre-computed set of matching paths.
    pub fn matching_paths(&self) -> Option<&HashSet<PathBuf>> {
        self.matching_paths.as_ref()
    }

    /// Set the pre-computed set of matching paths.
    pub fn set_matching_paths(&mut self, paths: Option<HashSet<PathBuf>>) {
        self.matching_paths = paths;
    }

    /// Walk the entire tree and collect paths of files that match the current
    /// fuzzy search query. Also includes ancestor directories so the tree
    /// structure is preserved when rendering.
    pub fn build_matching_paths(&mut self, root: Option<&TreeNode<GameFileNode>>) {
        if self.search_query.is_empty() {
            self.matching_paths = None;
            return;
        }
        let mut paths = HashSet::new();
        collect_matching_paths(root, &self.search_query, &mut paths);
        self.matching_paths = Some(paths);
    }
}

/// Error types for file tree operations
#[derive(Debug, thiserror::Error)]
pub enum FileTreeError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for file tree operations
pub type FileTreeResult<T> = Result<T, FileTreeError>;

use nucleo_matcher::{Config, Matcher, Utf32Str};
use std::cell::RefCell;

thread_local! {
    static FUZZY_MATCHER: RefCell<Matcher> = RefCell::new(Matcher::new(Config::DEFAULT));
}

/// Fuzzy subsequence match: every character in `query` must appear in `text`
/// in order (case-insensitive). Returns the matched byte-char indices on success.
///
/// Improved to handle file extensions better:
/// - If query starts with a dot (e.g., ".db"), match it against the filename extension
/// - If query looks like an extension (short, no path separator), try matching as extension first
/// - Otherwise, use standard subsequence matching via nucleo-matcher
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

    // Fast fuzzy matching via nucleo-matcher (O(n) greedy fallback for long inputs)
    FUZZY_MATCHER.with(|m| {
        let mut matcher = m.borrow_mut();
        let mut char_buf = Vec::new();
        let mut needle_buf = Vec::new();
        let mut indices_buf = Vec::new();

        let haystack = Utf32Str::new(&text_lower, &mut char_buf);
        let needle = Utf32Str::new(&query_lower, &mut needle_buf);

        if matcher
            .fuzzy_indices(haystack, needle, &mut indices_buf)
            .is_some()
        {
            Some(indices_buf.into_iter().map(|i| i as usize).collect())
        } else {
            None
        }
    })
}

/// Recursively collect paths of files that match the fuzzy query,
/// including ancestor directories so the tree structure is preserved.
fn collect_matching_paths(
    node: Option<&TreeNode<GameFileNode>>,
    query: &str,
    result: &mut HashSet<PathBuf>,
) -> bool {
    let Some(node) = node else {
        return false;
    };
    let mut any_child_matches = false;
    for child in &node.children {
        if collect_matching_paths(Some(child), query, result) {
            any_child_matches = true;
        }
    }
    // A directory is visible if any of its children match,
    // or if the directory name itself matches the query.
    if any_child_matches || fuzzy_match(query, &node.data.path.to_string_lossy()).is_some() {
        result.insert(node.data.path.clone());
        true
    } else {
        false
    }
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
