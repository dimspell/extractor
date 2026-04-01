use dispel_core::{EventItem, Extractor};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct EventItemEditorState {
    pub catalog: Option<Vec<EventItem>>,
    pub filtered_items: Vec<(usize, EventItem)>,
    pub selected_idx: Option<usize>,

    pub edit_name: String,
    pub edit_description: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl EventItemEditorState {
    pub fn refresh_items(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered_items = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect::<Vec<_>>();
        }
    }

    pub fn select_item(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        if let Some((_, record)) = self.filtered_items.get(idx) {
            self.edit_name = record.name.clone();
            self.edit_description = record.description.clone();
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_items.get_mut(idx).map(|(_, r)| r) {
            match field {
                "name" => {
                    self.edit_name = value.clone();
                    record.name = value;
                }
                "description" => {
                    self.edit_description = value.clone();
                    record.description = value;
                }
                _ => {}
            }
            self.refresh_items();
        }
    }

    pub fn save_items(&self, game_path: &str) -> Result<(), String> {
        let path = PathBuf::from(game_path)
            .join("CharacterInGame")
            .join("EventItem.db");
        if let Some(catalog) = &self.catalog {
            EventItem::save_file(catalog, &path)
                .map_err(|e| format!("Failed to save event items: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }

    pub fn scan_and_read(path: &Path) -> Result<Vec<EventItem>, String> {
        EventItem::read_file(&path.join("CharacterInGame").join("EventItem.db"))
            .map_err(|e: std::io::Error| format!("Failed to read event items: {}", e))
    }
}
