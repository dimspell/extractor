use dispel_core::{Extractor, PartyRef};
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct PartyRefEditorState {
    pub game_path: String,
    pub catalog: Option<Vec<PartyRef>>,
    pub filtered_party: Vec<(usize, PartyRef)>,
    pub selected_idx: Option<usize>,

    pub edit_full_name: String,
    pub edit_job_name: String,
    pub edit_root_map_id: String,
    pub edit_npc_id: String,
    pub edit_dlg_not_in_party: String,
    pub edit_dlg_in_party: String,
    pub edit_ghost_face_id: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl PartyRefEditorState {
    pub fn refresh_party(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered_party = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect::<Vec<_>>();
        }
    }

    pub fn select_member(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        if let Some((_, record)) = self.filtered_party.get(idx) {
            self.edit_full_name = record.full_name.clone().unwrap_or_default();
            self.edit_job_name = record.job_name.clone().unwrap_or_default();
            self.edit_root_map_id = record.root_map_id.to_string();
            self.edit_npc_id = record.npc_id.to_string();
            self.edit_dlg_not_in_party = record.dlg_when_not_in_party.to_string();
            self.edit_dlg_in_party = record.dlg_when_in_party.to_string();
            self.edit_ghost_face_id = format!("{:?}", record.ghost_face_id);
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_party.get_mut(idx).map(|(_, r)| r) {
            match field {
                "full_name" => {
                    self.edit_full_name = value.clone();
                    record.full_name = if value.is_empty() { None } else { Some(value) };
                }
                "job_name" => {
                    self.edit_job_name = value.clone();
                    record.job_name = if value.is_empty() { None } else { Some(value) };
                }
                "root_map_id" => {
                    self.edit_root_map_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.root_map_id = v
                    }
                }
                "npc_id" => {
                    self.edit_npc_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.npc_id = v
                    }
                }
                "dlg_when_not_in_party" => {
                    self.edit_dlg_not_in_party = value.clone();
                    if let Ok(v) = value.parse() {
                        record.dlg_when_not_in_party = v
                    }
                }
                "dlg_when_in_party" => {
                    self.edit_dlg_in_party = value.clone();
                    if let Ok(v) = value.parse() {
                        record.dlg_when_in_party = v
                    }
                }
                _ => {}
            }
            self.refresh_party();
        }
    }

    pub fn save_party(&self) -> Result<(), String> {
        let path = PathBuf::from(&self.game_path)
            .join("Ref")
            .join("PartyRef.ref");
        if let Some(catalog) = &self.catalog {
            PartyRef::save_file(catalog, &path)
                .map_err(|e| format!("Failed to save party refs: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
