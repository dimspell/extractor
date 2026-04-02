use dispel_core::Quest;
use dispel_core::Extractor;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct QuestScrEditorState {
    pub catalog: Option<Vec<Quest>>,
    pub filtered_quests: Vec<(usize, Quest)>,
    pub selected_idx: Option<usize>,

    pub edit_id: String,
    pub edit_type_id: String,
    pub edit_title: String,
    pub edit_description: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl QuestScrEditorState {
    pub fn refresh_quests(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered_quests = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect::<Vec<_>>();
        }
    }

    pub fn select_quest(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        if let Some((_, record)) = self.filtered_quests.get(idx) {
            self.edit_id = record.id.to_string();
            self.edit_type_id = record.type_id.to_string();
            self.edit_title = record.title.clone().unwrap_or_default();
            self.edit_description = record.description.clone().unwrap_or_default();
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_quests.get_mut(idx).map(|(_, r)| r) {
            match field {
                "id" => {
                    self.edit_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.id = v
                    }
                }
                "type_id" => {
                    self.edit_type_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.type_id = v
                    }
                }
                "title" => {
                    self.edit_title = value.clone();
                    record.title = if value.is_empty() { None } else { Some(value) };
                }
                "description" => {
                    self.edit_description = value.clone();
                    record.description = if value.is_empty() { None } else { Some(value) };
                }
                _ => {}
            }
            self.refresh_quests();
        }
    }

    pub fn save_quests(&self, game_path: &str) -> Result<(), String> {
        let path = PathBuf::from(game_path)
            .join("ExtraInGame")
            .join("Quest.scr");
        if let Some(catalog) = &self.catalog {
            Quest::save_file(catalog, &path).map_err(|e| format!("Failed to save quests: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
