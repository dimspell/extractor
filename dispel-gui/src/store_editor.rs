use dispel_core::{Extractor, Store};
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct StoreEditorState {
    pub game_path: String,
    pub catalog: Option<Vec<Store>>,
    pub filtered_stores: Vec<(usize, Store)>,
    pub selected_idx: Option<usize>,

    pub edit_store_name: String,
    pub edit_inn_night_cost: String,
    pub edit_some_unknown_number: String,
    pub edit_invitation: String,
    pub edit_haggle_success: String,
    pub edit_haggle_fail: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl StoreEditorState {
    pub fn refresh_stores(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered_stores = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect::<Vec<_>>();
        }
    }

    pub fn select_store(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        if let Some((_, record)) = self.filtered_stores.get(idx) {
            self.edit_store_name = record.store_name.clone();
            self.edit_inn_night_cost = record.inn_night_cost.to_string();
            self.edit_some_unknown_number = record.some_unknown_number.to_string();
            self.edit_invitation = record.invitation.clone();
            self.edit_haggle_success = record.haggle_success.clone();
            self.edit_haggle_fail = record.haggle_fail.clone();
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_stores.get_mut(idx).map(|(_, r)| r) {
            match field {
                "store_name" => record.store_name = value.clone(),
                "inn_night_cost" => {
                    if let Ok(v) = value.parse() {
                        record.inn_night_cost = v
                    }
                }
                "some_unknown_number" => {
                    if let Ok(v) = value.parse() {
                        record.some_unknown_number = v
                    }
                }
                "invitation" => record.invitation = value.clone(),
                "haggle_success" => record.haggle_success = value.clone(),
                "haggle_fail" => record.haggle_fail = value.clone(),
                _ => {}
            }
            self.refresh_stores();
        }
    }

    pub fn save_stores(&self) -> Result<(), String> {
        let path = PathBuf::from(&self.game_path)
            .join("CharacterInGame")
            .join("STORE.DB");
        if let Some(catalog) = &self.catalog {
            Store::save_file(catalog, &path).map_err(|e| format!("Failed to save stores: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
