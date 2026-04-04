use super::editable::{EditableRecord, FieldDescriptor, FieldKind};
use super::message_scr::Message as ScrMessage;

impl EditableRecord for ScrMessage {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "id",
                label: "ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "line1",
                label: "Line 1:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "line2",
                label: "Line 2:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "line3",
                label: "Line 3:",
                kind: FieldKind::String,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "id" => self.id.to_string(),
            "line1" => self.line1.clone().unwrap_or_default(),
            "line2" => self.line2.clone().unwrap_or_default(),
            "line3" => self.line3.clone().unwrap_or_default(),
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
            "line1" => {
                self.line1 = if value.is_empty() { None } else { Some(value) };
                true
            }
            "line2" => {
                self.line2 = if value.is_empty() { None } else { Some(value) };
                true
            }
            "line3" => {
                self.line3 = if value.is_empty() { None } else { Some(value) };
                true
            }
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        let text = self.line1.as_deref().unwrap_or("");
        format!(
            "[{}] {}",
            self.id,
            &text.chars().take(40).collect::<String>()
        )
    }

    fn detail_title() -> &'static str {
        "Message Details"
    }
    fn empty_selection_text() -> &'static str {
        "No message selected"
    }
    fn save_button_label() -> &'static str {
        "Save Messages"
    }
    fn detail_width() -> f32 {
        320.0
    }
}
