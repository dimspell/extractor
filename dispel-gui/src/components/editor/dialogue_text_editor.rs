use super::editable::{set_int, set_str, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::DialogueText;

impl EditableRecord for DialogueText {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "id",
                label: "ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "text",
                label: "Text:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "comment",
                label: "Comment:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "param1",
                label: "Param 1:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "wave_ini_entry_id",
                label: "Wave ID:",
                kind: FieldKind::Integer,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "id" => self.id.to_string(),
            "text" => self.text.clone(),
            "comment" => self.comment.clone(),
            "param1" => self.param1.to_string(),
            "wave_ini_entry_id" => self.wave_ini_entry_id.to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "id" => set_int(&mut self.id, value),
            "text" => set_str(&mut self.text, value),
            "comment" => set_str(&mut self.comment, value),
            "param1" => set_int(&mut self.param1, value),
            "wave_ini_entry_id" => set_int(&mut self.wave_ini_entry_id, value),
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!("[{}] {}", self.id, self.text)
    }

    fn detail_title() -> &'static str {
        "Dialogue Text Details"
    }
    fn empty_selection_text() -> &'static str {
        "No dialogue text selected"
    }
    fn save_button_label() -> &'static str {
        "Save Dialogue Text"
    }
    fn detail_width() -> f32 {
        340.0
    }
}
