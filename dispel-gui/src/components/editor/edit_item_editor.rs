use super::editable::{
    fmt_enum, set_enum, set_int, set_str, EditableRecord, FieldDescriptor, FieldKind,
};
use dispel_core::{EditItem, EditItemEffect, EditItemModification};

impl EditableRecord for EditItem {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "name",
                label: "Name:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "description",
                label: "Description:",
                kind: FieldKind::TextArea,
            },
            FieldDescriptor {
                name: "base_price",
                label: "Base Price:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "health_points",
                label: "HP:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "mana_points",
                label: "MP:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "strength",
                label: "STR:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "agility",
                label: "AGI:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "wisdom",
                label: "WIS:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "constitution",
                label: "CON:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "to_dodge",
                label: "Dodge:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "to_hit",
                label: "Hit:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "offense",
                label: "Offense:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "defense",
                label: "Defense:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "magical_power",
                label: "Magic Power:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "item_destroying_power",
                label: "Durability Cost:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "modifies_item",
                label: "Modifies Item:",
                kind: FieldKind::Enum {
                    variants: &["DoesNotModify", "CanModify"],
                },
            },
            FieldDescriptor {
                name: "additional_effect",
                label: "Effect:",
                kind: FieldKind::Enum {
                    variants: &["None", "Fire", "ManaDrain"],
                },
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "name" => self.name.clone(),
            "description" => self.description.clone(),
            "base_price" => self.base_price.to_string(),
            "health_points" => self.health_points.to_string(),
            "mana_points" => self.mana_points.to_string(),
            "strength" => self.strength.to_string(),
            "agility" => self.agility.to_string(),
            "wisdom" => self.wisdom.to_string(),
            "constitution" => self.constitution.to_string(),
            "to_dodge" => self.to_dodge.to_string(),
            "to_hit" => self.to_hit.to_string(),
            "offense" => self.offense.to_string(),
            "defense" => self.defense.to_string(),
            "magical_power" => self.magical_power.to_string(),
            "item_destroying_power" => self.item_destroying_power.to_string(),
            "modifies_item" => fmt_enum(&self.modifies_item),
            "additional_effect" => fmt_enum(&self.additional_effect),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "name" => set_str(&mut self.name, value),
            "description" => set_str(&mut self.description, value),
            "base_price" => set_int(&mut self.base_price, value),
            "health_points" => set_int(&mut self.health_points, value),
            "mana_points" => set_int(&mut self.mana_points, value),
            "strength" => set_int(&mut self.strength, value),
            "agility" => set_int(&mut self.agility, value),
            "wisdom" => set_int(&mut self.wisdom, value),
            "constitution" => set_int(&mut self.constitution, value),
            "to_dodge" => set_int(&mut self.to_dodge, value),
            "to_hit" => set_int(&mut self.to_hit, value),
            "offense" => set_int(&mut self.offense, value),
            "defense" => set_int(&mut self.defense, value),
            "magical_power" => set_int(&mut self.magical_power, value),
            "item_destroying_power" => set_int(&mut self.item_destroying_power, value),
            "modifies_item" => set_enum(
                &mut self.modifies_item,
                value,
                EditItemModification::from_name,
            ),
            "additional_effect" => set_enum(
                &mut self.additional_effect,
                value,
                EditItemEffect::from_name,
            ),
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!(
            "[{}] {} - {}g (ATK:{}/DEF:{})",
            self.index, self.name, self.base_price, self.offense, self.defense
        )
    }

    fn detail_title() -> &'static str {
        "Edit Item Details"
    }
    fn empty_selection_text() -> &'static str {
        "No edit item selected"
    }
    fn save_button_label() -> &'static str {
        "Save Edit Items"
    }
    fn detail_width() -> f32 {
        340.0
    }
}
