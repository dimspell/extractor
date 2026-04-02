use dispel_core::DialogueText;
use dispel_core::Extractor;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct DialogueTextEditorState {
    pub catalog: Option<Vec<DialogueText>>,
    pub filtered_texts: Vec<(usize, DialogueText)>,
    pub selected_idx: Option<usize>,
    pub current_file: String,

    pub edit_id: String,
    pub edit_text: String,
    pub edit_comment: String,
    pub edit_param1: String,
    pub edit_wave_ini_entry_id: String,

    pub status_msg: String,
    pub is_loading: bool,
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
            self.edit_text = record.text.clone();
            self.edit_comment = record.comment.clone();
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
                "text" => {
                    self.edit_text = value.clone();
                    record.text = value;
                }
                "comment" => {
                    self.edit_comment = value.clone();
                    record.comment = value;
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
