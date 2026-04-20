
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

    /// Get the config directory path for the index file.
    pub fn index_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("dispel-gui");
        path.push("search_index.json");
        path
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



fn index_all_files_recursive(
    game_path: &Path,
    dir: &Path,
    file_mappings: &mut Vec<FileMapping>,
) {
    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                index_all_files_recursive(game_path, &path, file_mappings);
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if let Ok(relative_path) = path.strip_prefix(game_path) {
                    let editor_type = match ext.to_lowercase().as_str() {
                        "db" => "DatabaseEditor",
                        "ini" => "IniEditor",
                        "ref" => "RefEditor",
                        "scr" => "ScriptEditor",
                        "map" => "MapEditor",
                        "dlg" | "pgp" => "DialogEditor",
                        "gtl" | "btl" => "TilesetEditor",
                        "spr" => "SpriteViewer",
                        _ => "UnknownEditor",
                    };
                    
                    file_mappings.push(FileMapping {
                        file_path: relative_path.to_string_lossy().to_string(),
                        editor_type: editor_type.to_string(),
                    });
                }
            }
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
            editor_type: "SpriteViewer".to_string(),
        });
    }
}

fn find_sprites_recursive(dir: &Path, entries: &mut Vec<IndexedEntry>, count: &mut usize) {
    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                find_sprites_recursive(&path, entries, count);
            } else if path.extension().is_some_and(|e| e == "spr") {
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    entries.push(IndexedEntry {
                        label: format!("[Sprite] {}", name),
                        editor_type: "SpriteViewer".to_string(),
                        record_idx: *count,
                        source_file: Some(path.to_string_lossy().to_string()),
                    });
                    *count += 1;
                }
            }
        }
    }
}
