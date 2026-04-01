use dispel_core::{Extractor, PartyIniNpc};
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct PartyIniEditorState {
    pub catalog: Option<Vec<PartyIniNpc>>,
    pub filtered_npcs: Vec<(usize, PartyIniNpc)>,
    pub selected_idx: Option<usize>,

    pub edit_name: String,
    pub edit_unknown1: String,
    pub edit_unknown2: String,
    pub edit_unknown3: String,
    pub edit_unknown4: String,
    pub edit_unknown5: String,
    pub edit_unknown6: String,

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
            self.edit_unknown1 = record.unknown1.to_string();
            self.edit_unknown2 = record.unknown2.to_string();
            self.edit_unknown3 = record.unknown3.to_string();
            self.edit_unknown4 = record.unknown4.to_string();
            self.edit_unknown5 = record.unknown5.to_string();
            self.edit_unknown6 = record.unknown6.to_string();
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_npcs.get_mut(idx).map(|(_, r)| r) {
            match field {
                "name" => {
                    self.edit_name = value.clone();
                    record.name = value;
                }
                "unknown1" => {
                    self.edit_unknown1 = value.clone();
                    if let Ok(v) = value.parse() {
                        record.unknown1 = v
                    }
                }
                "unknown2" => {
                    self.edit_unknown2 = value.clone();
                    if let Ok(v) = value.parse() {
                        record.unknown2 = v
                    }
                }
                "unknown3" => {
                    self.edit_unknown3 = value.clone();
                    if let Ok(v) = value.parse() {
                        record.unknown3 = v
                    }
                }
                "unknown4" => {
                    self.edit_unknown4 = value.clone();
                    if let Ok(v) = value.parse() {
                        record.unknown4 = v
                    }
                }
                "unknown5" => {
                    self.edit_unknown5 = value.clone();
                    if let Ok(v) = value.parse() {
                        record.unknown5 = v
                    }
                }
                "unknown6" => {
                    self.edit_unknown6 = value.clone();
                    if let Ok(v) = value.parse() {
                        record.unknown6 = v
                    }
                }
                _ => {}
            }
            self.refresh_npcs();
        }
    }

    pub fn save_npcs(&self, game_path: &str) -> Result<(), String> {
        let path = PathBuf::from(game_path).join("NpcInGame").join("PrtIni.db");
        if let Some(catalog) = &self.catalog {
            PartyIniNpc::save_file(catalog, &path)
                .map_err(|e| format!("Failed to save party ini: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
