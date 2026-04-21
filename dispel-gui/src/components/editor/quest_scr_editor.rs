use super::editable::{set_int, set_opt_str, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::Quest;

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
                kind: FieldKind::TextArea,
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
            "id" => set_int(&mut self.id, value),
            "type_id" => set_int(&mut self.type_id, value),
            "title" => set_opt_str(&mut self.title, value),
            "description" => set_opt_str(&mut self.description, value),
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
