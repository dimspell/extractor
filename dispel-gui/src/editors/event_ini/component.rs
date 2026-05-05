use super::editable::{
    fmt_enum, set_i32_enum, set_int, set_opt_str, EditableRecord, FieldDescriptor, FieldKind,
};
use dispel_core::{Event, EventType};

impl EditableRecord for Event {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "event_id",
                label: "Event ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "required_event_id",
                label: "Required Event ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "event_type",
                label: "Event Type:",
                kind: FieldKind::Enum {
                    variants: &[
                        "Unknown",
                        "Conditional",
                        "ContinueOnUnsatisfied",
                        "ExecuteOnSatisfied",
                    ],
                },
            },
            FieldDescriptor {
                name: "event_filename",
                label: "Script Filename:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "counter",
                label: "Counter:",
                kind: FieldKind::Integer,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "event_id" => self.event_id.to_string(),
            "required_event_id" => self.required_event_id.to_string(),
            "event_type" => fmt_enum(&self.event_type),
            "event_filename" => self.event_filename.clone().unwrap_or_default(),
            "counter" => self.counter.to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "event_id" => set_int(&mut self.event_id, value),
            "required_event_id" => set_int(&mut self.required_event_id, value),
            "event_type" => set_i32_enum(&mut self.event_type, value, EventType::from_i32),
            "event_filename" => set_opt_str(&mut self.event_filename, value),
            "counter" => set_int(&mut self.counter, value),
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!(
            "[{}] Type: {:?} (prev: {})",
            self.event_id, self.event_type, self.required_event_id
        )
    }

    fn detail_title() -> &'static str {
        "Event Details"
    }
    fn empty_selection_text() -> &'static str {
        "No event selected"
    }
    fn save_button_label() -> &'static str {
        "Save Events"
    }
    fn detail_width() -> f32 {
        320.0
    }
}
