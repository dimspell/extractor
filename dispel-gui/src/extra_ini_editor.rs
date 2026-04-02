use dispel_core::Extra;
use dispel_core::Extractor;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct ExtraIniEditorState {
    pub catalog: Option<Vec<Extra>>,
    pub filtered_extras: Vec<(usize, Extra)>,
    pub selected_idx: Option<usize>,

    pub edit_id: String,
    pub edit_sprite_filename: String,
    pub edit_unknown: String,
    pub edit_description: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl ExtraIniEditorState {
    pub fn refresh_extras(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered_extras = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect::<Vec<_>>();
        }
    }

    pub fn select_extra(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        if let Some((_, record)) = self.filtered_extras.get(idx) {
            self.edit_id = record.id.to_string();
            self.edit_sprite_filename = record.sprite_filename.clone().unwrap_or_default();
            self.edit_unknown = record.unknown.to_string();
            self.edit_description = record.description.clone().unwrap_or_default();
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_extras.get_mut(idx).map(|(_, r)| r) {
            match field {
                "id" => {
                    self.edit_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.id = v
                    }
                }
                "sprite_filename" => {
                    self.edit_sprite_filename = value.clone();
                    record.sprite_filename = if value.is_empty() { None } else { Some(value) };
                }
                "unknown" => {
                    self.edit_unknown = value.clone();
                    if let Ok(v) = value.parse() {
                        record.unknown = v
                    }
                }
                "description" => {
                    self.edit_description = value.clone();
                    record.description = if value.is_empty() { None } else { Some(value) };
                }
                _ => {}
            }
            self.refresh_extras();
        }
    }

    pub fn save_extras(&self, game_path: &str) -> Result<(), String> {
        let path = PathBuf::from(game_path).join("Extra.ini");
        if let Some(catalog) = &self.catalog {
            Extra::save_file(catalog, &path).map_err(|e| format!("Failed to save extras: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
