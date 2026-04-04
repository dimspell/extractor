use super::editable::{EditableRecord, FieldDescriptor, FieldKind};
use super::enums::EventType;
use super::event_ini::Event;

impl EditableRecord for Event {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "event_id",
                label: "Event ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "previous_event_id",
                label: "Previous Event ID:",
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
            "previous_event_id" => self.previous_event_id.to_string(),
            "event_type" => format!("{:?}", self.event_type),
            "event_filename" => self.event_filename.clone().unwrap_or_default(),
            "counter" => self.counter.to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "event_id" => {
                if let Ok(v) = value.parse() {
                    self.event_id = v;
                    true
                } else {
                    false
                }
            }
            "previous_event_id" => {
                if let Ok(v) = value.parse() {
                    self.previous_event_id = v;
                    true
                } else {
                    false
                }
            }
            "event_type" => {
                if let Ok(v) = value.parse::<i32>() {
                    if let Some(t) = super::enums::EventType::from_i32(v) {
                        self.event_type = t;
                        return true;
                    }
                }
                false
            }
            "event_filename" => {
                self.event_filename = if value.is_empty() { None } else { Some(value) };
                true
            }
            "counter" => {
                if let Ok(v) = value.parse() {
                    self.counter = v;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!(
            "[{}] Type: {:?} (prev: {})",
            self.event_id, self.event_type, self.previous_event_id
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
