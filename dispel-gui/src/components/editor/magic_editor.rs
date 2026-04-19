use super::editable::{fmt_enum, set_enum, set_int, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::{MagicSchool, MagicSpell, MagicSpellFlag, SpellTargetType};

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
            "enabled" => fmt_enum(&self.enabled),
            "mana_cost" => self.mana_cost.to_string(),
            "success_rate" => self.success_rate.to_string(),
            "base_damage" => self.base_damage.to_string(),
            "range" => self.range.to_string(),
            "level_required" => self.level_required.to_string(),
            "effect_value" => self.effect_value.to_string(),
            "effect_type" => self.effect_type.to_string(),
            "effect_modifier" => self.effect_modifier.to_string(),
            "magic_school" => fmt_enum(&self.magic_school),
            "animation_id" => self.animation_id.to_string(),
            "visual_id" => self.visual_id.to_string(),
            "icon_id" => self.icon_id.to_string(),
            "target_type" => fmt_enum(&self.target_type),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "enabled" => set_enum(&mut self.enabled, value, MagicSpellFlag::from_name),
            "mana_cost" => set_int(&mut self.mana_cost, value),
            "success_rate" => set_int(&mut self.success_rate, value),
            "base_damage" => set_int(&mut self.base_damage, value),
            "range" => set_int(&mut self.range, value),
            "level_required" => set_int(&mut self.level_required, value),
            "effect_value" => set_int(&mut self.effect_value, value),
            "effect_type" => set_int(&mut self.effect_type, value),
            "effect_modifier" => set_int(&mut self.effect_modifier, value),
            "magic_school" => set_enum(&mut self.magic_school, value, MagicSchool::from_name),
            "animation_id" => set_int(&mut self.animation_id, value),
            "visual_id" => set_int(&mut self.visual_id, value),
            "icon_id" => set_int(&mut self.icon_id, value),
            "target_type" => set_enum(&mut self.target_type, value, SpellTargetType::from_name),
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
