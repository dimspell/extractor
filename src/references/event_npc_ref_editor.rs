use super::editable::{EditableRecord, FieldDescriptor, FieldKind};
use super::event_npc_ref::EventNpcRef;

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
            "id" => {
                if let Ok(v) = value.parse() {
                    self.id = v;
                    true
                } else {
                    false
                }
            }
            "event_id" => {
                if let Ok(v) = value.parse() {
                    self.event_id = v;
                    true
                } else {
                    false
                }
            }
            "name" => {
                self.name = value;
                true
            }
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
