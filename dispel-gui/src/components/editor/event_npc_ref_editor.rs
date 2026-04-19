use super::editable::{set_int, set_str, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::EventNpcRef;

impl EditableRecord for EventNpcRef {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "id",
                label: "ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "event_id",
                label: "Event ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "name",
                label: "Name:",
                kind: FieldKind::String,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "id" => self.id.to_string(),
            "event_id" => self.event_id.to_string(),
            "name" => self.name.clone(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "id" => set_int(&mut self.id, value),
            "event_id" => set_int(&mut self.event_id, value),
            "name" => set_str(&mut self.name, value),
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!("[{}] {} (Event: {})", self.id, self.name, self.event_id)
    }

    fn detail_title() -> &'static str {
        "Event NPC Details"
    }
    fn empty_selection_text() -> &'static str {
        "No event NPC selected"
    }
    fn save_button_label() -> &'static str {
        "Save Event NPCs"
    }
    fn detail_width() -> f32 {
        280.0
    }
}
