use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A single indexed entry from a game file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedEntry {
    /// Display label shown in search results.
    pub label: String,
    /// The editor type this entry belongs to (e.g. "WeaponEditor").
    pub editor_type: String,
    /// The record index within the editor's catalog.
    pub record_idx: usize,
    /// Optional source file path (for file-based navigation).
    pub source_file: Option<String>,
}

/// Maps a file path to the editor type that handles it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMapping {
    pub file_path: String,
    pub editor_type: String,
}

/// The persistent search index.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchIndex {
    /// All indexed entries.
    pub entries: Vec<IndexedEntry>,
    /// File-to-editor mappings.
    pub file_mappings: Vec<FileMapping>,
    /// The game path this index was built for.
    pub game_path: Option<String>,
    /// Whether indexing is currently in progress.
    pub indexing: bool,
    /// Progress of indexing (0.0 to 1.0).
    pub progress: f32,
}

impl SearchIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Search for file mappings matching the query.
    /// Optimized to return only top results for performance.
    pub fn search_files(&self, query: &str) -> Vec<FileMapping> {
        if query.is_empty() {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();

        // Optimize: Limit to reasonable number of results for UI performance
        const MAX_RESULTS: usize = 100;

        self.file_mappings
            .iter()
            .filter(|m| m.file_path.to_lowercase().contains(&query_lower))
            .take(MAX_RESULTS)
            .cloned()
            .collect()
    }

    /// Save the index to a JSON file.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Load the index from a JSON file.
    pub fn load(path: &Path) -> Result<Self, String> {
        let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }

    /// Get the config directory path for the index file (under dispel-gui config).
    pub fn index_path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("dispel-gui").join("search_index.json")
    }

    /// Clear the index and reset state.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.file_mappings.clear();
        self.game_path = None;
        self.indexing = false;
        self.progress = 0.0;
    }
}

/// Build a search index from the given game path.
pub async fn build_index(game_path: &Path) -> SearchIndex {
    let mut index = SearchIndex::new();
    index.game_path = Some(game_path.to_string_lossy().to_string());

    // Recursively index all common game file types in all directories
    index_all_files_recursive(game_path, game_path, &mut index.file_mappings);

    // Index sprite files (for file mappings only)
    index_sprites(game_path, &mut index.entries, &mut index.file_mappings);

    index
}

fn index_all_files_recursive(game_path: &Path, dir: &Path, file_mappings: &mut Vec<FileMapping>) {
    let read_dir = match std::fs::read_dir(dir) {
        Ok(read_dir) => read_dir,
        Err(e) => {
            eprintln!("Index: cannot read directory {}: {}", dir.display(), e);
            return;
        }
    };
    for entry in read_dir {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Index: skipping unreadable entry: {}", e);
                continue;
            }
        };
        let path = entry.path();
        if path.is_dir() {
            index_all_files_recursive(game_path, &path, file_mappings);
        } else if path.extension().is_some()
            && let Ok(relative_path) = path.strip_prefix(game_path)
        {
            // Editor names come from the catalog via the same path detection used when opening a file.
            let editor_type = crate::workspace::EditorType::from_path(&path)
                .name()
                .to_string();

            file_mappings.push(FileMapping {
                file_path: relative_path.to_string_lossy().to_string(),
                editor_type,
            });
        }
    }
}

fn index_sprites(
    game_path: &Path,
    entries: &mut Vec<IndexedEntry>,
    file_mappings: &mut Vec<FileMapping>,
) {
    let mut sprite_count = 0;
    find_sprites_recursive(game_path, entries, &mut sprite_count);
    if sprite_count > 0 {
        file_mappings.push(FileMapping {
            file_path: "*.spr".to_string(),
            editor_type: crate::workspace::EditorType::SpriteViewer
                .name()
                .to_string(),
        });
    }
}

fn find_sprites_recursive(dir: &Path, entries: &mut Vec<IndexedEntry>, count: &mut usize) {
    let read_dir = match std::fs::read_dir(dir) {
        Ok(read_dir) => read_dir,
        Err(e) => {
            eprintln!("Index: cannot read directory {}: {}", dir.display(), e);
            return;
        }
    };
    for entry in read_dir {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Index: skipping unreadable entry: {}", e);
                continue;
            }
        };
        let path = entry.path();
        if path.is_dir() {
            find_sprites_recursive(&path, entries, count);
        } else if path.extension().is_some_and(|e| e == "spr")
            && let Some(name) = path.file_stem().and_then(|n| n.to_str())
        {
            entries.push(IndexedEntry {
                label: format!("[Sprite] {}", name),
                editor_type: crate::workspace::EditorType::SpriteViewer
                    .name()
                    .to_string(),
                record_idx: *count,
                source_file: Some(path.to_string_lossy().to_string()),
            });
            *count += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_index_normal_tree_keeps_mappings_and_entries() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let db_dir = root.join("CharacterInGame");
        let sprite_dir = root.join("Sprites");
        std::fs::create_dir_all(&db_dir).unwrap();
        std::fs::create_dir_all(&sprite_dir).unwrap();
        std::fs::write(db_dir.join("WeaponItem.db"), b"").unwrap();
        std::fs::write(sprite_dir.join("hero.spr"), b"").unwrap();
        // A file without an extension is skipped by the recursive mapping pass.
        std::fs::write(root.join("README"), b"").unwrap();

        let index = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(build_index(root));

        assert_eq!(index.game_path.as_deref(), Some(root.to_str().unwrap()));

        let db_rel = Path::new("CharacterInGame")
            .join("WeaponItem.db")
            .to_string_lossy()
            .to_string();
        let db_mapping = index
            .file_mappings
            .iter()
            .find(|m| m.file_path == db_rel)
            .expect("WeaponItem.db mapping should be indexed");
        assert_eq!(db_mapping.editor_type, "WeaponEditor");

        let spr_rel = Path::new("Sprites")
            .join("hero.spr")
            .to_string_lossy()
            .to_string();
        assert!(
            index
                .file_mappings
                .iter()
                .any(|m| m.file_path == spr_rel && m.editor_type == "SpriteViewer"),
            "sprite file mapping should be indexed"
        );
        assert!(
            !index.file_mappings.iter().any(|m| m.file_path == "README"),
            "extension-less files should not produce mappings"
        );

        let sprite_entries: Vec<_> = index
            .entries
            .iter()
            .filter(|e| e.editor_type == "SpriteViewer")
            .collect();
        assert_eq!(sprite_entries.len(), 1, "exactly one sprite entry expected");
        assert_eq!(sprite_entries[0].label, "[Sprite] hero");
        assert_eq!(sprite_entries[0].record_idx, 0);
        assert_eq!(
            sprite_entries[0].source_file.as_deref(),
            Some(sprite_dir.join("hero.spr").to_str().unwrap())
        );
        assert!(
            index.file_mappings.iter().any(|m| m.file_path == "*.spr"),
            "a *.spr summary mapping should be present when sprites exist"
        );
    }

    #[test]
    fn test_build_index_empty_tree_has_no_entries() {
        let tmp = tempfile::tempdir().unwrap();
        let index = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(build_index(tmp.path()));

        assert!(index.entries.is_empty());
        assert!(index.file_mappings.is_empty());
    }
}
