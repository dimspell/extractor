use crate::components::editable::EditableRecord;
use crate::components::generic_editor::GenericEditorState;
use crate::view::editor::SpreadsheetState;
use std::ops::{Deref, DerefMut};

/// Bundles a `GenericEditorState<T>` with its `SpreadsheetState`.
///
/// They always travel together — every standard editor needs both, and
/// updating one without the other has caused bugs in the past. This wrapper
/// makes the pairing explicit and removes ~20 paired field declarations from
/// `AppState`.
///
/// Implements `Deref` / `DerefMut` to `GenericEditorState<T>` so existing
/// code that calls `.catalog`, `.select(idx)`, `.refresh()`, etc. on the
/// editor continues to work without changes.
pub struct StandardEditor<T: EditableRecord> {
    pub state: GenericEditorState<T>,
    pub spreadsheet: SpreadsheetState,
}

impl<T: EditableRecord + Default> Default for StandardEditor<T> {
    fn default() -> Self {
        Self {
            state: GenericEditorState::default(),
            spreadsheet: SpreadsheetState::new(),
        }
    }
}

impl<T: EditableRecord> Deref for StandardEditor<T> {
    type Target = GenericEditorState<T>;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<T: EditableRecord> DerefMut for StandardEditor<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}
