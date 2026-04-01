use dispel_core::{Extractor, WeaponItem};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct WeaponEditorState {
    pub game_path: String,
    pub catalog: Option<Vec<WeaponItem>>,
    pub filtered_weapons: Vec<(usize, WeaponItem)>, // (original_index, record)
    pub selected_idx: Option<usize>,                // Index into filtered_weapons

    // String buffers for text inputs (iced lifetime requirement)
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
    pub edit_attack: String,
    pub edit_defense: String,
    pub edit_magical_strength: String,
    pub edit_durability: String,
    pub edit_req_strength: String,
    pub edit_req_agility: String,
    pub edit_req_wisdom: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl WeaponEditorState {
    pub fn refresh_weapons(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered_weapons = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect::<Vec<_>>();
        }
    }

    pub fn select_weapon(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        if let Some((_, record)) = self.filtered_weapons.get(idx) {
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
            self.edit_attack = record.attack.to_string();
            self.edit_defense = record.defense.to_string();
            self.edit_magical_strength = record.magical_strength.to_string();
            self.edit_durability = record.durability.to_string();
            self.edit_req_strength = record.req_strength.to_string();
            self.edit_req_agility = record.req_agility.to_string();
            self.edit_req_wisdom = record.req_wisdom.to_string();
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_weapons.get_mut(idx).map(|(_, r)| r) {
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
                "strength" => {
                    if let Ok(v) = value.parse() {
                        record.strength = v
                    }
                }
                "agility" => {
                    if let Ok(v) = value.parse() {
                        record.agility = v
                    }
                }
                "wisdom" => {
                    if let Ok(v) = value.parse() {
                        record.wisdom = v
                    }
                }
                "constitution" => {
                    if let Ok(v) = value.parse() {
                        record.constitution = v
                    }
                }
                "to_dodge" => {
                    if let Ok(v) = value.parse() {
                        record.to_dodge = v
                    }
                }
                "to_hit" => {
                    if let Ok(v) = value.parse() {
                        record.to_hit = v
                    }
                }
                "attack" => {
                    if let Ok(v) = value.parse() {
                        record.attack = v
                    }
                }
                "defense" => {
                    if let Ok(v) = value.parse() {
                        record.defense = v
                    }
                }
                "magical_strength" => {
                    if let Ok(v) = value.parse() {
                        record.magical_strength = v
                    }
                }
                "durability" => {
                    if let Ok(v) = value.parse() {
                        record.durability = v
                    }
                }
                "req_strength" => {
                    if let Ok(v) = value.parse() {
                        record.req_strength = v
                    }
                }
                "req_agility" => {
                    if let Ok(v) = value.parse() {
                        record.req_agility = v
                    }
                }
                "req_wisdom" => {
                    if let Ok(v) = value.parse() {
                        record.req_wisdom = v
                    }
                }
                _ => {}
            }
            self.refresh_weapons();
        }
    }

    pub fn save_weapons(&self) -> Result<(), String> {
        let path = PathBuf::from(&self.game_path)
            .join("CharacterInGame")
            .join("weaponItem.db");
        if let Some(catalog) = &self.catalog {
            WeaponItem::save_file(catalog, &path)
                .map_err(|e| format!("Failed to save weapons: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }

    pub fn scan_and_read(path: &Path) -> Result<Vec<WeaponItem>, String> {
        WeaponItem::read_file(&path.join("CharacterInGame").join("weaponItem.db"))
            .map_err(|e: std::io::Error| format!("Failed to read weapons: {}", e))
    }
}
