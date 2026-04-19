use super::editable::{set_int, set_str, set_u8_enum, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::{ExtraObjectType, ExtraRef, ItemTypeId, VisibilityType};

const ITEM_TYPES: FieldKind = FieldKind::Enum {
    variants: &["Weapon", "Healing", "Edit", "Event", "Misc"],
};

impl EditableRecord for ExtraRef {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "id",
                label: "ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "ext_id",
                label: "Extra ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "name",
                label: "Name:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "object_type",
                label: "Type:",
                kind: FieldKind::Enum {
                    variants: &["Chest", "Door", "Sign", "Altar", "Interactive", "Magic"],
                },
            },
            FieldDescriptor {
                name: "x_pos",
                label: "X:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "y_pos",
                label: "Y:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "rotation",
                label: "Rotation:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "closed",
                label: "Closed:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "required_item_id",
                label: "Required Item:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "required_item_type_id",
                label: "Required Type:",
                kind: ITEM_TYPES,
            },
            FieldDescriptor {
                name: "gold_amount",
                label: "Gold:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "item_id",
                label: "Item ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "item_type_id",
                label: "Item Type:",
                kind: ITEM_TYPES,
            },
            FieldDescriptor {
                name: "item_count",
                label: "Item Count:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "event_id",
                label: "Event ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "message_id",
                label: "Message ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "is_quest_element",
                label: "Quest Element:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "interactive_element_type",
                label: "Interactive Type:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "visibility",
                label: "Visibility:",
                kind: FieldKind::Enum {
                    variants: &["Visible0", "Visible10"],
                },
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "id" => self.id.to_string(),
            "ext_id" => self.ext_id.to_string(),
            "name" => self.name.clone(),
            "object_type" => u8::from(self.object_type).to_string(),
            "x_pos" => self.x_pos.to_string(),
            "y_pos" => self.y_pos.to_string(),
            "rotation" => self.rotation.to_string(),
            "closed" => self.closed.to_string(),
            "required_item_id" => self.required_item_id.to_string(),
            "required_item_type_id" => u8::from(self.required_item_type_id).to_string(),
            "gold_amount" => self.gold_amount.to_string(),
            "item_id" => self.item_id.to_string(),
            "item_type_id" => u8::from(self.item_type_id).to_string(),
            "item_count" => self.item_count.to_string(),
            "event_id" => self.event_id.to_string(),
            "message_id" => self.message_id.to_string(),
            "is_quest_element" => self.is_quest_element.to_string(),
            "interactive_element_type" => self.interactive_element_type.to_string(),
            "visibility" => u8::from(self.visibility).to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "id" => set_int(&mut self.id, value),
            "ext_id" => set_int(&mut self.ext_id, value),
            "name" => set_str(&mut self.name, value),
            "object_type" => set_u8_enum(&mut self.object_type, value, ExtraObjectType::from_u8),
            "x_pos" => set_int(&mut self.x_pos, value),
            "y_pos" => set_int(&mut self.y_pos, value),
            "rotation" => set_int(&mut self.rotation, value),
            "closed" => set_int(&mut self.closed, value),
            "required_item_id" => set_int(&mut self.required_item_id, value),
            "required_item_type_id" => {
                set_u8_enum(&mut self.required_item_type_id, value, ItemTypeId::from_u8)
            }
            "gold_amount" => set_int(&mut self.gold_amount, value),
            "item_id" => set_int(&mut self.item_id, value),
            "item_type_id" => set_u8_enum(&mut self.item_type_id, value, ItemTypeId::from_u8),
            "item_count" => set_int(&mut self.item_count, value),
            "event_id" => set_int(&mut self.event_id, value),
            "message_id" => set_int(&mut self.message_id, value),
            "is_quest_element" => set_int(&mut self.is_quest_element, value),
            "interactive_element_type" => set_int(&mut self.interactive_element_type, value),
            "visibility" => set_u8_enum(&mut self.visibility, value, VisibilityType::from_u8),
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!(
            "[{}] {} @ ({}, {})",
            self.id, self.name, self.x_pos, self.y_pos
        )
    }

    fn detail_title() -> &'static str {
        "ExtraRef Details"
    }
    fn empty_selection_text() -> &'static str {
        "No extra ref selected"
    }
    fn save_button_label() -> &'static str {
        "Save ExtraRef"
    }
    fn detail_width() -> f32 {
        340.0
    }
}
