use super::editable::{set_int, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::{references::enums::ItemTypeId, DrawItem};

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
            FieldDescriptor {
                name: "item_type",
                label: "Item Type:",
                kind: FieldKind::Enum {
                    variants: &["Weapon", "Healing", "Edit", "Event", "Misc", "Other"],
                },
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "map_id" => self.map_id.to_string(),
            "x_coord" => self.x_coord.to_string(),
            "y_coord" => self.y_coord.to_string(),
            "item_id" => self.item_id.to_string(),
            "item_type" => self.item_type.to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "map_id" => set_int(&mut self.map_id, value),
            "x_coord" => set_int(&mut self.x_coord, value),
            "y_coord" => set_int(&mut self.y_coord, value),
            "item_id" => set_int(&mut self.item_id, value),
            "item_type" => {
                if let Some(item_type) = ItemTypeId::from_name(&value) {
                    self.item_type = item_type;
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
