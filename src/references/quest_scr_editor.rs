use super::editable::{EditableRecord, FieldDescriptor, FieldKind};
use super::quest_scr::Quest;

impl EditableRecord for Quest {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "id",
                label: "ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "type_id",
                label: "Type:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "title",
                label: "Title:",
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
            "type_id" => self.type_id.to_string(),
            "title" => self.title.clone().unwrap_or_default(),
            "description" => self.description.clone().unwrap_or_default(),
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
            "type_id" => {
                if let Ok(v) = value.parse() {
                    self.type_id = v;
                    true
                } else {
                    false
                }
            }
            "title" => {
                self.title = if value.is_empty() { None } else { Some(value) };
                true
            }
            "description" => {
                self.description = if value.is_empty() { None } else { Some(value) };
                true
            }
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        let title = self.title.as_deref().unwrap_or("???");
        format!(
            "[{}] {}",
            self.id,
            &title.chars().take(40).collect::<String>()
        )
    }

    fn detail_title() -> &'static str {
        "Quest Details"
    }
    fn empty_selection_text() -> &'static str {
        "No quest selected"
    }
    fn save_button_label() -> &'static str {
        "Save Quests"
    }
    fn detail_width() -> f32 {
        340.0
    }
}
