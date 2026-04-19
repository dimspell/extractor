use super::editable::{set_int, set_opt_str, set_str, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::NpcIni;

impl EditableRecord for NpcIni {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "id",
                label: "ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "sprite_filename",
                label: "Sprite:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "description",
                label: "Description:",
                kind: FieldKind::String,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "id" => self.id.to_string(),
            "sprite_filename" => self.sprite_filename.clone().unwrap_or_default(),
            "description" => self.description.clone(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "id" => set_int(&mut self.id, value),
            "sprite_filename" => set_opt_str(&mut self.sprite_filename, value),
            "description" => set_str(&mut self.description, value),
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!("[{}] {}", self.id, self.description)
    }

    fn detail_title() -> &'static str {
        "NPC Details"
    }
    fn empty_selection_text() -> &'static str {
        "Select an NPC to view details"
    }
    fn save_button_label() -> &'static str {
        "Save NPC Ini"
    }
}
