use super::editable::{EditableRecord, FieldDescriptor, FieldKind};
use super::weapons_db::WeaponItem;

impl EditableRecord for WeaponItem {
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
                label: "Base Price (gold):",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "health_points",
                label: "HP Bonus:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "mana_points",
                label: "MP Bonus:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "strength",
                label: "STR Bonus:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "agility",
                label: "AGI Bonus:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "wisdom",
                label: "WIS Bonus:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "constitution",
                label: "CON Bonus:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "to_dodge",
                label: "Dodge Bonus:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "to_hit",
                label: "Hit Bonus:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "attack",
                label: "Attack:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "defense",
                label: "Defense:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "magical_strength",
                label: "Magic Strength:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "durability",
                label: "Durability:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "req_strength",
                label: "Required STR:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "req_agility",
                label: "Required AGI:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "req_wisdom",
                label: "Required WIS:",
                kind: FieldKind::Integer,
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
            "attack" => self.attack.to_string(),
            "defense" => self.defense.to_string(),
            "magical_strength" => self.magical_strength.to_string(),
            "durability" => self.durability.to_string(),
            "req_strength" => self.req_strength.to_string(),
            "req_agility" => self.req_agility.to_string(),
            "req_wisdom" => self.req_wisdom.to_string(),
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
            "attack" => {
                if let Ok(v) = value.parse() {
                    self.attack = v;
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
            "magical_strength" => {
                if let Ok(v) = value.parse() {
                    self.magical_strength = v;
                    true
                } else {
                    false
                }
            }
            "durability" => {
                if let Ok(v) = value.parse() {
                    self.durability = v;
                    true
                } else {
                    false
                }
            }
            "req_strength" => {
                if let Ok(v) = value.parse() {
                    self.req_strength = v;
                    true
                } else {
                    false
                }
            }
            "req_agility" => {
                if let Ok(v) = value.parse() {
                    self.req_agility = v;
                    true
                } else {
                    false
                }
            }
            "req_wisdom" => {
                if let Ok(v) = value.parse() {
                    self.req_wisdom = v;
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
            "[{}] {} - {}g\n  ATK:{}/DEF:{}/MAG:{}\n  STR:{}/AGI:{}/WIS:{}",
            self.id,
            self.name,
            self.base_price,
            self.attack,
            self.defense,
            self.magical_strength,
            self.req_strength,
            self.req_agility,
            self.req_wisdom
        )
    }

    fn detail_title() -> &'static str {
        "Weapon Details"
    }

    fn empty_selection_text() -> &'static str {
        "No weapon selected"
    }

    fn save_button_label() -> &'static str {
        "Save Weapons"
    }

    fn detail_width() -> f32 {
        280.0
    }
}
