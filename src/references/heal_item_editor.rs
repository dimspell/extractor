use super::editable::{EditableRecord, FieldDescriptor, FieldKind};
use super::enums::HealItemFlag;
use super::heal_item_db::HealItem;

impl EditableRecord for HealItem {
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
                label: "HP Restore:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "mana_points",
                label: "MP Restore:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "restore_full_health",
                label: "Full HP:",
                kind: FieldKind::Enum {
                    variants: &["None", "FullRestoration"],
                },
            },
            FieldDescriptor {
                name: "restore_full_mana",
                label: "Full MP:",
                kind: FieldKind::Enum {
                    variants: &["None", "FullRestoration"],
                },
            },
            FieldDescriptor {
                name: "poison_heal",
                label: "Cure Poison:",
                kind: FieldKind::Enum {
                    variants: &["None", "FullRestoration"],
                },
            },
            FieldDescriptor {
                name: "petrif_heal",
                label: "Cure Petrify:",
                kind: FieldKind::Enum {
                    variants: &["None", "FullRestoration"],
                },
            },
            FieldDescriptor {
                name: "polimorph_heal",
                label: "Cure Polymorph:",
                kind: FieldKind::Enum {
                    variants: &["None", "FullRestoration"],
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
            "restore_full_health" => format!("{:?}", self.restore_full_health),
            "restore_full_mana" => format!("{:?}", self.restore_full_mana),
            "poison_heal" => format!("{:?}", self.poison_heal),
            "petrif_heal" => format!("{:?}", self.petrif_heal),
            "polimorph_heal" => format!("{:?}", self.polimorph_heal),
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
            "restore_full_health" => {
                if let Some(v) = HealItemFlag::from_name(&value) {
                    self.restore_full_health = v;
                    true
                } else {
                    false
                }
            }
            "restore_full_mana" => {
                if let Some(v) = HealItemFlag::from_name(&value) {
                    self.restore_full_mana = v;
                    true
                } else {
                    false
                }
            }
            "poison_heal" => {
                if let Some(v) = HealItemFlag::from_name(&value) {
                    self.poison_heal = v;
                    true
                } else {
                    false
                }
            }
            "petrif_heal" => {
                if let Some(v) = HealItemFlag::from_name(&value) {
                    self.petrif_heal = v;
                    true
                } else {
                    false
                }
            }
            "polimorph_heal" => {
                if let Some(v) = HealItemFlag::from_name(&value) {
                    self.polimorph_heal = v;
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
            "[{}] {} - {}g (HP:{}/MP:{})",
            self.id, self.name, self.base_price, self.health_points, self.mana_points
        )
    }

    fn detail_title() -> &'static str {
        "Heal Item Details"
    }
    fn empty_selection_text() -> &'static str {
        "No heal item selected"
    }
    fn save_button_label() -> &'static str {
        "Save Heal Items"
    }
    fn detail_width() -> f32 {
        320.0
    }
}
