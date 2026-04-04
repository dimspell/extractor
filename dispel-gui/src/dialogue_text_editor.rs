use crate::edit_history::EditHistory;
use crate::generic_editor::UndoRedo;
use dispel_core::DialogueText;
use dispel_core::Extractor;
use iced::widget::text_editor;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct DialogueTextEditorState {
    pub catalog: Option<Vec<DialogueText>>,
    pub filtered_texts: Vec<(usize, DialogueText)>,
    pub selected_idx: Option<usize>,
    pub current_file: String,
    pub text_files: Vec<PathBuf>,

    pub edit_id: String,
    pub edit_text_content: text_editor::Content,
    pub edit_comment_content: text_editor::Content,
    pub edit_param1: String,
    pub edit_wave_ini_entry_id: String,

    pub status_msg: String,
    pub is_loading: bool,
    pub edit_history: EditHistory,
}

impl UndoRedo for DialogueTextEditorState {
    fn undo(&mut self) -> Option<String> {
        if let Some(action) = self.edit_history.undo() {
            Some(format!("Undid: {:?}", action))
        } else {
            None
        }
    }

    fn redo(&mut self) -> Option<String> {
        if let Some(action) = self.edit_history.redo() {
            Some(format!("Redid: {:?}", action))
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

    fn edit_history(&self) -> &EditHistory {
        &self.edit_history
    }
}

impl DialogueTextEditorState {
    pub fn refresh_texts(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered_texts = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect::<Vec<_>>();
        }
    }

    pub fn select_text(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        if let Some((_, record)) = self.filtered_texts.get(idx) {
            self.edit_id = record.id.to_string();
            self.edit_text_content = text_editor::Content::with_text(&record.text);
            self.edit_comment_content = text_editor::Content::with_text(&record.comment);
            self.edit_param1 = record.param1.to_string();
            self.edit_wave_ini_entry_id = record.wave_ini_entry_id.to_string();
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_texts.get_mut(idx).map(|(_, r)| r) {
            match field {
                "id" => {
                    self.edit_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.id = v
                    }
                }
                "param1" => {
                    self.edit_param1 = value.clone();
                    if let Ok(v) = value.parse() {
                        record.param1 = v
                    }
                }
                "wave_ini_entry_id" => {
                    self.edit_wave_ini_entry_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.wave_ini_entry_id = v
                    }
                }
                _ => {}
            }
            self.refresh_texts();
        }
    }

    pub fn update_text_content(&mut self, idx: usize) {
        if let Some(record) = self.filtered_texts.get_mut(idx).map(|(_, r)| r) {
            record.text = self.edit_text_content.text().to_string();
        }
    }

    pub fn update_comment_content(&mut self, idx: usize) {
        if let Some(record) = self.filtered_texts.get_mut(idx).map(|(_, r)| r) {
            record.comment = self.edit_comment_content.text().to_string();
        }
    }

    pub fn save_texts(&self, game_path: &str, filename: &str) -> Result<(), String> {
        let path = PathBuf::from(game_path).join(filename);
        if let Some(catalog) = &self.catalog {
            DialogueText::save_file(catalog, &path)
                .map_err(|e| format!("Failed to save texts: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
