use dispel_core::{Extractor, PartyIniNpc};
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct PartyIniEditorState {
    pub game_path: String,
    pub catalog: Option<Vec<PartyIniNpc>>,
    pub filtered_npcs: Vec<(usize, PartyIniNpc)>,
    pub selected_idx: Option<usize>,

    pub edit_name: String,
    pub edit_flags: String,
    pub edit_kind: String,
    pub edit_value: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl PartyIniEditorState {
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
            self.edit_name = record.name.clone();
            self.edit_flags = record.flags.to_string();
            self.edit_kind = record.kind.to_string();
            self.edit_value = record.value.to_string();
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_npcs.get_mut(idx).map(|(_, r)| r) {
            match field {
                "name" => record.name = value.clone(),
                "flags" => {
                    if let Ok(v) = value.parse() {
                        record.flags = v
                    }
                }
                "kind" => {
                    if let Ok(v) = value.parse() {
                        record.kind = v
                    }
                }
                "value" => {
                    if let Ok(v) = value.parse() {
                        record.value = v
                    }
                }
                _ => {}
            }
            self.refresh_npcs();
        }
    }

    pub fn save_npcs(&self) -> Result<(), String> {
        let path = PathBuf::from(&self.game_path)
            .join("NpcInGame")
            .join("PrtIni.db");
        if let Some(catalog) = &self.catalog {
            PartyIniNpc::save_file(catalog, &path)
                .map_err(|e| format!("Failed to save party ini: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
