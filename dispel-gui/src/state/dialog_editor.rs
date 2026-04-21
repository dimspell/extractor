use crate::edit_history::EditHistory;
use crate::generic_editor::{GenericEditorState, UndoRedo};
use dispel_core::Dialog;
use dispel_core::Extractor;

#[derive(Debug, Clone, Default)]
pub struct DialogEditorState {
    pub editor: GenericEditorState<Dialog>,
    pub current_file: String,
}

impl UndoRedo for DialogEditorState {
    fn undo(&mut self) -> Option<String> {
        self.editor.undo()
    }

    fn redo(&mut self) -> Option<String> {
        self.editor.redo()
    }

    fn can_undo(&self) -> bool {
        self.editor.can_undo()
    }

    fn can_redo(&self) -> bool {
        self.editor.can_redo()
    }

    fn edit_history(&self) -> &EditHistory {
        self.editor.edit_history()
    }
}

impl DialogEditorState {
    pub fn refresh_dialogs(&mut self) {
        self.editor.refresh();
    }

    pub fn select_dialog(&mut self, idx: usize) {
        self.editor.select(idx);
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        self.editor.update_field(idx, field, value);
    }

    pub fn save(&self) -> Result<(), String> {
        if self.current_file.is_empty() {
            return Err("No file selected".to_string());
        }
        if let Some(catalog) = &self.editor.catalog {
            let path = std::path::PathBuf::from(&self.current_file);
            Dialog::save_file(catalog, &path).map_err(|e| format!("Failed to save dialogs: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
