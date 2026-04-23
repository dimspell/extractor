use crate::edit_history::EditHistory;
use crate::generic_editor::{GenericEditorState, UndoRedo};
use dispel_core::DialogueParagraph;
use dispel_core::Extractor;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct DialogueParagraphEditorState {
    pub editor: GenericEditorState<DialogueParagraph>,
    pub current_file: String,
}

impl UndoRedo for DialogueParagraphEditorState {
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

impl DialogueParagraphEditorState {
    pub fn refresh(&mut self) {
        self.editor.refresh();
    }

    pub fn select(&mut self, idx: usize) {
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
            let path = PathBuf::from(&self.current_file);
            DialogueParagraph::save_file(catalog, &path)
                .map_err(|e| format!("Failed to save texts: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
