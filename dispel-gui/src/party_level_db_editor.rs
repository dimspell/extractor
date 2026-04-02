use dispel_core::Extractor;
use dispel_core::{PartyLevelNpc, PartyRef};
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct PartyLevelDbEditorState {
    pub catalog: Option<Vec<PartyLevelNpc>>,
    pub party_refs: Option<Vec<PartyRef>>,
    pub selected_npc_idx: Option<usize>,
    pub selected_record_idx: Option<usize>,

    pub edit_level: String,
    pub edit_strength: String,
    pub edit_constitution: String,
    pub edit_wisdom: String,
    pub edit_health_points: String,
    pub edit_mana_points: String,
    pub edit_agility: String,
    pub edit_attack: String,
    pub edit_mana_recharge: String,
    pub edit_defense: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl PartyLevelDbEditorState {
    pub fn npc_display_name(&self, npc_index: usize) -> String {
        if let Some(refs) = &self.party_refs {
            if let Some(pr) = refs.iter().find(|p| p.id == npc_index as i32) {
                if let Some(name) = &pr.full_name {
                    if !name.is_empty() {
                        return format!("[{}] {}", npc_index, name);
                    }
                }
            }
        }
        format!("[{}] NPC {}", npc_index, npc_index)
    }

    pub fn select_npc(&mut self, idx: usize) {
        self.selected_npc_idx = Some(idx);
        self.selected_record_idx = None;
        self.clear_edit_fields();
    }

    pub fn select_record(&mut self, idx: usize) {
        self.selected_record_idx = Some(idx);
        if let Some(npc_idx) = self.selected_npc_idx {
            if let Some(catalog) = &self.catalog {
                if let Some(npc) = catalog.get(npc_idx) {
                    if let Some(record) = npc.records.get(idx) {
                        self.edit_level = record.level.to_string();
                        self.edit_strength = record.strength.to_string();
                        self.edit_constitution = record.constitution.to_string();
                        self.edit_wisdom = record.wisdom.to_string();
                        self.edit_health_points = record.health_points.to_string();
                        self.edit_mana_points = record.mana_points.to_string();
                        self.edit_agility = record.agility.to_string();
                        self.edit_attack = record.attack.to_string();
                        self.edit_mana_recharge = record.mana_recharge.to_string();
                        self.edit_defense = record.defense.to_string();
                    }
                }
            }
        }
    }

    fn clear_edit_fields(&mut self) {
        self.edit_level.clear();
        self.edit_strength.clear();
        self.edit_constitution.clear();
        self.edit_wisdom.clear();
        self.edit_health_points.clear();
        self.edit_mana_points.clear();
        self.edit_agility.clear();
        self.edit_attack.clear();
        self.edit_mana_recharge.clear();
        self.edit_defense.clear();
    }

    pub fn update_field(&mut self, field: &str, value: String) {
        if let Some(npc_idx) = self.selected_npc_idx {
            if let Some(record_idx) = self.selected_record_idx {
                if let Some(catalog) = &mut self.catalog {
                    if let Some(npc) = catalog.get_mut(npc_idx) {
                        if let Some(record) = npc.records.get_mut(record_idx) {
                            match field {
                                "level" => {
                                    self.edit_level = value.clone();
                                    if let Ok(v) = value.parse() {
                                        record.level = v
                                    }
                                }
                                "strength" => {
                                    self.edit_strength = value.clone();
                                    if let Ok(v) = value.parse() {
                                        record.strength = v
                                    }
                                }
                                "constitution" => {
                                    self.edit_constitution = value.clone();
                                    if let Ok(v) = value.parse() {
                                        record.constitution = v
                                    }
                                }
                                "wisdom" => {
                                    self.edit_wisdom = value.clone();
                                    if let Ok(v) = value.parse() {
                                        record.wisdom = v
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
                                "agility" => {
                                    self.edit_agility = value.clone();
                                    if let Ok(v) = value.parse() {
                                        record.agility = v
                                    }
                                }
                                "attack" => {
                                    self.edit_attack = value.clone();
                                    if let Ok(v) = value.parse() {
                                        record.attack = v
                                    }
                                }
                                "mana_recharge" => {
                                    self.edit_mana_recharge = value.clone();
                                    if let Ok(v) = value.parse() {
                                        record.mana_recharge = v
                                    }
                                }
                                "defense" => {
                                    self.edit_defense = value.clone();
                                    if let Ok(v) = value.parse() {
                                        record.defense = v
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn save_levels(&self, game_path: &str) -> Result<(), String> {
        let path = PathBuf::from(game_path)
            .join("NpcInGame")
            .join("PrtLevel.db");
        if let Some(catalog) = &self.catalog {
            PartyLevelNpc::save_file(catalog, &path)
                .map_err(|e| format!("Failed to save party levels: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
