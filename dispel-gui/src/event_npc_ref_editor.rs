use dispel_core::EventNpcRef;
use dispel_core::Extractor;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct EventNpcRefEditorState {
    pub catalog: Option<Vec<EventNpcRef>>,
    pub filtered_npcs: Vec<(usize, EventNpcRef)>,
    pub selected_idx: Option<usize>,

    pub edit_id: String,
    pub edit_event_id: String,
    pub edit_name: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl EventNpcRefEditorState {
    pub fn refresh_npcs(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered_npcs = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect::<Vec<_>>();
        }
    }

    pub fn select_npc(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        if let Some((_, record)) = self.filtered_npcs.get(idx) {
            self.edit_id = record.id.to_string();
            self.edit_event_id = record.event_id.to_string();
            self.edit_name = record.name.clone();
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_npcs.get_mut(idx).map(|(_, r)| r) {
            match field {
                "id" => {
                    self.edit_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.id = v
                    }
                }
                "event_id" => {
                    self.edit_event_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.event_id = v
                    }
                }
                "name" => {
                    self.edit_name = value.clone();
                    record.name = value;
                }
                _ => {}
            }
            self.refresh_npcs();
        }
    }

    pub fn save_npcs(&self, game_path: &str) -> Result<(), String> {
        let path = PathBuf::from(game_path)
            .join("NpcInGame")
            .join("Eventnpc.ref");
        if let Some(catalog) = &self.catalog {
            EventNpcRef::save_file(catalog, &path)
                .map_err(|e| format!("Failed to save event NPCs: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
