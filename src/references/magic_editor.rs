use super::editable::{EditableRecord, FieldDescriptor, FieldKind};
use super::enums::{MagicSchool, MagicSpellFlag, SpellTargetType};
use super::magic_db::MagicSpell;

impl EditableRecord for MagicSpell {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "enabled",
                label: "Enabled:",
                kind: FieldKind::Enum {
                    variants: &["Disabled", "Enabled"],
                },
            },
            FieldDescriptor {
                name: "mana_cost",
                label: "Mana Cost:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "success_rate",
                label: "Success Rate:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "base_damage",
                label: "Base Damage:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "range",
                label: "Range:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "level_required",
                label: "Level Required:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "effect_value",
                label: "Effect Value:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "effect_type",
                label: "Effect Type:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "effect_modifier",
                label: "Effect Modifier:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "magic_school",
                label: "Magic School:",
                kind: FieldKind::Enum {
                    variants: &[
                        "Unknown", "School1", "School2", "School3", "School4", "School5", "School6",
                    ],
                },
            },
            FieldDescriptor {
                name: "animation_id",
                label: "Animation ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "visual_id",
                label: "Visual ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "icon_id",
                label: "Icon ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "target_type",
                label: "Target Type:",
                kind: FieldKind::Enum {
                    variants: &["Single", "SelfTarget", "AreaOfEffect", "MultiTarget"],
                },
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "enabled" => format!("{:?}", self.enabled),
            "mana_cost" => self.mana_cost.to_string(),
            "success_rate" => self.success_rate.to_string(),
            "base_damage" => self.base_damage.to_string(),
            "range" => self.range.to_string(),
            "level_required" => self.level_required.to_string(),
            "effect_value" => self.effect_value.to_string(),
            "effect_type" => self.effect_type.to_string(),
            "effect_modifier" => self.effect_modifier.to_string(),
            "magic_school" => format!("{:?}", self.magic_school),
            "animation_id" => self.animation_id.to_string(),
            "visual_id" => self.visual_id.to_string(),
            "icon_id" => self.icon_id.to_string(),
            "target_type" => format!("{:?}", self.target_type),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "enabled" => {
                if let Some(v) = MagicSpellFlag::from_name(&value) {
                    self.enabled = v;
                    true
                } else {
                    false
                }
            }
            "mana_cost" => {
                if let Ok(v) = value.parse() {
                    self.mana_cost = v;
                    true
                } else {
                    false
                }
            }
            "success_rate" => {
                if let Ok(v) = value.parse() {
                    self.success_rate = v;
                    true
                } else {
                    false
                }
            }
            "base_damage" => {
                if let Ok(v) = value.parse() {
                    self.base_damage = v;
                    true
                } else {
                    false
                }
            }
            "range" => {
                if let Ok(v) = value.parse() {
                    self.range = v;
                    true
                } else {
                    false
                }
            }
            "level_required" => {
                if let Ok(v) = value.parse() {
                    self.level_required = v;
                    true
                } else {
                    false
                }
            }
            "effect_value" => {
                if let Ok(v) = value.parse() {
                    self.effect_value = v;
                    true
                } else {
                    false
                }
            }
            "effect_type" => {
                if let Ok(v) = value.parse() {
                    self.effect_type = v;
                    true
                } else {
                    false
                }
            }
            "effect_modifier" => {
                if let Ok(v) = value.parse() {
                    self.effect_modifier = v;
                    true
                } else {
                    false
                }
            }
            "magic_school" => {
                if let Some(v) = MagicSchool::from_name(&value) {
                    self.magic_school = v;
                    true
                } else {
                    false
                }
            }
            "animation_id" => {
                if let Ok(v) = value.parse() {
                    self.animation_id = v;
                    true
                } else {
                    false
                }
            }
            "visual_id" => {
                if let Ok(v) = value.parse() {
                    self.visual_id = v;
                    true
                } else {
                    false
                }
            }
            "icon_id" => {
                if let Ok(v) = value.parse() {
                    self.icon_id = v;
                    true
                } else {
                    false
                }
            }
            "target_type" => {
                if let Some(v) = SpellTargetType::from_name(&value) {
                    self.target_type = v;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!(
            "[{}] School:{:?} Mana:{} DMG:{} Lv:{}",
            self.id, self.magic_school, self.mana_cost, self.base_damage, self.level_required
        )
    }

    fn detail_title() -> &'static str {
        "Spell Details"
    }
    fn empty_selection_text() -> &'static str {
        "No spell selected"
    }
    fn save_button_label() -> &'static str {
        "Save Spells"
    }
    fn detail_width() -> f32 {
        380.0
    }
}
