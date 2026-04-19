use super::editable::{set_int, set_opt_str, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::Extra;

impl EditableRecord for Extra {
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
                name: "unknown",
                label: "Unknown:",
                kind: FieldKind::Integer,
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
            "unknown" => self.unknown.to_string(),
            "description" => self.description.clone().unwrap_or_default(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "id" => set_int(&mut self.id, value),
            "sprite_filename" => set_opt_str(&mut self.sprite_filename, value),
            "unknown" => set_int(&mut self.unknown, value),
            "description" => set_opt_str(&mut self.description, value),
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!(
            "[{}] {}",
            self.id,
            self.sprite_filename.as_deref().unwrap_or("???")
        )
    }

    fn detail_title() -> &'static str {
        "Extra Object Details"
    }
    fn empty_selection_text() -> &'static str {
        "No extra object selected"
    }
    fn save_button_label() -> &'static str {
        "Save Extras"
    }
    fn detail_width() -> f32 {
        280.0
    }
}
