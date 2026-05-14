use crate::components::editable::{set_int, set_str, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::ChData;

impl EditableRecord for ChData {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "unused_name",
                label: "Name:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "warrior_strength",
                label: "Warrior STR:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "warrior_constitution",
                label: "Warrior CON:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "warrior_wisdom",
                label: "Warrior WIS:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "warrior_agility",
                label: "Warrior AGI:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "knight_strength",
                label: "Knight STR:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "knight_constitution",
                label: "Knight CON:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "knight_wisdom",
                label: "Knight WIS:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "knight_agility",
                label: "Knight AGI:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "archer_strength",
                label: "Archer STR:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "archer_constitution",
                label: "Archer CON:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "archer_wisdom",
                label: "Archer WIS:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "archer_agility",
                label: "Archer AGI:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "mage_strength",
                label: "Mage STR:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "mage_constitution",
                label: "Mage CON:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "mage_wisdom",
                label: "Mage WIS:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "mage_agility",
                label: "Mage AGI:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "reserved_stat",
                label: "Reserved:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "warrior_extra_points",
                label: "Warrior Extra:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "knight_extra_points",
                label: "Knight Extra:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "archer_extra_points",
                label: "Archer Extra:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "mage_extra_points",
                label: "Mage Extra:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "extra_points_per_level",
                label: "Per Level:",
                kind: FieldKind::Integer,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "unused_name" => self.unused_name.clone(),
            "warrior_strength" => self.warrior_strength.to_string(),
            "warrior_constitution" => self.warrior_constitution.to_string(),
            "warrior_wisdom" => self.warrior_wisdom.to_string(),
            "warrior_agility" => self.warrior_agility.to_string(),
            "knight_strength" => self.knight_strength.to_string(),
            "knight_constitution" => self.knight_constitution.to_string(),
            "knight_wisdom" => self.knight_wisdom.to_string(),
            "knight_agility" => self.knight_agility.to_string(),
            "archer_strength" => self.archer_strength.to_string(),
            "archer_constitution" => self.archer_constitution.to_string(),
            "archer_wisdom" => self.archer_wisdom.to_string(),
            "archer_agility" => self.archer_agility.to_string(),
            "mage_strength" => self.mage_strength.to_string(),
            "mage_constitution" => self.mage_constitution.to_string(),
            "mage_wisdom" => self.mage_wisdom.to_string(),
            "mage_agility" => self.mage_agility.to_string(),
            "reserved_stat" => self.reserved_stat.to_string(),
            "warrior_extra_points" => self.warrior_extra_points.to_string(),
            "knight_extra_points" => self.knight_extra_points.to_string(),
            "archer_extra_points" => self.archer_extra_points.to_string(),
            "mage_extra_points" => self.mage_extra_points.to_string(),
            "extra_points_per_level" => self.extra_points_per_level.to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "unused_name" => set_str(&mut self.unused_name, value),
            "warrior_strength" => set_int(&mut self.warrior_strength, value),
            "warrior_constitution" => set_int(&mut self.warrior_constitution, value),
            "warrior_wisdom" => set_int(&mut self.warrior_wisdom, value),
            "warrior_agility" => set_int(&mut self.warrior_agility, value),
            "knight_strength" => set_int(&mut self.knight_strength, value),
            "knight_constitution" => set_int(&mut self.knight_constitution, value),
            "knight_wisdom" => set_int(&mut self.knight_wisdom, value),
            "knight_agility" => set_int(&mut self.knight_agility, value),
            "archer_strength" => set_int(&mut self.archer_strength, value),
            "archer_constitution" => set_int(&mut self.archer_constitution, value),
            "archer_wisdom" => set_int(&mut self.archer_wisdom, value),
            "archer_agility" => set_int(&mut self.archer_agility, value),
            "mage_strength" => set_int(&mut self.mage_strength, value),
            "mage_constitution" => set_int(&mut self.mage_constitution, value),
            "mage_wisdom" => set_int(&mut self.mage_wisdom, value),
            "mage_agility" => set_int(&mut self.mage_agility, value),
            "reserved_stat" => set_int(&mut self.reserved_stat, value),
            "warrior_extra_points" => set_int(&mut self.warrior_extra_points, value),
            "knight_extra_points" => set_int(&mut self.knight_extra_points, value),
            "archer_extra_points" => set_int(&mut self.archer_extra_points, value),
            "mage_extra_points" => set_int(&mut self.mage_extra_points, value),
            "extra_points_per_level" => set_int(&mut self.extra_points_per_level, value),
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!("[0] {}", self.unused_name)
    }

    fn detail_title() -> &'static str {
        "Character Data"
    }
    fn empty_selection_text() -> &'static str {
        "No character data loaded"
    }
    fn save_button_label() -> &'static str {
        "Save ChData"
    }
    fn detail_width() -> f32 {
        380.0
    }
}
