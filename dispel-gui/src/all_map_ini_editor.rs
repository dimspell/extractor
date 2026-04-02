use dispel_core::Map;
use dispel_core::Extractor;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct AllMapIniEditorState {
    pub catalog: Option<Vec<Map>>,
    pub filtered_maps: Vec<(usize, Map)>,
    pub selected_idx: Option<usize>,

    pub edit_id: String,
    pub edit_map_filename: String,
    pub edit_map_name: String,
    pub edit_pgp_filename: String,
    pub edit_dlg_filename: String,
    pub edit_lighting: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl AllMapIniEditorState {
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
            self.edit_map_filename = record.map_filename.clone();
            self.edit_map_name = record.map_name.clone();
            self.edit_pgp_filename = record.pgp_filename.clone().unwrap_or_default();
            self.edit_dlg_filename = record.dlg_filename.clone().unwrap_or_default();
            self.edit_lighting = format!("{:?}", record.lighting);
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
                "map_filename" => {
                    self.edit_map_filename = value.clone();
                    record.map_filename = value;
                }
                "map_name" => {
                    self.edit_map_name = value.clone();
                    record.map_name = value;
                }
                "pgp_filename" => {
                    self.edit_pgp_filename = value.clone();
                    record.pgp_filename = if value.is_empty() { None } else { Some(value) };
                }
                "dlg_filename" => {
                    self.edit_dlg_filename = value.clone();
                    record.dlg_filename = if value.is_empty() { None } else { Some(value) };
                }
                "lighting" => {
                    self.edit_lighting = value.clone();
                    if value.contains("Light") {
                        record.lighting = dispel_core::MapLighting::Light;
                    } else if value.contains("Dark") {
                        record.lighting = dispel_core::MapLighting::Dark;
                    }
                }
                _ => {}
            }
            self.refresh_maps();
        }
    }

    pub fn save_maps(&self, game_path: &str) -> Result<(), String> {
        let path = PathBuf::from(game_path).join("AllMap.ini");
        if let Some(catalog) = &self.catalog {
            Map::save_file(catalog, &path).map_err(|e| format!("Failed to save maps: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
