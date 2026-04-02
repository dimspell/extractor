use dispel_core::WaveIni;
use dispel_core::Extractor;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct WaveIniEditorState {
    pub catalog: Option<Vec<WaveIni>>,
    pub filtered_waves: Vec<(usize, WaveIni)>,
    pub selected_idx: Option<usize>,

    pub edit_id: String,
    pub edit_snf_filename: String,
    pub edit_unknown_flag: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl WaveIniEditorState {
    pub fn refresh_waves(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered_waves = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect::<Vec<_>>();
        }
    }

    pub fn select_wave(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        if let Some((_, record)) = self.filtered_waves.get(idx) {
            self.edit_id = record.id.to_string();
            self.edit_snf_filename = record.snf_filename.clone().unwrap_or_default();
            self.edit_unknown_flag = record.unknown_flag.clone().unwrap_or_default();
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_waves.get_mut(idx).map(|(_, r)| r) {
            match field {
                "id" => {
                    self.edit_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.id = v
                    }
                }
                "snf_filename" => {
                    self.edit_snf_filename = value.clone();
                    record.snf_filename = if value.is_empty() { None } else { Some(value) };
                }
                "unknown_flag" => {
                    self.edit_unknown_flag = value.clone();
                    record.unknown_flag = if value.is_empty() { None } else { Some(value) };
                }
                _ => {}
            }
            self.refresh_waves();
        }
    }

    pub fn save_waves(&self, game_path: &str) -> Result<(), String> {
        let path = PathBuf::from(game_path).join("Wave.ini");
        if let Some(catalog) = &self.catalog {
            WaveIni::save_file(catalog, &path)
                .map_err(|e| format!("Failed to save wave ini: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
