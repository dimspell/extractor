use dispel_core::{Extractor, Monster};
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct MonsterEditorState {
    pub catalog: Option<Vec<Monster>>,
    pub filtered_monsters: Vec<(usize, Monster)>,
    pub selected_idx: Option<usize>,

    pub edit_name: String,
    pub edit_hp_max: String,
    pub edit_hp_min: String,
    pub edit_mp_max: String,
    pub edit_mp_min: String,
    pub edit_walk_speed: String,
    pub edit_to_hit_max: String,
    pub edit_to_hit_min: String,
    pub edit_to_dodge_max: String,
    pub edit_to_dodge_min: String,
    pub edit_offense_max: String,
    pub edit_offense_min: String,
    pub edit_defense_max: String,
    pub edit_defense_min: String,
    pub edit_magic_attack_max: String,
    pub edit_magic_attack_min: String,
    pub edit_is_undead: String,
    pub edit_has_blood: String,
    pub edit_ai_type: String,
    pub edit_exp_gain_max: String,
    pub edit_exp_gain_min: String,
    pub edit_gold_drop_max: String,
    pub edit_gold_drop_min: String,
    pub edit_detection_sight_size: String,
    pub edit_distance_range_size: String,
    pub edit_known_spell_slot1: String,
    pub edit_known_spell_slot2: String,
    pub edit_known_spell_slot3: String,
    pub edit_is_oversize: String,
    pub edit_magic_level: String,
    pub edit_special_attack: String,
    pub edit_special_attack_chance: String,
    pub edit_special_attack_duration: String,
    pub edit_boldness: String,
    pub edit_attack_speed: String,

    pub status_msg: String,
    pub is_loading: bool,
}

impl MonsterEditorState {
    pub fn refresh_monsters(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered_monsters = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect::<Vec<_>>();
        }
    }

    pub fn select_monster(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        if let Some((_, record)) = self.filtered_monsters.get(idx) {
            self.edit_name = record.name.clone();
            self.edit_hp_max = record.health_points_max.to_string();
            self.edit_hp_min = record.health_points_min.to_string();
            self.edit_mp_max = record.mana_points_max.to_string();
            self.edit_mp_min = record.mana_points_min.to_string();
            self.edit_walk_speed = record.walk_speed.to_string();
            self.edit_to_hit_max = record.to_hit_max.to_string();
            self.edit_to_hit_min = record.to_hit_min.to_string();
            self.edit_to_dodge_max = record.to_dodge_max.to_string();
            self.edit_to_dodge_min = record.to_dodge_min.to_string();
            self.edit_offense_max = record.offense_max.to_string();
            self.edit_offense_min = record.offense_min.to_string();
            self.edit_defense_max = record.defense_max.to_string();
            self.edit_defense_min = record.defense_min.to_string();
            self.edit_magic_attack_max = record.magic_attack_max.to_string();
            self.edit_magic_attack_min = record.magic_attack_min.to_string();
            self.edit_is_undead = format!("{:?}", record.is_undead);
            self.edit_has_blood = format!("{:?}", record.has_blood);
            self.edit_ai_type = format!("{:?}", record.ai_type);
            self.edit_exp_gain_max = record.exp_gain_max.to_string();
            self.edit_exp_gain_min = record.exp_gain_min.to_string();
            self.edit_gold_drop_max = record.gold_drop_max.to_string();
            self.edit_gold_drop_min = record.gold_drop_min.to_string();
            self.edit_detection_sight_size = record.detection_sight_size.to_string();
            self.edit_distance_range_size = record.distance_range_size.to_string();
            self.edit_known_spell_slot1 = record.known_spell_slot1.to_string();
            self.edit_known_spell_slot2 = record.known_spell_slot2.to_string();
            self.edit_known_spell_slot3 = record.known_spell_slot3.to_string();
            self.edit_is_oversize = record.is_oversize.to_string();
            self.edit_magic_level = record.magic_level.to_string();
            self.edit_special_attack = record.special_attack.to_string();
            self.edit_special_attack_chance = record.special_attack_chance.to_string();
            self.edit_special_attack_duration = record.special_attack_duration.to_string();
            self.edit_boldness = record.boldness.to_string();
            self.edit_attack_speed = record.attack_speed.to_string();
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_monsters.get_mut(idx).map(|(_, r)| r) {
            match field {
                "name" => {
                    self.edit_name = value.clone();
                    record.name = value;
                }
                "health_points_max" => {
                    self.edit_hp_max = value.clone();
                    if let Ok(v) = value.parse() {
                        record.health_points_max = v
                    }
                }
                "health_points_min" => {
                    self.edit_hp_min = value.clone();
                    if let Ok(v) = value.parse() {
                        record.health_points_min = v
                    }
                }
                "mana_points_max" => {
                    self.edit_mp_max = value.clone();
                    if let Ok(v) = value.parse() {
                        record.mana_points_max = v
                    }
                }
                "mana_points_min" => {
                    self.edit_mp_min = value.clone();
                    if let Ok(v) = value.parse() {
                        record.mana_points_min = v
                    }
                }
                "walk_speed" => {
                    self.edit_walk_speed = value.clone();
                    if let Ok(v) = value.parse() {
                        record.walk_speed = v
                    }
                }
                "to_hit_max" => {
                    self.edit_to_hit_max = value.clone();
                    if let Ok(v) = value.parse() {
                        record.to_hit_max = v
                    }
                }
                "to_hit_min" => {
                    self.edit_to_hit_min = value.clone();
                    if let Ok(v) = value.parse() {
                        record.to_hit_min = v
                    }
                }
                "to_dodge_max" => {
                    self.edit_to_dodge_max = value.clone();
                    if let Ok(v) = value.parse() {
                        record.to_dodge_max = v
                    }
                }
                "to_dodge_min" => {
                    self.edit_to_dodge_min = value.clone();
                    if let Ok(v) = value.parse() {
                        record.to_dodge_min = v
                    }
                }
                "offense_max" => {
                    self.edit_offense_max = value.clone();
                    if let Ok(v) = value.parse() {
                        record.offense_max = v
                    }
                }
                "offense_min" => {
                    self.edit_offense_min = value.clone();
                    if let Ok(v) = value.parse() {
                        record.offense_min = v
                    }
                }
                "defense_max" => {
                    self.edit_defense_max = value.clone();
                    if let Ok(v) = value.parse() {
                        record.defense_max = v
                    }
                }
                "defense_min" => {
                    self.edit_defense_min = value.clone();
                    if let Ok(v) = value.parse() {
                        record.defense_min = v
                    }
                }
                "magic_attack_max" => {
                    self.edit_magic_attack_max = value.clone();
                    if let Ok(v) = value.parse() {
                        record.magic_attack_max = v
                    }
                }
                "magic_attack_min" => {
                    self.edit_magic_attack_min = value.clone();
                    if let Ok(v) = value.parse() {
                        record.magic_attack_min = v
                    }
                }
                "is_undead" => {
                    self.edit_is_undead = value.clone();
                    record.is_undead = if value.contains("Present") {
                        dispel_core::PropertyFlag::Present
                    } else {
                        dispel_core::PropertyFlag::Absent
                    };
                }
                "has_blood" => {
                    self.edit_has_blood = value.clone();
                    record.has_blood = if value.contains("Present") {
                        dispel_core::PropertyFlag::Present
                    } else {
                        dispel_core::PropertyFlag::Absent
                    };
                }
                "ai_type" => {
                    self.edit_ai_type = value.clone();
                    if value.contains("Aggressive") {
                        record.ai_type = dispel_core::MonsterAiType::Aggressive;
                    } else if value.contains("Defensive") {
                        record.ai_type = dispel_core::MonsterAiType::Defensive;
                    } else if value.contains("Ranged") {
                        record.ai_type = dispel_core::MonsterAiType::Ranged;
                    } else if value.contains("Boss") {
                        record.ai_type = dispel_core::MonsterAiType::Boss;
                    } else if value.contains("Special") {
                        record.ai_type = dispel_core::MonsterAiType::Special;
                    } else if value.contains("Custom") {
                        record.ai_type = dispel_core::MonsterAiType::Custom;
                    } else {
                        record.ai_type = dispel_core::MonsterAiType::Passive;
                    }
                }
                "exp_gain_max" => {
                    self.edit_exp_gain_max = value.clone();
                    if let Ok(v) = value.parse() {
                        record.exp_gain_max = v
                    }
                }
                "exp_gain_min" => {
                    self.edit_exp_gain_min = value.clone();
                    if let Ok(v) = value.parse() {
                        record.exp_gain_min = v
                    }
                }
                "gold_drop_max" => {
                    self.edit_gold_drop_max = value.clone();
                    if let Ok(v) = value.parse() {
                        record.gold_drop_max = v
                    }
                }
                "gold_drop_min" => {
                    self.edit_gold_drop_min = value.clone();
                    if let Ok(v) = value.parse() {
                        record.gold_drop_min = v
                    }
                }
                "detection_sight_size" => {
                    self.edit_detection_sight_size = value.clone();
                    if let Ok(v) = value.parse() {
                        record.detection_sight_size = v
                    }
                }
                "distance_range_size" => {
                    self.edit_distance_range_size = value.clone();
                    if let Ok(v) = value.parse() {
                        record.distance_range_size = v
                    }
                }
                "known_spell_slot1" => {
                    self.edit_known_spell_slot1 = value.clone();
                    if let Ok(v) = value.parse() {
                        record.known_spell_slot1 = v
                    }
                }
                "known_spell_slot2" => {
                    self.edit_known_spell_slot2 = value.clone();
                    if let Ok(v) = value.parse() {
                        record.known_spell_slot2 = v
                    }
                }
                "known_spell_slot3" => {
                    self.edit_known_spell_slot3 = value.clone();
                    if let Ok(v) = value.parse() {
                        record.known_spell_slot3 = v
                    }
                }
                "is_oversize" => {
                    self.edit_is_oversize = value.clone();
                    if let Ok(v) = value.parse() {
                        record.is_oversize = v
                    }
                }
                "magic_level" => {
                    self.edit_magic_level = value.clone();
                    if let Ok(v) = value.parse() {
                        record.magic_level = v
                    }
                }
                "special_attack" => {
                    self.edit_special_attack = value.clone();
                    if let Ok(v) = value.parse() {
                        record.special_attack = v
                    }
                }
                "special_attack_chance" => {
                    self.edit_special_attack_chance = value.clone();
                    if let Ok(v) = value.parse() {
                        record.special_attack_chance = v
                    }
                }
                "special_attack_duration" => {
                    self.edit_special_attack_duration = value.clone();
                    if let Ok(v) = value.parse() {
                        record.special_attack_duration = v
                    }
                }
                "boldness" => {
                    self.edit_boldness = value.clone();
                    if let Ok(v) = value.parse() {
                        record.boldness = v
                    }
                }
                "attack_speed" => {
                    self.edit_attack_speed = value.clone();
                    if let Ok(v) = value.parse() {
                        record.attack_speed = v
                    }
                }
                _ => {}
            }
            self.refresh_monsters();
        }
    }

    pub fn save_monsters(&self, game_path: &str) -> Result<(), String> {
        let path = PathBuf::from(game_path)
            .join("MonsterInGame")
            .join("Monster.db");
        if let Some(catalog) = &self.catalog {
            Monster::save_file(catalog, &path)
                .map_err(|e| format!("Failed to save monsters: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
