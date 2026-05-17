use crate::components::edit_history::EditHistory;
use crate::components::editable::EditableRecord;
use crate::components::textarea::TextAreaContent;
use crate::view::editor::spreadsheet::ColumnFilterOption;
use dispel_core::Extractor;
use iced::widget::pane_grid;
use iced::widget::pane_grid::Pane;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub trait UndoRedo {
    fn undo(&mut self) -> Option<String>;
    fn redo(&mut self) -> Option<String>;
    fn can_undo(&self) -> bool;
    fn can_redo(&self) -> bool;
    fn edit_history(&self) -> &EditHistory;
}

/// Generic editor state that works with any `EditableRecord` type.
///
/// Replaces the 28 duplicated `*EditorState` structs with a single
/// parameterized implementation.
#[derive(Clone, Debug)]
pub struct GenericEditorState<R: EditableRecord> {
    pub catalog: Option<Vec<R>>,
    pub filtered: Vec<(usize, R)>,
    pub selected_idx: Option<usize>,
    /// One string buffer per field descriptor, indexed by position.
    pub edit_buffers: Vec<String>,
    pub status_msg: String,
    pub loading_state: crate::components::loading_state::LoadingState<()>,
    pub edit_history: EditHistory,
    pub pane_state: Option<pane_grid::State<PaneContent>>,
    pub pane_focus: Option<Pane>,
}

impl<R: EditableRecord + Default> Default for GenericEditorState<R> {
    fn default() -> Self {
        Self {
            catalog: None,
            filtered: Vec::new(),
            selected_idx: None,
            edit_buffers: Vec::new(),
            status_msg: String::new(),
            loading_state: crate::components::loading_state::LoadingState::default(),
            edit_history: EditHistory::default(),
            pane_state: None,
            pane_focus: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PaneContent {
    ItemList,
    Inspector,
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
    /// Returns true if the field was valid and updated, false if validation failed.
    ///
    /// `idx` is the **catalog index** (not a filtered-list position). The method
    /// looks up the record by matching catalog indices in `filtered`, which is
    /// the same approach used by `undo` and `redo`.
    pub fn update_field(&mut self, idx: usize, field: &str, value: String) -> bool {
        if let Some((_, record)) = self.filtered.iter_mut().find(|(i, _)| *i == idx) {
            let old_value = record.get_field(field);
            if old_value == value {
                return true;
            }
            // First validate before attempting to set
            if let Some(error) = record.validate_field(field, &value) {
                self.status_msg = format!("Invalid '{}': {}", field, error);
                return false;
            }
            if record.set_field(field, value.clone()) {
                // Record the change in history
                self.edit_history
                    .push(crate::components::edit_history::EditAction::FieldChange {
                        record_idx: idx,
                        field: field.to_string(),
                        old_value,
                        new_value: value.clone(),
                    });
                // Update the matching buffer
                if let Some(pos) = R::field_descriptors().iter().position(|d| d.name == field) {
                    if let Some(buf) = self.edit_buffers.get_mut(pos) {
                        *buf = value;
                    }
                }
                // Sync back to the original catalog entry
                if let Some(catalog) = &mut self.catalog {
                    if let Some(catalog_record) = catalog.get_mut(idx) {
                        *catalog_record = record.clone();
                    }
                }
                return true;
            }
        }
        false
    }

    /// Undo the last edit and return information about what was undone.
    pub fn undo(&mut self) -> Option<String> {
        if let Some(action) = self.edit_history.undo() {
            match action {
                crate::components::edit_history::EditAction::FieldChange {
                    record_idx,
                    field,
                    old_value,
                    new_value: _,
                } => {
                    // Apply the old value back
                    if let Some((_, record)) =
                        self.filtered.iter_mut().find(|(i, _)| *i == record_idx)
                    {
                        let _ = record.set_field(&field, old_value.clone());
                        // Update buffer
                        if let Some(pos) =
                            R::field_descriptors().iter().position(|d| d.name == field)
                        {
                            if let Some(buf) = self.edit_buffers.get_mut(pos) {
                                *buf = old_value.clone();
                            }
                        }
                        // Update catalog
                        if let Some(catalog) = &mut self.catalog {
                            if let Some(cat_record) = catalog.get_mut(record_idx) {
                                let _ = cat_record.set_field(&field, old_value);
                            }
                        }
                        return Some(format!("Undo: {} changed back", field));
                    }
                    None
                }
                crate::components::edit_history::EditAction::RecordRemove { record_idx, data } => {
                    if let Ok(record) = serde_json::from_str::<R>(&data) {
                        if let Some(catalog) = &mut self.catalog {
                            if record_idx <= catalog.len() {
                                catalog.insert(record_idx, record);
                                self.refresh();
                                self.edit_history.adjust_for_addition(record_idx);
                                self.edit_buffers.clear();
                                self.selected_idx = None;
                                return Some(format!("Undo: restored record #{}", record_idx));
                            }
                        }
                    }
                    None
                }
                _ => Some("Undo: unsupported action".to_string()),
            }
        } else {
            None
        }
    }

    /// Redo a previously undone edit.
    pub fn redo(&mut self) -> Option<String> {
        if let Some(action) = self.edit_history.redo() {
            match action {
                crate::components::edit_history::EditAction::FieldChange {
                    record_idx,
                    field,
                    // The redo action is the inverted undo action: old/new are swapped.
                    // old_value here is the value we want to re-apply (the original new_value).
                    old_value,
                    new_value: _,
                } => {
                    if let Some((_, record)) =
                        self.filtered.iter_mut().find(|(i, _)| *i == record_idx)
                    {
                        let _ = record.set_field(&field, old_value.clone());
                        // Update buffer
                        if let Some(pos) =
                            R::field_descriptors().iter().position(|d| d.name == field)
                        {
                            if let Some(buf) = self.edit_buffers.get_mut(pos) {
                                *buf = old_value.clone();
                            }
                        }
                        // Update catalog
                        if let Some(catalog) = &mut self.catalog {
                            if let Some(cat_record) = catalog.get_mut(record_idx) {
                                let _ = cat_record.set_field(&field, old_value);
                            }
                        }
                        return Some(format!("Redo: {} changed", field));
                    }
                    None
                }
                crate::components::edit_history::EditAction::RecordAdd {
                    record_idx,
                    data: _,
                } => {
                    if let Some(catalog) = &mut self.catalog {
                        if record_idx < catalog.len() {
                            catalog.remove(record_idx);
                            self.refresh();
                            self.edit_history.adjust_for_removal(record_idx);
                            self.edit_buffers.clear();
                            self.selected_idx = None;
                            return Some(format!("Redo: removed record #{}", record_idx));
                        }
                    }
                    None
                }
                _ => Some("Redo: unsupported action".to_string()),
            }
        } else {
            None
        }
    }

    fn can_undo(&self) -> bool {
        self.edit_history.can_undo()
    }

    fn can_redo(&self) -> bool {
        self.edit_history.can_redo()
    }
}

impl<R: EditableRecord + Extractor> UndoRedo for GenericEditorState<R> {
    fn undo(&mut self) -> Option<String> {
        GenericEditorState::undo(self)
    }

    fn redo(&mut self) -> Option<String> {
        GenericEditorState::redo(self)
    }

    fn can_undo(&self) -> bool {
        self.edit_history.can_undo()
    }

    fn can_redo(&self) -> bool {
        self.edit_history.can_redo()
    }

    fn edit_history(&self) -> &EditHistory {
        &self.edit_history
    }
}

impl<R: EditableRecord + Extractor> GenericEditorState<R> {
    pub fn save(&self, game_path: &str, db_path: &str) -> Result<(), String> {
        // Run pre-save validation first
        self.validate_before_save()?;

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

    /// Validate all records and return a formatted error if any fail.
    ///
    /// Returns `Ok(())` when all records pass, or an `Err` with a summary
    /// like `"3 record(s) have validation errors:\n  #0 field 'foo': too large\n  ..."`.
    pub fn validate_before_save(&self) -> Result<(), String> {
        let errors = self.validate_all();
        if errors.is_empty() {
            return Ok(());
        }
        let mut msg = format!("{} record(s) have validation errors:", errors.len());
        for (record_idx, field_errors) in &errors {
            for (field, err) in field_errors {
                msg.push_str(&format!("\n  #{} field '{}': {}", record_idx, field, err));
            }
        }
        Err(msg)
    }

    /// Validate all records in the catalog and return errors grouped by record index.
    pub fn validate_all(&self) -> Vec<(usize, Vec<(&'static str, String)>)> {
        let mut errors = Vec::new();
        if let Some(catalog) = &self.catalog {
            for (idx, record) in catalog.iter().enumerate() {
                let record_errors = record.validate_all();
                if !record_errors.is_empty() {
                    errors.push((idx, record_errors));
                }
            }
        }
        errors
    }

    /// Read a file from disk.
    pub fn scan_and_read(base_path: &Path, db_path: &str) -> Result<Vec<R>, String> {
        R::read_file(&base_path.join(db_path)).map_err(|e| format!("Failed to read: {}", e))
    }

    pub fn edit_history(&self) -> &EditHistory {
        &self.edit_history
    }

    /// Build a fresh map of `TextAreaContent` for every field of `orig_idx`'s
    /// record. Called by `handle_spreadsheet_messages!` when a row is selected
    /// so the inspector's `text_editor` widgets have stable state across renders.
    pub fn make_inspector_textarea_contents(
        &self,
        orig_idx: usize,
    ) -> HashMap<String, TextAreaContent> {
        let mut map = HashMap::new();
        if let Some(catalog) = &self.catalog {
            if let Some(record) = catalog.get(orig_idx) {
                for d in R::field_descriptors() {
                    map.insert(
                        d.name.to_string(),
                        TextAreaContent::with_text(&record.get_field(d.name)),
                    );
                }
            }
        }
        map
    }

    /// Return a sorted, deduplicated list of every value that appears in
    /// column `col` (by field descriptor index) across the full catalog.
    /// Returns an empty `Vec` when `col` is out of range or no catalog is loaded.
    /// The options are sorted by frequency (most common first) with counts.
    pub fn unique_values_for_column(&self, col: usize) -> Vec<ColumnFilterOption> {
        let descriptors = R::field_descriptors();
        let Some(desc) = descriptors.get(col) else {
            return Vec::new();
        };
        let Some(catalog) = &self.catalog else {
            return Vec::new();
        };
        let mut counts = std::collections::HashMap::new();
        for record in catalog {
            let value = record.get_field(desc.name);
            *counts.entry(value).or_insert(0) += 1;
        }
        let mut options: Vec<ColumnFilterOption> = counts
            .into_iter()
            .map(|(value, count)| ColumnFilterOption { value, count })
            .collect();
        // Sort by count descending (most frequent first), then alphabetically
        options.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.value.cmp(&b.value)));
        options
    }

    pub fn init_pane_state(&mut self) {
        let (mut state, first) = pane_grid::State::new(PaneContent::ItemList);
        state.split(pane_grid::Axis::Vertical, first, PaneContent::Inspector);
        self.pane_state = Some(state);
        self.pane_focus = Some(first);
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
        if let Some((orig_idx, record)) = self.editor.filtered.get(idx).cloned() {
            // Serialize the removed record so undo can restore it
            let data = serde_json::to_string(&record).unwrap_or_default();

            // Fix up stale indices in existing history BEFORE pushing the
            // removal action — adjust_for_removal drops actions whose
            // record_idx == removed_idx, so the new RecordRemove must come
            // after the adjustment.
            self.editor.edit_history.adjust_for_removal(orig_idx);
            self.editor.edit_history.push(
                crate::components::edit_history::EditAction::RecordRemove {
                    record_idx: orig_idx,
                    data,
                },
            );

            if let Some(catalog) = &mut self.editor.catalog {
                if orig_idx < catalog.len() {
                    catalog.remove(orig_idx);
                    self.editor.refresh();
                    self.editor.selected_idx = None;
                    self.editor.edit_buffers.clear();
                }
            }
        }
    }

    pub fn undo(&mut self) -> Option<String> {
        self.editor.undo()
    }

    pub fn redo(&mut self) -> Option<String> {
        self.editor.redo()
    }

    pub fn can_undo(&self) -> bool {
        self.editor.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.editor.can_redo()
    }

    pub fn edit_history(&self) -> &EditHistory {
        self.editor.edit_history()
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
    let name_chars: Vec<char> = name.chars().collect();
    let pat_chars: Vec<char> = pattern.chars().collect();
    let mut ni = 0;
    let mut pi = 0;
    let nc = name_chars.len();
    let pc = pat_chars.len();
    let mut star_pi = None;
    let mut match_ni = 0;

    while ni < nc {
        if pi < pc && (pat_chars[pi] == '?' || pat_chars[pi] == name_chars[ni]) {
            ni += 1;
            pi += 1;
        } else if pi < pc && pat_chars[pi] == '*' {
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

    while pi < pc && pat_chars[pi] == '*' {
        pi += 1;
    }

    pi == pc
}

#[cfg(test)]
mod tests;
