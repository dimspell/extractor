use super::editable::{
    fmt_enum, set_enum, set_int, set_str, EditableRecord, FieldDescriptor, FieldKind,
};
use dispel_core::{HealItem, HealItemFlag};

impl EditableRecord for HealItem {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        const FLAGS: FieldKind = FieldKind::Enum {
            variants: &["None", "FullRestoration"],
        };
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
                name: "padding1",
                label: "Padding 1:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "padding2",
                label: "Padding 2:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "padding3",
                label: "Padding 3:",
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
                kind: FLAGS,
            },
            FieldDescriptor {
                name: "restore_full_mana",
                label: "Full MP:",
                kind: FLAGS,
            },
            FieldDescriptor {
                name: "poison_heal",
                label: "Cure Poison:",
                kind: FLAGS,
            },
            FieldDescriptor {
                name: "petrif_heal",
                label: "Cure Petrify:",
                kind: FLAGS,
            },
            FieldDescriptor {
                name: "polimorph_heal",
                label: "Cure Polymorph:",
                kind: FLAGS,
            },
            FieldDescriptor {
                name: "padding4",
                label: "Padding 4:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "padding5",
                label: "Padding 5:",
                kind: FieldKind::Integer,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "name" => self.name.clone(),
            "description" => self.description.clone(),
            "base_price" => self.base_price.to_string(),
            "padding1" => self.padding1.to_string(),
            "padding2" => self.padding2.to_string(),
            "padding3" => self.padding3.to_string(),
            "health_points" => self.health_points.to_string(),
            "mana_points" => self.mana_points.to_string(),
            "restore_full_health" => fmt_enum(&self.restore_full_health),
            "restore_full_mana" => fmt_enum(&self.restore_full_mana),
            "poison_heal" => fmt_enum(&self.poison_heal),
            "petrif_heal" => fmt_enum(&self.petrif_heal),
            "polimorph_heal" => fmt_enum(&self.polimorph_heal),
            "padding4" => self.padding4.to_string(),
            "padding5" => self.padding5.to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "name" => set_str(&mut self.name, value),
            "description" => set_str(&mut self.description, value),
            "base_price" => set_int(&mut self.base_price, value),
            "padding1" => set_int(&mut self.padding1, value),
            "padding2" => set_int(&mut self.padding2, value),
            "padding3" => set_int(&mut self.padding3, value),
            "health_points" => set_int(&mut self.health_points, value),
            "mana_points" => set_int(&mut self.mana_points, value),
            "restore_full_health" => set_enum(
                &mut self.restore_full_health,
                value,
                HealItemFlag::from_name,
            ),
            "restore_full_mana" => {
                set_enum(&mut self.restore_full_mana, value, HealItemFlag::from_name)
            }
            "poison_heal" => set_enum(&mut self.poison_heal, value, HealItemFlag::from_name),
            "petrif_heal" => set_enum(&mut self.petrif_heal, value, HealItemFlag::from_name),
            "polimorph_heal" => set_enum(&mut self.polimorph_heal, value, HealItemFlag::from_name),
            "padding4" => set_int(&mut self.padding4, value),
            "padding5" => set_int(&mut self.padding5, value),
            _ => false,
        }
    }

    fn validate_field(&self, field: &str, value: &str) -> Option<String> {
        match field {
            "name" | "description" => {
                if value.trim().is_empty() {
                    Some(format!("{field} cannot be empty"))
                } else {
                    None
                }
            }
            "base_price" | "health_points" | "mana_points" => match value.parse::<i32>() {
                Ok(v) if v < 0 => Some(format!("{field} must be non-negative")),
                Err(_) => Some(format!("{field} must be a valid integer")),
                _ => None,
            },
            "restore_full_health"
            | "restore_full_mana"
            | "poison_heal"
            | "petrif_heal"
            | "polimorph_heal" => {
                if HealItemFlag::from_name(value).is_none() {
                    Some(format!("Invalid {field}"))
                } else {
                    None
                }
            }
            _ => None,
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
