use dispel_core::{EditItem, EditItemEffect, EditItemModification, Extractor};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct EditItemEditorState {
    pub game_path: String,
    pub catalog: Option<Vec<EditItem>>,
    pub filtered_items: Vec<(usize, EditItem)>,
    pub selected_idx: Option<usize>,

    pub edit_name: String,
    pub edit_description: String,
    pub edit_base_price: String,
    pub edit_health_points: String,
    pub edit_mana_points: String,
    pub edit_strength: String,
    pub edit_agility: String,
    pub edit_wisdom: String,
    pub edit_constitution: String,
    pub edit_to_dodge: String,
    pub edit_to_hit: String,
    pub edit_offense: String,
    pub edit_defense: String,
    pub edit_magical_power: String,
    pub edit_item_destroying_power: String,
    pub edit_modifies_item: String,
    pub edit_additional_effect: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl EditItemEditorState {
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
            self.edit_strength = record.strength.to_string();
            self.edit_agility = record.agility.to_string();
            self.edit_wisdom = record.wisdom.to_string();
            self.edit_constitution = record.constitution.to_string();
            self.edit_to_dodge = record.to_dodge.to_string();
            self.edit_to_hit = record.to_hit.to_string();
            self.edit_offense = record.offense.to_string();
            self.edit_defense = record.defense.to_string();
            self.edit_magical_power = record.magical_power.to_string();
            self.edit_item_destroying_power = record.item_destroying_power.to_string();
            self.edit_modifies_item = format!("{:?}", record.modifies_item);
            self.edit_additional_effect = format!("{:?}", record.additional_effect);
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_items.get_mut(idx).map(|(_, r)| r) {
            match field {
                "name" => {
                    self.edit_name = value.clone();
                    record.name = value;
                }
                "description" => {
                    self.edit_description = value.clone();
                    record.description = value;
                }
                "base_price" => {
                    self.edit_base_price = value.clone();
                    if let Ok(v) = value.parse() {
                        record.base_price = v
                    }
                }
                "health_points" => {
                    self.edit_health_points = value.clone();
                    if let Ok(v) = value.parse() {
                        record.health_points = v
                    }
                }
                "mana_points" => {
                    self.edit_mana_points = value.clone();
                    if let Ok(v) = value.parse() {
                        record.mana_points = v
                    }
                }
                "strength" => {
                    self.edit_strength = value.clone();
                    if let Ok(v) = value.parse() {
                        record.strength = v
                    }
                }
                "agility" => {
                    self.edit_agility = value.clone();
                    if let Ok(v) = value.parse() {
                        record.agility = v
                    }
                }
                "wisdom" => {
                    self.edit_wisdom = value.clone();
                    if let Ok(v) = value.parse() {
                        record.wisdom = v
                    }
                }
                "constitution" => {
                    self.edit_constitution = value.clone();
                    if let Ok(v) = value.parse() {
                        record.constitution = v
                    }
                }
                "to_dodge" => {
                    self.edit_to_dodge = value.clone();
                    if let Ok(v) = value.parse() {
                        record.to_dodge = v
                    }
                }
                "to_hit" => {
                    self.edit_to_hit = value.clone();
                    if let Ok(v) = value.parse() {
                        record.to_hit = v
                    }
                }
                "offense" => {
                    self.edit_offense = value.clone();
                    if let Ok(v) = value.parse() {
                        record.offense = v
                    }
                }
                "defense" => {
                    self.edit_defense = value.clone();
                    if let Ok(v) = value.parse() {
                        record.defense = v
                    }
                }
                "magical_power" => {
                    self.edit_magical_power = value.clone();
                    if let Ok(v) = value.parse() {
                        record.magical_power = v
                    }
                }
                "item_destroying_power" => {
                    self.edit_item_destroying_power = value.clone();
                    if let Ok(v) = value.parse() {
                        record.item_destroying_power = v
                    }
                }
                "modifies_item" => {
                    self.edit_modifies_item = value.clone();
                    record.modifies_item = if value.contains("CanModify") {
                        EditItemModification::CanModify
                    } else {
                        EditItemModification::DoesNotModify
                    };
                }
                "additional_effect" => {
                    self.edit_additional_effect = value.clone();
                    record.additional_effect = if value.contains("Fire") {
                        EditItemEffect::Fire
                    } else if value.contains("ManaDrain") {
                        EditItemEffect::ManaDrain
                    } else {
                        EditItemEffect::None
                    };
                }
                _ => {}
            }
            self.refresh_items();
        }
    }

    pub fn save_items(&self) -> Result<(), String> {
        let path = PathBuf::from(&self.game_path)
            .join("CharacterInGame")
            .join("EditItem.db");
        if let Some(catalog) = &self.catalog {
            EditItem::save_file(catalog, &path)
                .map_err(|e| format!("Failed to save edit items: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }

    pub fn scan_and_read(path: &Path) -> Result<Vec<EditItem>, String> {
        EditItem::read_file(&path.join("CharacterInGame").join("EditItem.db"))
            .map_err(|e: std::io::Error| format!("Failed to read edit items: {}", e))
    }
}
