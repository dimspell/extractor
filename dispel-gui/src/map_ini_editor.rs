use dispel_core::MapIni;
use dispel_core::Extractor;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct MapIniEditorState {
    pub catalog: Option<Vec<MapIni>>,
    pub filtered_maps: Vec<(usize, MapIni)>,
    pub selected_idx: Option<usize>,

    pub edit_id: String,
    pub edit_event_id_on_camera_move: String,
    pub edit_start_pos_x: String,
    pub edit_start_pos_y: String,
    pub edit_map_id: String,
    pub edit_monsters_filename: String,
    pub edit_npc_filename: String,
    pub edit_extra_filename: String,
    pub edit_cd_music_track_number: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl MapIniEditorState {
    pub fn refresh_maps(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered_maps = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect::<Vec<_>>();
        }
    }

    pub fn select_map(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        if let Some((_, record)) = self.filtered_maps.get(idx) {
            self.edit_id = record.id.to_string();
            self.edit_event_id_on_camera_move = record.event_id_on_camera_move.to_string();
            self.edit_start_pos_x = record.start_pos_x.to_string();
            self.edit_start_pos_y = record.start_pos_y.to_string();
            self.edit_map_id = record.map_id.to_string();
            self.edit_monsters_filename = record.monsters_filename.clone().unwrap_or_default();
            self.edit_npc_filename = record.npc_filename.clone().unwrap_or_default();
            self.edit_extra_filename = record.extra_filename.clone().unwrap_or_default();
            self.edit_cd_music_track_number = record.cd_music_track_number.to_string();
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_maps.get_mut(idx).map(|(_, r)| r) {
            match field {
                "id" => {
                    self.edit_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.id = v
                    }
                }
                "event_id_on_camera_move" => {
                    self.edit_event_id_on_camera_move = value.clone();
                    if let Ok(v) = value.parse() {
                        record.event_id_on_camera_move = v
                    }
                }
                "start_pos_x" => {
                    self.edit_start_pos_x = value.clone();
                    if let Ok(v) = value.parse() {
                        record.start_pos_x = v
                    }
                }
                "start_pos_y" => {
                    self.edit_start_pos_y = value.clone();
                    if let Ok(v) = value.parse() {
                        record.start_pos_y = v
                    }
                }
                "map_id" => {
                    self.edit_map_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.map_id = v
                    }
                }
                "monsters_filename" => {
                    self.edit_monsters_filename = value.clone();
                    record.monsters_filename = if value.is_empty() { None } else { Some(value) };
                }
                "npc_filename" => {
                    self.edit_npc_filename = value.clone();
                    record.npc_filename = if value.is_empty() { None } else { Some(value) };
                }
                "extra_filename" => {
                    self.edit_extra_filename = value.clone();
                    record.extra_filename = if value.is_empty() { None } else { Some(value) };
                }
                "cd_music_track_number" => {
                    self.edit_cd_music_track_number = value.clone();
                    if let Ok(v) = value.parse() {
                        record.cd_music_track_number = v
                    }
                }
                _ => {}
            }
            self.refresh_maps();
        }
    }

    pub fn save_maps(&self, game_path: &str) -> Result<(), String> {
        let path = PathBuf::from(game_path).join("Ref").join("Map.ini");
        if let Some(catalog) = &self.catalog {
            MapIni::save_file(catalog, &path).map_err(|e| format!("Failed to save map ini: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
