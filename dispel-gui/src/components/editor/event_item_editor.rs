use super::editable::{set_str, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::EventItem;

impl EditableRecord for EventItem {
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
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "name" => self.name.clone(),
            "description" => self.description.clone(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "name" => set_str(&mut self.name, value),
            "description" => set_str(&mut self.description, value),
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!("[{}] {}", self.id, self.name)
    }

    fn detail_title() -> &'static str {
        "Event Item Details"
    }
    fn empty_selection_text() -> &'static str {
        "No event item selected"
    }
    fn save_button_label() -> &'static str {
        "Save Event Items"
    }
    fn detail_width() -> f32 {
        320.0
    }
}
