use dispel_core::ChData;
use dispel_core::Extractor;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct ChDataEditorState {
    pub catalog: Option<Vec<ChData>>,

    pub edit_magic: String,
    pub edit_values: String,
    pub edit_counts: String,
    pub edit_total: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl ChDataEditorState {
    pub fn select_data(&mut self) {
        if let Some(catalog) = &self.catalog {
            if let Some(record) = catalog.first() {
                self.edit_magic = record.magic.clone();
                self.edit_values = record
                    .values
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                self.edit_counts = record
                    .counts
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                self.edit_total = record.total.to_string();
            }
        }
    }

    pub fn update_field(&mut self, field: &str, value: String) {
        if let Some(catalog) = &mut self.catalog {
            if let Some(record) = catalog.first_mut() {
                match field {
                    "magic" => {
                        self.edit_magic = value.clone();
                        record.magic = value;
                    }
                    "values" => {
                        self.edit_values = value.clone();
                        record.values = value
                            .split(',')
                            .filter_map(|s| s.trim().parse::<u16>().ok())
                            .collect();
                    }
                    "counts" => {
                        self.edit_counts = value.clone();
                        record.counts = value
                            .split(',')
                            .filter_map(|s| s.trim().parse::<u32>().ok())
                            .collect();
                    }
                    "total" => {
                        self.edit_total = value.clone();
                        if let Ok(v) = value.parse() {
                            record.total = v
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn save_data(&self, game_path: &str) -> Result<(), String> {
        let path = PathBuf::from(game_path)
            .join("CharacterInGame")
            .join("ChData.db");
        if let Some(catalog) = &self.catalog {
            ChData::save_file(catalog, &path).map_err(|e| format!("Failed to save ch data: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
