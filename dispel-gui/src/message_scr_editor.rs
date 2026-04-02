use dispel_core::Message;
use dispel_core::Extractor;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct MessageScrEditorState {
    pub catalog: Option<Vec<Message>>,
    pub filtered_messages: Vec<(usize, Message)>,
    pub selected_idx: Option<usize>,

    pub edit_id: String,
    pub edit_line1: String,
    pub edit_line2: String,
    pub edit_line3: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl MessageScrEditorState {
    pub fn refresh_messages(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered_messages = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect::<Vec<_>>();
        }
    }

    pub fn select_message(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        if let Some((_, record)) = self.filtered_messages.get(idx) {
            self.edit_id = record.id.to_string();
            self.edit_line1 = record.line1.clone().unwrap_or_default();
            self.edit_line2 = record.line2.clone().unwrap_or_default();
            self.edit_line3 = record.line3.clone().unwrap_or_default();
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_messages.get_mut(idx).map(|(_, r)| r) {
            match field {
                "id" => {
                    self.edit_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.id = v
                    }
                }
                "line1" => {
                    self.edit_line1 = value.clone();
                    record.line1 = if value.is_empty() { None } else { Some(value) };
                }
                "line2" => {
                    self.edit_line2 = value.clone();
                    record.line2 = if value.is_empty() { None } else { Some(value) };
                }
                "line3" => {
                    self.edit_line3 = value.clone();
                    record.line3 = if value.is_empty() { None } else { Some(value) };
                }
                _ => {}
            }
            self.refresh_messages();
        }
    }

    pub fn save_messages(&self, game_path: &str) -> Result<(), String> {
        let path = PathBuf::from(game_path)
            .join("ExtraInGame")
            .join("Message.scr");
        if let Some(catalog) = &self.catalog {
            Message::save_file(catalog, &path)
                .map_err(|e| format!("Failed to save messages: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
