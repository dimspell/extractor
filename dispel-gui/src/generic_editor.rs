use dispel_core::references::editable::EditableRecord;
use dispel_core::Extractor;
use std::path::{Path, PathBuf};

/// Generic editor state that works with any `EditableRecord` type.
///
/// Replaces the 28 duplicated `*EditorState` structs with a single
/// parameterized implementation.
#[derive(Clone, Debug, Default)]
pub struct GenericEditorState<R: EditableRecord> {
    pub catalog: Option<Vec<R>>,
    pub filtered: Vec<(usize, R)>,
    pub selected_idx: Option<usize>,
    /// One string buffer per field descriptor, indexed by position.
    pub edit_buffers: Vec<String>,
    pub status_msg: String,
    pub is_loading: bool,
}

impl<R: EditableRecord + Extractor> GenericEditorState<R> {
    /// Populate the filtered list from the catalog.
    pub fn refresh(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect();
        }
    }

    /// Select a record by index in the filtered list, loading its fields into edit buffers.
    pub fn select(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        if let Some((_, record)) = self.filtered.get(idx) {
            let descriptors = R::field_descriptors();
            self.edit_buffers = descriptors
                .iter()
                .map(|d| record.get_field(d.name))
                .collect();
        }
    }

    /// Update a field value in the edit buffer, the filtered record, and the original catalog.
    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some((orig_idx, record)) = self.filtered.get_mut(idx) {
            if record.set_field(field, value.clone()) {
                // Update the matching buffer
                if let Some(pos) = R::field_descriptors().iter().position(|d| d.name == field) {
                    self.edit_buffers[pos] = value;
                }
                // Sync back to the original catalog entry
                let orig = *orig_idx;
                if let Some(catalog) = &mut self.catalog {
                    if let Some(catalog_record) = catalog.get_mut(orig) {
                        *catalog_record = record.clone();
                    }
                }
            }
        }
    }

    /// Save the catalog to disk, creating a timestamped .bak backup first.
    pub fn save(&self, game_path: &str, db_path: &str) -> Result<(), String> {
        let path = std::path::PathBuf::from(game_path).join(db_path);
        if let Some(catalog) = &self.catalog {
            // Create timestamped backup
            if path.exists() {
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let backup_path = path.with_extension(format!(
                    "{}.{}.bak",
                    path.extension().and_then(|e| e.to_str()).unwrap_or("bak"),
                    ts
                ));
                std::fs::copy(&path, &backup_path)
                    .map_err(|e| format!("Failed to create backup: {}", e))?;
            }
            R::save_file(catalog, &path).map_err(|e| format!("Failed to save: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }

    /// Read a file from disk.
    pub fn scan_and_read(base_path: &Path, db_path: &str) -> Result<Vec<R>, String> {
        R::read_file(&base_path.join(db_path)).map_err(|e| format!("Failed to read: {}", e))
    }
}

// ===========================================================================
// Multi-file editor (3-panel: file list | record list | record editor)
// ===========================================================================

/// Generic editor state for types that span multiple files.
///
/// Used by MonsterRef, ExtraRef, NpcRef, DialogueText, PartyLevelDb, etc.
/// Each file contains records of the same type, and the user picks a file
/// to edit its records.
///
/// Wraps a `GenericEditorState` for the record editing portion.
#[derive(Clone, Debug, Default)]
pub struct MultiFileEditorState<R: EditableRecord> {
    pub file_list: Vec<PathBuf>,
    pub current_file: Option<PathBuf>,
    /// The underlying generic editor for record-level operations.
    pub editor: GenericEditorState<R>,
}

impl<R: EditableRecord + Extractor> MultiFileEditorState<R> {
    /// Scan for files matching the glob pattern under the game path.
    pub fn scan_files(&mut self, game_path: &Path, pattern: &str) {
        self.file_list.clear();
        self.current_file = None;
        self.editor.catalog = None;
        self.editor.filtered.clear();
        self.editor.selected_idx = None;
        self.editor.edit_buffers.clear();

        if let Ok(entries) = std::fs::read_dir(game_path) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if glob_match(name, pattern) {
                        self.file_list.push(path);
                    }
                }
            }
        }
        self.file_list.sort();
    }

    /// Select a file and load its records.
    pub fn select_file(&mut self, path: PathBuf) {
        self.current_file = Some(path.clone());
        match R::read_file(&path) {
            Ok(catalog) => {
                self.editor.catalog = Some(catalog);
                self.editor.refresh();
            }
            Err(e) => {
                self.editor.status_msg = format!("Error loading {}: {}", path.display(), e);
                self.editor.catalog = None;
            }
        }
    }

    /// Update a field value, syncing to both filtered and catalog.
    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        self.editor.update_field(idx, field, value);
    }

    /// Select a record by index.
    pub fn select(&mut self, idx: usize) {
        self.editor.select(idx);
    }

    /// Add a new default record to the catalog.
    pub fn add_record(&mut self) -> usize {
        if let Some(catalog) = &mut self.editor.catalog {
            let idx = catalog.len();
            catalog.push(R::default());
            self.editor.refresh();
            self.select(idx);
            idx
        } else {
            0
        }
    }

    /// Remove a record by its filtered index.
    pub fn remove_record(&mut self, idx: usize) {
        if let Some(catalog) = &mut self.editor.catalog {
            if let Some((orig_idx, _)) = self.editor.filtered.get(idx) {
                let orig = *orig_idx;
                if orig < catalog.len() {
                    catalog.remove(orig);
                    // Rebuild filtered list since indices shifted
                    self.editor.refresh();
                    // Clear selection
                    self.editor.selected_idx = None;
                    self.editor.edit_buffers.clear();
                }
            }
        }
    }

    /// Save the current file's catalog back to disk, creating a timestamped .bak backup first.
    pub fn save(&self) -> Result<(), String> {
        let path = self.current_file.as_ref().ok_or("No file selected")?;
        if let Some(catalog) = &self.editor.catalog {
            // Create timestamped backup
            if path.exists() {
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let backup_path = path.with_extension(format!(
                    "{}.{}.bak",
                    path.extension().and_then(|e| e.to_str()).unwrap_or("bak"),
                    ts
                ));
                std::fs::copy(path, &backup_path)
                    .map_err(|e| format!("Failed to create backup: {}", e))?;
            }
            R::save_file(catalog, path).map_err(|e| format!("Failed to save: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}

/// Simple glob matching: supports `*` (any chars) and `?` (one char).
fn glob_match(name: &str, pattern: &str) -> bool {
    let mut ni = 0;
    let mut pi = 0;
    let nb = name.len();
    let pb = pattern.len();
    let name_bytes = name.as_bytes();
    let pat_bytes = pattern.as_bytes();
    let mut star_pi = None;
    let mut match_ni = 0;

    while ni < nb {
        if pi < pb && (pat_bytes[pi] == b'?' || pat_bytes[pi] == name_bytes[ni]) {
            ni += 1;
            pi += 1;
        } else if pi < pb && pat_bytes[pi] == b'*' {
            star_pi = Some(pi);
            match_ni = ni;
            pi += 1;
        } else if let Some(sp) = star_pi {
            pi = sp + 1;
            match_ni += 1;
            ni = match_ni;
        } else {
            return false;
        }
    }

    while pi < pb && pat_bytes[pi] == b'*' {
        pi += 1;
    }

    pi == pb
}
