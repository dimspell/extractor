use crate::components::editable::{
    fmt_enum, set_enum, set_int, EditableRecord, FieldDescriptor, FieldKind,
};
use dispel_core::{MagicSchool, MagicSpell, MagicSpellConstant, MagicSpellFlag, SpellTargetType};

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
                name: "flag1",
                label: "Flag 1:",
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
                name: "reserved1",
                label: "Reserved 1:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "reserved2",
                label: "Reserved 2:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "flag2",
                label: "Flag 2:",
                kind: FieldKind::Enum {
                    variants: &["Disabled", "Enabled"],
                },
            },
            FieldDescriptor {
                name: "range",
                label: "Range:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "reserved3",
                label: "Reserved 3:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "level_required",
                label: "Level Required:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "constant1",
                label: "Constant 1:",
                kind: FieldKind::Enum {
                    variants: &["Invalid", "Standard"],
                },
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
                name: "reserved4",
                label: "Reserved 4:",
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
                name: "flag3",
                label: "Flag 3:",
                kind: FieldKind::Enum {
                    variants: &["Disabled", "Enabled"],
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
            "flag1" => fmt_enum(&self.flag1),
            "mana_cost" => self.mana_cost.to_string(),
            "success_rate" => self.success_rate.to_string(),
            "base_damage" => self.base_damage.to_string(),
            "reserved1" => self.reserved1.to_string(),
            "reserved2" => self.reserved2.to_string(),
            "flag2" => fmt_enum(&self.flag2),
            "range" => self.range.to_string(),
            "reserved3" => self.reserved3.to_string(),
            "level_required" => self.level_required.to_string(),
            "constant1" => fmt_enum(&self.constant1),
            "effect_value" => self.effect_value.to_string(),
            "effect_type" => self.effect_type.to_string(),
            "effect_modifier" => self.effect_modifier.to_string(),
            "reserved4" => self.reserved4.to_string(),
            "magic_school" => fmt_enum(&self.magic_school),
            "flag3" => fmt_enum(&self.flag3),
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
            "flag1" => set_enum(&mut self.flag1, value, MagicSpellFlag::from_name),
            "mana_cost" => set_int(&mut self.mana_cost, value),
            "success_rate" => set_int(&mut self.success_rate, value),
            "base_damage" => set_int(&mut self.base_damage, value),
            "reserved1" => set_int(&mut self.reserved1, value),
            "reserved2" => set_int(&mut self.reserved2, value),
            "flag2" => set_enum(&mut self.flag2, value, MagicSpellFlag::from_name),
            "range" => set_int(&mut self.range, value),
            "reserved3" => set_int(&mut self.reserved3, value),
            "level_required" => set_int(&mut self.level_required, value),
            "constant1" => set_enum(&mut self.constant1, value, MagicSpellConstant::from_name),
            "effect_value" => set_int(&mut self.effect_value, value),
            "effect_type" => set_int(&mut self.effect_type, value),
            "effect_modifier" => set_int(&mut self.effect_modifier, value),
            "reserved4" => set_int(&mut self.reserved4, value),
            "magic_school" => set_enum(&mut self.magic_school, value, MagicSchool::from_name),
            "flag3" => set_enum(&mut self.flag3, value, MagicSpellFlag::from_name),
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
