use super::editable::{EditableRecord, FieldDescriptor, FieldKind};
use super::misc_item_db::MiscItem;

impl EditableRecord for MiscItem {
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
            FieldDescriptor {
                name: "base_price",
                label: "Base Price:",
                kind: FieldKind::Integer,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "name" => self.name.clone(),
            "description" => self.description.clone(),
            "base_price" => self.base_price.to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "name" => {
                self.name = value;
                true
            }
            "description" => {
                self.description = value;
                true
            }
            "base_price" => {
                if let Ok(v) = value.parse() {
                    self.base_price = v;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!("[{}] {} - {}g", self.id, self.name, self.base_price)
    }

    fn detail_title() -> &'static str {
        "Misc Item Details"
    }
    fn empty_selection_text() -> &'static str {
        "No misc item selected"
    }
    fn save_button_label() -> &'static str {
        "Save Misc Items"
    }
    fn detail_width() -> f32 {
        320.0
    }
}
