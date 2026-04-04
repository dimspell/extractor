use super::draw_item::DrawItem;
use super::editable::{EditableRecord, FieldDescriptor, FieldKind};

impl EditableRecord for DrawItem {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "map_id",
                label: "Map ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "x_coord",
                label: "X:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "y_coord",
                label: "Y:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "item_id",
                label: "Item ID:",
                kind: FieldKind::Integer,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "map_id" => self.map_id.to_string(),
            "x_coord" => self.x_coord.to_string(),
            "y_coord" => self.y_coord.to_string(),
            "item_id" => self.item_id.to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "map_id" => {
                if let Ok(v) = value.parse() {
                    self.map_id = v;
                    true
                } else {
                    false
                }
            }
            "x_coord" => {
                if let Ok(v) = value.parse() {
                    self.x_coord = v;
                    true
                } else {
                    false
                }
            }
            "y_coord" => {
                if let Ok(v) = value.parse() {
                    self.y_coord = v;
                    true
                } else {
                    false
                }
            }
            "item_id" => {
                if let Ok(v) = value.parse() {
                    self.item_id = v;
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
            "[Map {}] ({}, {}) Item: {}",
            self.map_id, self.x_coord, self.y_coord, self.item_id
        )
    }

    fn detail_title() -> &'static str {
        "Draw Item Details"
    }
    fn empty_selection_text() -> &'static str {
        "No draw item selected"
    }
    fn save_button_label() -> &'static str {
        "Save Draw Items"
    }
    fn detail_width() -> f32 {
        280.0
    }
}
