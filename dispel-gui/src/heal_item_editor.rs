use dispel_core::{Extractor, HealItem};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct HealItemEditorState {
    pub game_path: String,
    pub catalog: Option<Vec<HealItem>>,
    pub filtered_items: Vec<(usize, HealItem)>, // (original_index, record)
    pub selected_idx: Option<usize>,            // Index into filtered_items

    // String buffers for text inputs (iced lifetime requirement)
    pub edit_name: String,
    pub edit_description: String,
    pub edit_base_price: String,
    pub edit_health_points: String,
    pub edit_mana_points: String,
    pub edit_restore_full_health: String,
    pub edit_restore_full_mana: String,
    pub edit_poison_heal: String,
    pub edit_petrif_heal: String,
    pub edit_polimorph_heal: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl HealItemEditorState {
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
            self.edit_name = record.name.clone();
            self.edit_description = record.description.clone();
            self.edit_base_price = record.base_price.to_string();
            self.edit_health_points = record.health_points.to_string();
            self.edit_mana_points = record.mana_points.to_string();
            self.edit_restore_full_health = format!("{:?}", record.restore_full_health);
            self.edit_restore_full_mana = format!("{:?}", record.restore_full_mana);
            self.edit_poison_heal = format!("{:?}", record.poison_heal);
            self.edit_petrif_heal = format!("{:?}", record.petrif_heal);
            self.edit_polimorph_heal = format!("{:?}", record.polimorph_heal);
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_items.get_mut(idx).map(|(_, r)| r) {
            match field {
                "name" => record.name = value.clone(),
                "description" => record.description = value.clone(),
                "base_price" => {
                    if let Ok(v) = value.parse() {
                        record.base_price = v
                    }
                }
                "health_points" => {
                    if let Ok(v) = value.parse() {
                        record.health_points = v
                    }
                }
                "mana_points" => {
                    if let Ok(v) = value.parse() {
                        record.mana_points = v
                    }
                }
                "restore_full_health" => {
                    // Parse enum value
                    if value == "FullRestoration" {
                        record.restore_full_health = dispel_core::HealItemFlag::FullRestoration;
                    } else {
                        record.restore_full_health = dispel_core::HealItemFlag::None;
                    }
                }
                "restore_full_mana" => {
                    if value == "FullRestoration" {
                        record.restore_full_mana = dispel_core::HealItemFlag::FullRestoration;
                    } else {
                        record.restore_full_mana = dispel_core::HealItemFlag::None;
                    }
                }
                "poison_heal" => {
                    if value == "FullRestoration" {
                        record.poison_heal = dispel_core::HealItemFlag::FullRestoration;
                    } else {
                        record.poison_heal = dispel_core::HealItemFlag::None;
                    }
                }
                "petrif_heal" => {
                    if value == "FullRestoration" {
                        record.petrif_heal = dispel_core::HealItemFlag::FullRestoration;
                    } else {
                        record.petrif_heal = dispel_core::HealItemFlag::None;
                    }
                }
                "polimorph_heal" => {
                    if value == "FullRestoration" {
                        record.polimorph_heal = dispel_core::HealItemFlag::FullRestoration;
                    } else {
                        record.polimorph_heal = dispel_core::HealItemFlag::None;
                    }
                }
                _ => {}
            }
            self.refresh_items();
        }
    }

    pub fn save_items(&self) -> Result<(), String> {
        let path = PathBuf::from(&self.game_path)
            .join("CharacterInGame")
            .join("HealItem.db");
        if let Some(catalog) = &self.catalog {
            HealItem::save_file(catalog, &path)
                .map_err(|e| format!("Failed to save heal items: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }

    pub fn scan_and_read(path: &Path) -> Result<Vec<HealItem>, String> {
        HealItem::read_file(&path.join("CharacterInGame").join("HealItem.db"))
            .map_err(|e: std::io::Error| format!("Failed to read heal items: {}", e))
    }
}
