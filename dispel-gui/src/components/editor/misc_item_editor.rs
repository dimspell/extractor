use super::editable::{set_int, set_str, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::MiscItem;

fn hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ")
}

fn parse_hex_string(s: &str) -> Option<Vec<u8>> {
    s.split_whitespace()
        .map(|part| u8::from_str_radix(part, 16).ok())
        .collect()
}

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
                kind: FieldKind::TextArea,
            },
            FieldDescriptor {
                name: "base_price",
                label: "Base Price:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "padding",
                label: "Padding:",
                kind: FieldKind::String,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "name" => self.name.clone(),
            "description" => self.description.clone(),
            "base_price" => self.base_price.to_string(),
            "padding" => hex_string(&self.padding),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "name" => set_str(&mut self.name, value),
            "description" => set_str(&mut self.description, value),
            "base_price" => set_int(&mut self.base_price, value),
            "padding" => parse_hex_string(&value).map_or(false, |v| {
                if v.len() == 20 {
                    self.padding = v.try_into().unwrap();
                    true
                } else {
                    false
                }
            }),
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
