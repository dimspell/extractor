use super::editable::{set_int, set_opt_str, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::MonsterIni;

impl EditableRecord for MonsterIni {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "id",
                label: "ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "name",
                label: "Name:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "sprite_filename",
                label: "Sprite:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "attack",
                label: "Attack Seq:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "hit",
                label: "Hit Seq:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "death",
                label: "Death Seq:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "walking",
                label: "Walking Seq:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "casting_magic",
                label: "Casting Seq:",
                kind: FieldKind::Integer,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "id" => self.id.to_string(),
            "name" => self.name.clone().unwrap_or_default(),
            "sprite_filename" => self.sprite_filename.clone().unwrap_or_default(),
            "attack" => self.attack.to_string(),
            "hit" => self.hit.to_string(),
            "death" => self.death.to_string(),
            "walking" => self.walking.to_string(),
            "casting_magic" => self.casting_magic.to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "id" => set_int(&mut self.id, value),
            "name" => set_opt_str(&mut self.name, value),
            "sprite_filename" => set_opt_str(&mut self.sprite_filename, value),
            "attack" => set_int(&mut self.attack, value),
            "hit" => set_int(&mut self.hit, value),
            "death" => set_int(&mut self.death, value),
            "walking" => set_int(&mut self.walking, value),
            "casting_magic" => set_int(&mut self.casting_magic, value),
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        match &self.name {
            Some(name) => format!("[{}] {}", self.id, name),
            None => format!("[{}]", self.id),
        }
    }

    fn detail_title() -> &'static str {
        "Monster Details"
    }
    fn empty_selection_text() -> &'static str {
        "Select a monster to view details"
    }
    fn save_button_label() -> &'static str {
        "Save Monster Ini"
    }
}
