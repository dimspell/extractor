use super::edit_item_db::EditItem;
use super::editable::{EditableRecord, FieldDescriptor, FieldKind};
use super::enums::{EditItemEffect, EditItemModification};

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
                kind: FieldKind::String,
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
            "modifies_item" => format!("{:?}", self.modifies_item),
            "additional_effect" => format!("{:?}", self.additional_effect),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "name" => {
                self.name = value;
                true
            }
            "description" => {
                self.description = value;
                true
            }
            "base_price" => {
                if let Ok(v) = value.parse() {
                    self.base_price = v;
                    true
                } else {
                    false
                }
            }
            "health_points" => {
                if let Ok(v) = value.parse() {
                    self.health_points = v;
                    true
                } else {
                    false
                }
            }
            "mana_points" => {
                if let Ok(v) = value.parse() {
                    self.mana_points = v;
                    true
                } else {
                    false
                }
            }
            "strength" => {
                if let Ok(v) = value.parse() {
                    self.strength = v;
                    true
                } else {
                    false
                }
            }
            "agility" => {
                if let Ok(v) = value.parse() {
                    self.agility = v;
                    true
                } else {
                    false
                }
            }
            "wisdom" => {
                if let Ok(v) = value.parse() {
                    self.wisdom = v;
                    true
                } else {
                    false
                }
            }
            "constitution" => {
                if let Ok(v) = value.parse() {
                    self.constitution = v;
                    true
                } else {
                    false
                }
            }
            "to_dodge" => {
                if let Ok(v) = value.parse() {
                    self.to_dodge = v;
                    true
                } else {
                    false
                }
            }
            "to_hit" => {
                if let Ok(v) = value.parse() {
                    self.to_hit = v;
                    true
                } else {
                    false
                }
            }
            "offense" => {
                if let Ok(v) = value.parse() {
                    self.offense = v;
                    true
                } else {
                    false
                }
            }
            "defense" => {
                if let Ok(v) = value.parse() {
                    self.defense = v;
                    true
                } else {
                    false
                }
            }
            "magical_power" => {
                if let Ok(v) = value.parse() {
                    self.magical_power = v;
                    true
                } else {
                    false
                }
            }
            "item_destroying_power" => {
                if let Ok(v) = value.parse() {
                    self.item_destroying_power = v;
                    true
                } else {
                    false
                }
            }
            "modifies_item" => {
                if let Some(v) = EditItemModification::from_name(&value) {
                    self.modifies_item = v;
                    true
                } else {
                    false
                }
            }
            "additional_effect" => {
                if let Some(v) = EditItemEffect::from_name(&value) {
                    self.additional_effect = v;
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
        380.0
    }
}
