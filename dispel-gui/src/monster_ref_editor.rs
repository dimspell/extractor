use dispel_core::{Extractor, MonsterRef};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct MonsterRefEntryState {
    pub index: i32,
    pub edit_file_id: String,
    pub edit_mon_id: String,
    pub edit_pos_x: String,
    pub edit_pos_y: String,
    pub edit_loot1_item_id: String,
    pub edit_loot1_item_type: String,
    pub edit_loot2_item_id: String,
    pub edit_loot2_item_type: String,
    pub edit_loot3_item_id: String,
    pub edit_loot3_item_type: String,
}

impl Default for MonsterRefEntryState {
    fn default() -> Self {
        Self {
            index: 0,
            edit_file_id: String::new(),
            edit_mon_id: String::new(),
            edit_pos_x: String::new(),
            edit_pos_y: String::new(),
            edit_loot1_item_id: String::new(),
            edit_loot1_item_type: String::new(),
            edit_loot2_item_id: String::new(),
            edit_loot2_item_type: String::new(),
            edit_loot3_item_id: String::new(),
            edit_loot3_item_type: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MonsterRefEditorState {
    pub map_files: Vec<PathBuf>,
    pub current_map_file: String,
    pub catalog: Option<Vec<MonsterRef>>,
    pub filtered_entries: Vec<(usize, MonsterRef)>,
    pub selected_idx: Option<usize>,
    pub entry_editor: MonsterRefEntryState,
    pub status_msg: String,
    pub is_loading: bool,
}

impl MonsterRefEditorState {
    pub fn refresh_entries(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered_entries = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect::<Vec<_>>();
        }
    }

    pub fn select_entry(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        if let Some((_, record)) = self.filtered_entries.get(idx) {
            self.entry_editor.index = record.index;
            self.entry_editor.edit_file_id = record.file_id.to_string();
            self.entry_editor.edit_mon_id = record.mon_id.to_string();
            self.entry_editor.edit_pos_x = record.pos_x.to_string();
            self.entry_editor.edit_pos_y = record.pos_y.to_string();
            self.entry_editor.edit_loot1_item_id = record.loot1_item_id.to_string();
            self.entry_editor.edit_loot1_item_type = format!("{:?}", record.loot1_item_type);
            self.entry_editor.edit_loot2_item_id = record.loot2_item_id.to_string();
            self.entry_editor.edit_loot2_item_type = format!("{:?}", record.loot2_item_type);
            self.entry_editor.edit_loot3_item_id = record.loot3_item_id.to_string();
            self.entry_editor.edit_loot3_item_type = format!("{:?}", record.loot3_item_type);
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_entries.get_mut(idx).map(|(_, r)| r) {
            match field {
                "file_id" => {
                    self.entry_editor.edit_file_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.file_id = v
                    }
                }
                "mon_id" => {
                    self.entry_editor.edit_mon_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.mon_id = v
                    }
                }
                "pos_x" => {
                    self.entry_editor.edit_pos_x = value.clone();
                    if let Ok(v) = value.parse() {
                        record.pos_x = v
                    }
                }
                "pos_y" => {
                    self.entry_editor.edit_pos_y = value.clone();
                    if let Ok(v) = value.parse() {
                        record.pos_y = v
                    }
                }
                "loot1_item_id" => {
                    self.entry_editor.edit_loot1_item_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.loot1_item_id = v
                    }
                }
                "loot1_item_type" => {
                    self.entry_editor.edit_loot1_item_type = value.clone();
                    if let Some(t) = dispel_core::ItemTypeId::from_name(&value) {
                        record.loot1_item_type = t;
                    }
                }
                "loot2_item_id" => {
                    self.entry_editor.edit_loot2_item_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.loot2_item_id = v
                    }
                }
                "loot2_item_type" => {
                    self.entry_editor.edit_loot2_item_type = value.clone();
                    if let Some(t) = dispel_core::ItemTypeId::from_name(&value) {
                        record.loot2_item_type = t;
                    }
                }
                "loot3_item_id" => {
                    self.entry_editor.edit_loot3_item_id = value.clone();
                    if let Ok(v) = value.parse() {
                        record.loot3_item_id = v
                    }
                }
                "loot3_item_type" => {
                    self.entry_editor.edit_loot3_item_type = value.clone();
                    if let Some(t) = dispel_core::ItemTypeId::from_name(&value) {
                        record.loot3_item_type = t;
                    }
                }
                _ => {}
            }
            self.refresh_entries();
        }
    }

    pub fn save_entries(&self) -> Result<(), String> {
        if self.current_map_file.is_empty() {
            return Err("No map file selected".to_string());
        }
        let path = PathBuf::from(&self.current_map_file);
        if let Some(catalog) = &self.catalog {
            MonsterRef::save_file(catalog, &path)
                .map_err(|e| format!("Failed to save monster ref: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
