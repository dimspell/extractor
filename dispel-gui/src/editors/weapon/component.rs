use crate::components::editable::{set_int, set_str, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::WeaponItem;

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
                kind: FieldKind::TextArea,
            },
            FieldDescriptor {
                name: "base_price",
                label: "Base Price (gold):",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "padding1",
                label: "Padding 1:",
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
                name: "req_strength",
                label: "Required STR:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "padding4",
                label: "Padding 4:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "req_agility",
                label: "Required AGI:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "padding5",
                label: "Padding 5:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "req_wisdom",
                label: "Required WIS:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "padding6",
                label: "Padding 6:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "padding7",
                label: "Padding 7:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "padding8",
                label: "Padding 8:",
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
            "padding2" => self.padding2.to_string(),
            "padding3" => self.padding3.to_string(),
            "req_strength" => self.req_strength.to_string(),
            "padding4" => self.padding4.to_string(),
            "req_agility" => self.req_agility.to_string(),
            "padding5" => self.padding5.to_string(),
            "req_wisdom" => self.req_wisdom.to_string(),
            "padding6" => self.padding6.to_string(),
            "padding7" => self.padding7.to_string(),
            "padding8" => self.padding8.to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "name" => set_str(&mut self.name, value),
            "description" => set_str(&mut self.description, value),
            "base_price" => set_int(&mut self.base_price, value),
            "padding1" => set_int(&mut self.padding1, value),
            "health_points" => set_int(&mut self.health_points, value),
            "mana_points" => set_int(&mut self.mana_points, value),
            "strength" => set_int(&mut self.strength, value),
            "agility" => set_int(&mut self.agility, value),
            "wisdom" => set_int(&mut self.wisdom, value),
            "constitution" => set_int(&mut self.constitution, value),
            "to_dodge" => set_int(&mut self.to_dodge, value),
            "to_hit" => set_int(&mut self.to_hit, value),
            "attack" => set_int(&mut self.attack, value),
            "defense" => set_int(&mut self.defense, value),
            "magical_strength" => set_int(&mut self.magical_strength, value),
            "durability" => set_int(&mut self.durability, value),
            "padding2" => set_int(&mut self.padding2, value),
            "padding3" => set_int(&mut self.padding3, value),
            "req_strength" => set_int(&mut self.req_strength, value),
            "padding4" => set_int(&mut self.padding4, value),
            "req_agility" => set_int(&mut self.req_agility, value),
            "padding5" => set_int(&mut self.padding5, value),
            "req_wisdom" => set_int(&mut self.req_wisdom, value),
            "padding6" => set_int(&mut self.padding6, value),
            "padding7" => set_int(&mut self.padding7, value),
            "padding8" => set_int(&mut self.padding8, value),
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
