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
mod tests {
    use super::*;
    use crate::components::edit_history::EditAction;
    use crate::components::editable::EditableRecord;
    use dispel_core::{EditItem, EventItem, HealItem, MiscItem, Monster, PartyRef, WeaponItem};

    #[test]
    fn test_weapon_editor_migration() {
        // Test that WeaponEditorState (now GenericEditorState<WeaponItem>) works correctly
        let mut editor = GenericEditorState::<WeaponItem>::default();

        // Should start with no catalog
        assert!(editor.catalog.is_none());
        assert!(editor.filtered.is_empty());
        assert!(editor.selected_idx.is_none());

        // Test that we can create a catalog
        let mut catalog = Vec::new();
        let mut weapon = WeaponItem::default();
        weapon.name = "Test Sword".to_string();
        weapon.base_price = 100;
        catalog.push(weapon);

        editor.catalog = Some(catalog);
        editor.refresh();

        // Should now have filtered items
        assert_eq!(editor.filtered.len(), 1);
        assert_eq!(editor.filtered[0].1.name, "Test Sword");
    }

    #[test]
    fn test_monster_editor_migration() {
        // Test that MonsterEditorState (now GenericEditorState<Monster>) works correctly
        let mut editor = GenericEditorState::<Monster>::default();

        // Should start with no catalog
        assert!(editor.catalog.is_none());
        assert!(editor.filtered.is_empty());
        assert!(editor.selected_idx.is_none());

        // Test that we can create a catalog
        let mut catalog = Vec::new();
        let mut monster = Monster::default();
        monster.name = "Test Monster".to_string();
        monster.health_points_max = 50;
        catalog.push(monster);

        editor.catalog = Some(catalog);
        editor.refresh();

        // Should now have filtered items
        assert_eq!(editor.filtered.len(), 1);
        assert_eq!(editor.filtered[0].1.name, "Test Monster");
    }

    #[test]
    fn test_simple_editor_migrations() {
        // Test each editor type individually to avoid type issues
        let editor1 = GenericEditorState::<EditItem>::default();
        assert!(editor1.catalog.is_none());
        assert!(editor1.filtered.is_empty());

        let editor2 = GenericEditorState::<EventItem>::default();
        assert!(editor2.catalog.is_none());
        assert!(editor2.filtered.is_empty());

        let editor3 = GenericEditorState::<HealItem>::default();
        assert!(editor3.catalog.is_none());
        assert!(editor3.filtered.is_empty());

        let editor4 = GenericEditorState::<MiscItem>::default();
        assert!(editor4.catalog.is_none());
        assert!(editor4.filtered.is_empty());

        let editor5 = GenericEditorState::<PartyRef>::default();
        assert!(editor5.catalog.is_none());
        assert!(editor5.filtered.is_empty());
    }

    #[test]
    fn test_editor_selection_functionality() {
        // Test that selection works correctly in migrated editors
        let mut editor = GenericEditorState::<WeaponItem>::default();

        // Create a catalog with multiple items
        let mut catalog = Vec::new();
        for i in 0..3 {
            let mut weapon = WeaponItem::default();
            weapon.name = format!("Weapon {}", i);
            weapon.base_price = 100 + i * 10;
            catalog.push(weapon);
        }

        editor.catalog = Some(catalog);
        editor.refresh();

        // Select the second item
        editor.select(1);

        // Should have selected index and edit buffers populated
        assert_eq!(editor.selected_idx, Some(1));
        assert_eq!(
            editor.edit_buffers.len(),
            WeaponItem::field_descriptors().len()
        );

        // The name buffer should match the selected weapon's name
        let name_pos = WeaponItem::field_descriptors()
            .iter()
            .position(|d| d.name == "name")
            .expect("name field should exist");

        assert_eq!(editor.edit_buffers[name_pos], "Weapon 1");
    }

    #[test]
    fn test_editor_field_update_functionality() {
        // Test that field updates work correctly in migrated editors
        let mut editor = GenericEditorState::<WeaponItem>::default();

        // Create a catalog
        let mut catalog = Vec::new();
        let mut weapon = WeaponItem::default();
        weapon.name = "Original Name".to_string();
        weapon.base_price = 100;
        catalog.push(weapon);

        editor.catalog = Some(catalog);
        editor.refresh();
        editor.select(0);

        // Update the name field
        let result = editor.update_field(0, "name", "Updated Name".to_string());
        assert!(result, "Field update should succeed");

        // Check that the catalog was updated
        if let Some(catalog) = &editor.catalog {
            assert_eq!(catalog[0].name, "Updated Name");
        } else {
            panic!("Catalog should exist");
        }

        // Check that the filtered list was updated
        assert_eq!(editor.filtered[0].1.name, "Updated Name");

        // Check that the edit buffer was updated
        let name_pos = WeaponItem::field_descriptors()
            .iter()
            .position(|d| d.name == "name")
            .expect("name field should exist");
        assert_eq!(editor.edit_buffers[name_pos], "Updated Name");
    }

    #[test]
    fn test_field_validation_works() {
        // Test that field validation works correctly in migrated editors
        let mut editor = GenericEditorState::<WeaponItem>::default();

        // Create a catalog
        let mut catalog = Vec::new();
        let weapon = WeaponItem::default();
        catalog.push(weapon);

        editor.catalog = Some(catalog);
        editor.refresh();
        editor.select(0);

        // Test valid integer field update
        let result = editor.update_field(0, "base_price", "250".to_string());
        assert!(result, "Valid integer field should be accepted");

        // Test invalid integer field update
        let result = editor.update_field(0, "base_price", "not_a_number".to_string());
        assert!(!result, "Invalid integer field should be rejected");

        // Test valid string field update
        let result = editor.update_field(0, "name", "Valid Name".to_string());
        assert!(result, "Valid string field should be accepted");
    }

    #[test]
    fn test_update_field_by_catalog_index() {
        use dispel_core::WeaponItem;
        let mut editor = GenericEditorState::<WeaponItem>::default();
        let mut catalog = Vec::new();
        for i in 0..5 {
            let mut w = WeaponItem::default();
            w.name = format!("Weapon {}", i);
            catalog.push(w);
        }
        editor.catalog = Some(catalog);
        editor.refresh();

        // filtered has [(0, W0), (1, W1), (2, W2), (3, W3), (4, W4)].
        // Update using catalog index 3 directly.
        let r = editor.update_field(3, "name", "Updated".to_string());
        assert!(r);
        assert_eq!(editor.catalog.as_ref().unwrap()[3].name, "Updated");
        assert_eq!(
            editor
                .filtered
                .iter()
                .find(|(i, _)| *i == 3)
                .unwrap()
                .1
                .name,
            "Updated"
        );
        assert!(editor.edit_history.can_undo());
    }

    #[test]
    fn test_update_field_works_despite_non_matching_filtered_position() {
        use dispel_core::WeaponItem;
        let mut editor = GenericEditorState::<WeaponItem>::default();
        let mut catalog = Vec::new();
        for i in 0..5 {
            let mut w = WeaponItem::default();
            w.name = format!("Weapon {}", i);
            catalog.push(w);
        }
        editor.catalog = Some(catalog);
        editor.refresh();

        // Remove the first element from `filtered` to simulate a filter
        // where the visible set differs from catalog indices.
        editor.filtered.remove(0);
        // filtered now has [(1, W1), (2, W2), (3, W3), (4, W4)].

        // Catalog index 3 (Weapon 3) is still present in filtered at
        // filtered position 2. update_field should find it by catalog
        // index, NOT by filtered position.
        let r = editor.update_field(3, "name", "Patched".to_string());
        assert!(r);
        assert_eq!(editor.catalog.as_ref().unwrap()[3].name, "Patched");

        // Editing filtered position 2 should correctly map to catalog index 3
        let r2 = editor.update_field(3, "name", "Again".to_string());
        assert!(r2);
        assert_eq!(editor.catalog.as_ref().unwrap()[3].name, "Again");
    }

    #[test]
    fn test_undo_redo_remove_record() {
        use dispel_core::WeaponItem;
        let mut editor = GenericEditorState::<WeaponItem>::default();
        let mut catalog = Vec::new();
        for i in 0..3 {
            let mut w = WeaponItem::default();
            w.name = format!("Weapon {}", i);
            catalog.push(w);
        }
        editor.catalog = Some(catalog);
        editor.refresh();

        // Simulate removing record at catalog index 1
        let record = editor
            .filtered
            .iter()
            .find(|(i, _)| *i == 1)
            .unwrap()
            .1
            .clone();
        let data = serde_json::to_string(&record).unwrap();
        // Must adjust BEFORE push (adjust_for_removal drops actions at removed_idx)
        editor.edit_history.adjust_for_removal(1);
        editor.edit_history.push(EditAction::RecordRemove {
            record_idx: 1,
            data,
        });
        editor.catalog.as_mut().unwrap().remove(1);
        editor.refresh();

        assert_eq!(editor.catalog.as_ref().unwrap().len(), 2);
        assert_eq!(editor.catalog.as_ref().unwrap()[1].name, "Weapon 2");

        // Undo: should restore the removed record
        let msg = editor.undo();
        let msg = msg.expect("undo should return Some message");
        assert!(msg.starts_with("Undo: restored"));
        assert_eq!(editor.catalog.as_ref().unwrap().len(), 3);
        assert_eq!(editor.catalog.as_ref().unwrap()[1].name, "Weapon 1");

        // Redo: should remove it again
        let msg = editor.redo();
        assert!(msg.unwrap().starts_with("Redo: removed"));
        assert_eq!(editor.catalog.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_undo_adjusts_history_after_remove() {
        use dispel_core::WeaponItem;
        let mut editor = GenericEditorState::<WeaponItem>::default();
        let mut catalog = Vec::new();
        for i in 0..4 {
            let mut w = WeaponItem::default();
            w.name = format!("Weapon {}", i);
            catalog.push(w);
        }
        editor.catalog = Some(catalog);
        editor.refresh();

        // Edit record at catalog index 3 (Weapon 3)
        editor.update_field(3, "name", "Edited".to_string());
        assert!(editor.edit_history.can_undo());

        // Simulate removing record at catalog index 1
        let record = editor
            .filtered
            .iter()
            .find(|(i, _)| *i == 1)
            .unwrap()
            .1
            .clone();
        let data = serde_json::to_string(&record).unwrap();
        editor.edit_history.adjust_for_removal(1);
        editor.edit_history.push(EditAction::RecordRemove {
            record_idx: 1,
            data,
        });
        editor.catalog.as_mut().unwrap().remove(1);
        editor.refresh();

        // The FieldChange for Weapon 3 should have its record_idx
        // decremented from 3 to 2 by adjust_for_removal(1).
        // The RecordRemove is at the front (most recent action).
        let stack = editor.edit_history.undo_stack();
        assert_eq!(stack.len(), 2, "should have RecordRemove + FieldChange");
        match &stack[1] {
            EditAction::FieldChange {
                record_idx, field, ..
            } => {
                assert_eq!(*record_idx, 2, "index should be decremented after removal");
                assert_eq!(field, "name");
            }
            _ => panic!("expected FieldChange at back of stack"),
        }
        match &stack[0] {
            EditAction::RecordRemove { record_idx, .. } => {
                assert_eq!(*record_idx, 1);
            }
            _ => panic!("expected RecordRemove at front of stack"),
        }
    }

    #[test]
    fn test_edit_history_adjust_for_addition() {
        let mut history = EditHistory::new();
        history.push(EditAction::FieldChange {
            record_idx: 0,
            field: "f".into(),
            old_value: "a".into(),
            new_value: "b".into(),
        });
        history.push(EditAction::FieldChange {
            record_idx: 2,
            field: "f".into(),
            old_value: "c".into(),
            new_value: "d".into(),
        });
        history.push(EditAction::FieldChange {
            record_idx: 5,
            field: "f".into(),
            old_value: "e".into(),
            new_value: "f".into(),
        });

        history.adjust_for_addition(2);

        let stack = history.undo_stack();
        // After addition at index 2: indices >= 2 incremented
        assert_eq!(stack[0].record_idx(), 6); // 5 → 6
        assert_eq!(stack[1].record_idx(), 3); // 2 → 3
        assert_eq!(stack[2].record_idx(), 0); // 0 unchanged
    }

    #[test]
    fn test_edit_history_adjust_for_removal_drops_matching() {
        let mut history = EditHistory::new();
        history.push(EditAction::FieldChange {
            record_idx: 0,
            field: "f".into(),
            old_value: "a".into(),
            new_value: "b".into(),
        });
        history.push(EditAction::RecordRemove {
            record_idx: 2,
            data: "{}".into(),
        });
        history.push(EditAction::FieldChange {
            record_idx: 5,
            field: "f".into(),
            old_value: "c".into(),
            new_value: "d".into(),
        });

        history.adjust_for_removal(2);

        let stack = history.undo_stack();
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0].record_idx(), 4); // 5 → 4
        assert_eq!(stack[1].record_idx(), 0); // 0 unchanged
                                              // RecordRemove at idx 2 was dropped
    }
}
