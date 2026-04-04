use super::editable::{EditableRecord, FieldDescriptor, FieldKind};
use super::enums::{MonsterAiType, PropertyFlag};
use super::monster_db::Monster;

impl EditableRecord for Monster {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "id",
                label: "ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "name",
                label: "Name:",
                kind: FieldKind::Text,
            },
            FieldDescriptor {
                name: "health_points_max",
                label: "HP Max:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "health_points_min",
                label: "HP Min:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "mana_points_max",
                label: "MP Max:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "mana_points_min",
                label: "MP Min:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "walk_speed",
                label: "Walk Speed:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "to_hit_max",
                label: "To Hit Max:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "to_hit_min",
                label: "To Hit Min:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "to_dodge_max",
                label: "Dodge Max:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "to_dodge_min",
                label: "Dodge Min:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "offense_max",
                label: "Offense Max:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "offense_min",
                label: "Offense Min:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "defense_max",
                label: "Defense Max:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "defense_min",
                label: "Defense Min:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "magic_attack_max",
                label: "Magic Atk Max:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "magic_attack_min",
                label: "Magic Atk Min:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "is_undead",
                label: "Undead:",
                kind: FieldKind::Boolean,
            },
            FieldDescriptor {
                name: "has_blood",
                label: "Has Blood:",
                kind: FieldKind::Boolean,
            },
            FieldDescriptor {
                name: "ai_type",
                label: "AI Type:",
                kind: FieldKind::Enum {
                    variants: &[
                        "Passive",
                        "Aggressive",
                        "Defensive",
                        "Ranged",
                        "Boss",
                        "Special",
                        "Custom",
                    ],
                },
            },
            FieldDescriptor {
                name: "exp_gain_max",
                label: "EXP Max:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "exp_gain_min",
                label: "EXP Min:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "gold_drop_max",
                label: "Gold Max:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "gold_drop_min",
                label: "Gold Min:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "detection_sight_size",
                label: "Sight Range:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "distance_range_size",
                label: "Attack Range:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "known_spell_slot1",
                label: "Spell Slot 1:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "known_spell_slot2",
                label: "Spell Slot 2:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "known_spell_slot3",
                label: "Spell Slot 3:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "is_oversize",
                label: "Oversize:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "magic_level",
                label: "Magic Level:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "special_attack",
                label: "Special Attack:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "special_attack_chance",
                label: "Special Atk Chance:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "special_attack_duration",
                label: "Special Atk Duration:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "boldness",
                label: "Boldness:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "attack_speed",
                label: "Attack Speed:",
                kind: FieldKind::Integer,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "id" => self.id.to_string(),
            "name" => self.name.clone(),
            "health_points_max" => self.health_points_max.to_string(),
            "health_points_min" => self.health_points_min.to_string(),
            "mana_points_max" => self.mana_points_max.to_string(),
            "mana_points_min" => self.mana_points_min.to_string(),
            "walk_speed" => self.walk_speed.to_string(),
            "to_hit_max" => self.to_hit_max.to_string(),
            "to_hit_min" => self.to_hit_min.to_string(),
            "to_dodge_max" => self.to_dodge_max.to_string(),
            "to_dodge_min" => self.to_dodge_min.to_string(),
            "offense_max" => self.offense_max.to_string(),
            "offense_min" => self.offense_min.to_string(),
            "defense_max" => self.defense_max.to_string(),
            "defense_min" => self.defense_min.to_string(),
            "magic_attack_max" => self.magic_attack_max.to_string(),
            "magic_attack_min" => self.magic_attack_min.to_string(),
            "is_undead" => (self.is_undead == PropertyFlag::Present).to_string(),
            "has_blood" => (self.has_blood == PropertyFlag::Present).to_string(),
            "ai_type" => self.ai_type.value().to_string(),
            "exp_gain_max" => self.exp_gain_max.to_string(),
            "exp_gain_min" => self.exp_gain_min.to_string(),
            "gold_drop_max" => self.gold_drop_max.to_string(),
            "gold_drop_min" => self.gold_drop_min.to_string(),
            "detection_sight_size" => self.detection_sight_size.to_string(),
            "distance_range_size" => self.distance_range_size.to_string(),
            "known_spell_slot1" => self.known_spell_slot1.to_string(),
            "known_spell_slot2" => self.known_spell_slot2.to_string(),
            "known_spell_slot3" => self.known_spell_slot3.to_string(),
            "is_oversize" => self.is_oversize.to_string(),
            "magic_level" => self.magic_level.to_string(),
            "special_attack" => self.special_attack.to_string(),
            "special_attack_chance" => self.special_attack_chance.to_string(),
            "special_attack_duration" => self.special_attack_duration.to_string(),
            "boldness" => self.boldness.to_string(),
            "attack_speed" => self.attack_speed.to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "id" => {
                if let Ok(v) = value.parse() {
                    self.id = v;
                    true
                } else {
                    false
                }
            }
            "name" => {
                self.name = value;
                true
            }
            "health_points_max" => {
                if let Ok(v) = value.parse() {
                    self.health_points_max = v;
                    true
                } else {
                    false
                }
            }
            "health_points_min" => {
                if let Ok(v) = value.parse() {
                    self.health_points_min = v;
                    true
                } else {
                    false
                }
            }
            "mana_points_max" => {
                if let Ok(v) = value.parse() {
                    self.mana_points_max = v;
                    true
                } else {
                    false
                }
            }
            "mana_points_min" => {
                if let Ok(v) = value.parse() {
                    self.mana_points_min = v;
                    true
                } else {
                    false
                }
            }
            "walk_speed" => {
                if let Ok(v) = value.parse() {
                    self.walk_speed = v;
                    true
                } else {
                    false
                }
            }
            "to_hit_max" => {
                if let Ok(v) = value.parse() {
                    self.to_hit_max = v;
                    true
                } else {
                    false
                }
            }
            "to_hit_min" => {
                if let Ok(v) = value.parse() {
                    self.to_hit_min = v;
                    true
                } else {
                    false
                }
            }
            "to_dodge_max" => {
                if let Ok(v) = value.parse() {
                    self.to_dodge_max = v;
                    true
                } else {
                    false
                }
            }
            "to_dodge_min" => {
                if let Ok(v) = value.parse() {
                    self.to_dodge_min = v;
                    true
                } else {
                    false
                }
            }
            "offense_max" => {
                if let Ok(v) = value.parse() {
                    self.offense_max = v;
                    true
                } else {
                    false
                }
            }
            "offense_min" => {
                if let Ok(v) = value.parse() {
                    self.offense_min = v;
                    true
                } else {
                    false
                }
            }
            "defense_max" => {
                if let Ok(v) = value.parse() {
                    self.defense_max = v;
                    true
                } else {
                    false
                }
            }
            "defense_min" => {
                if let Ok(v) = value.parse() {
                    self.defense_min = v;
                    true
                } else {
                    false
                }
            }
            "magic_attack_max" => {
                if let Ok(v) = value.parse() {
                    self.magic_attack_max = v;
                    true
                } else {
                    false
                }
            }
            "magic_attack_min" => {
                if let Ok(v) = value.parse() {
                    self.magic_attack_min = v;
                    true
                } else {
                    false
                }
            }
            "is_undead" => {
                let flag = if value == "true" {
                    PropertyFlag::Present
                } else {
                    PropertyFlag::Absent
                };
                self.is_undead = flag;
                true
            }
            "has_blood" => {
                let flag = if value == "true" {
                    PropertyFlag::Present
                } else {
                    PropertyFlag::Absent
                };
                self.has_blood = flag;
                true
            }
            "ai_type" => {
                if let Ok(v) = value.parse() {
                    if let Some(t) = MonsterAiType::from_i32(v) {
                        self.ai_type = t;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            "exp_gain_max" => {
                if let Ok(v) = value.parse() {
                    self.exp_gain_max = v;
                    true
                } else {
                    false
                }
            }
            "exp_gain_min" => {
                if let Ok(v) = value.parse() {
                    self.exp_gain_min = v;
                    true
                } else {
                    false
                }
            }
            "gold_drop_max" => {
                if let Ok(v) = value.parse() {
                    self.gold_drop_max = v;
                    true
                } else {
                    false
                }
            }
            "gold_drop_min" => {
                if let Ok(v) = value.parse() {
                    self.gold_drop_min = v;
                    true
                } else {
                    false
                }
            }
            "detection_sight_size" => {
                if let Ok(v) = value.parse() {
                    self.detection_sight_size = v;
                    true
                } else {
                    false
                }
            }
            "distance_range_size" => {
                if let Ok(v) = value.parse() {
                    self.distance_range_size = v;
                    true
                } else {
                    false
                }
            }
            "known_spell_slot1" => {
                if let Ok(v) = value.parse() {
                    self.known_spell_slot1 = v;
                    true
                } else {
                    false
                }
            }
            "known_spell_slot2" => {
                if let Ok(v) = value.parse() {
                    self.known_spell_slot2 = v;
                    true
                } else {
                    false
                }
            }
            "known_spell_slot3" => {
                if let Ok(v) = value.parse() {
                    self.known_spell_slot3 = v;
                    true
                } else {
                    false
                }
            }
            "is_oversize" => {
                if let Ok(v) = value.parse() {
                    self.is_oversize = v;
                    true
                } else {
                    false
                }
            }
            "magic_level" => {
                if let Ok(v) = value.parse() {
                    self.magic_level = v;
                    true
                } else {
                    false
                }
            }
            "special_attack" => {
                if let Ok(v) = value.parse() {
                    self.special_attack = v;
                    true
                } else {
                    false
                }
            }
            "special_attack_chance" => {
                if let Ok(v) = value.parse() {
                    self.special_attack_chance = v;
                    true
                } else {
                    false
                }
            }
            "special_attack_duration" => {
                if let Ok(v) = value.parse() {
                    self.special_attack_duration = v;
                    true
                } else {
                    false
                }
            }
            "boldness" => {
                if let Ok(v) = value.parse() {
                    self.boldness = v;
                    true
                } else {
                    false
                }
            }
            "attack_speed" => {
                if let Ok(v) = value.parse() {
                    self.attack_speed = v;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!("[{}] {}", self.id, self.name)
    }

    fn detail_title() -> &'static str {
        "Monster Details"
    }

    fn empty_selection_text() -> &'static str {
        "No monster selected"
    }

    fn save_button_label() -> &'static str {
        "Save Monster"
    }

    fn detail_width() -> f32 {
        340.0
    }
}
