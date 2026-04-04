use super::dialogue_text::DialogueText;
use super::editable::{EditableRecord, FieldDescriptor, FieldKind};

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
            "id" => {
                if let Ok(v) = value.parse() {
                    self.id = v;
                    true
                } else {
                    false
                }
            }
            "text" => {
                self.text = value;
                true
            }
            "comment" => {
                self.comment = value;
                true
            }
            "param1" => {
                if let Ok(v) = value.parse() {
                    self.param1 = v;
                    true
                } else {
                    false
                }
            }
            "wave_ini_entry_id" => {
                if let Ok(v) = value.parse() {
                    self.wave_ini_entry_id = v;
                    true
                } else {
                    false
                }
            }
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
