use dispel_core::DrawItem;
use dispel_core::Extractor;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct DrawItemEditorState {
    pub catalog: Option<Vec<DrawItem>>,
    pub filtered_items: Vec<(usize, DrawItem)>,
    pub selected_idx: Option<usize>,

    pub edit_map_id: String,
    pub edit_x_coord: String,
    pub edit_y_coord: String,
    pub edit_item_id: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl DrawItemEditorState {
    pub fn refresh_items(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered_items = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect::<Vec<_>>();
        }
    }

    pub fn select_item(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        if let Some((_, record)) = self.filtered_items.get(idx) {
            self.edit_map_id = record.map_id.to_string();
            self.edit_x_coord = record.x_coord.to_string();
            self.edit_y_coord = record.y_coord.to_string();
            self.edit_item_id = record.item_id.to_string();
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_items.get_mut(idx).map(|(_, r)| r) {
            match field {
                "map_id" => {
                    self.edit_map_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.map_id = v
                    }
                }
                "x_coord" => {
                    self.edit_x_coord = value.clone();
                    if let Ok(v) = value.parse() {
                        record.x_coord = v
                    }
                }
                "y_coord" => {
                    self.edit_y_coord = value.clone();
                    if let Ok(v) = value.parse() {
                        record.y_coord = v
                    }
                }
                "item_id" => {
                    self.edit_item_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.item_id = v
                    }
                }
                _ => {}
            }
            self.refresh_items();
        }
    }

    pub fn save_items(&self, game_path: &str) -> Result<(), String> {
        let path = PathBuf::from(game_path).join("Ref").join("DRAWITEM.ref");
        if let Some(catalog) = &self.catalog {
            DrawItem::save_file(catalog, &path)
                .map_err(|e| format!("Failed to save draw items: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
