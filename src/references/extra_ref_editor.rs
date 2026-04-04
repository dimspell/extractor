use super::editable::{EditableRecord, FieldDescriptor, FieldKind};
use super::enums::{ExtraObjectType, ItemTypeId, VisibilityType};
use super::extra_ref::ExtraRef;

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
                kind: FieldKind::Enum {
                    variants: &["Weapon", "Healing", "Edit", "Event", "Misc"],
                },
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
                kind: FieldKind::Enum {
                    variants: &["Weapon", "Healing", "Edit", "Event", "Misc"],
                },
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
            "id" => {
                if let Ok(v) = value.parse() {
                    self.id = v;
                    true
                } else {
                    false
                }
            }
            "ext_id" => {
                if let Ok(v) = value.parse() {
                    self.ext_id = v;
                    true
                } else {
                    false
                }
            }
            "name" => {
                self.name = value;
                true
            }
            "object_type" => {
                if let Ok(v) = value.parse() {
                    if let Some(t) = ExtraObjectType::from_u8(v) {
                        self.object_type = t;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            "x_pos" => {
                if let Ok(v) = value.parse() {
                    self.x_pos = v;
                    true
                } else {
                    false
                }
            }
            "y_pos" => {
                if let Ok(v) = value.parse() {
                    self.y_pos = v;
                    true
                } else {
                    false
                }
            }
            "rotation" => {
                if let Ok(v) = value.parse() {
                    self.rotation = v;
                    true
                } else {
                    false
                }
            }
            "closed" => {
                if let Ok(v) = value.parse() {
                    self.closed = v;
                    true
                } else {
                    false
                }
            }
            "required_item_id" => {
                if let Ok(v) = value.parse() {
                    self.required_item_id = v;
                    true
                } else {
                    false
                }
            }
            "required_item_type_id" => {
                if let Ok(v) = value.parse() {
                    if let Some(t) = ItemTypeId::from_u8(v) {
                        self.required_item_type_id = t;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            "gold_amount" => {
                if let Ok(v) = value.parse() {
                    self.gold_amount = v;
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
            "item_type_id" => {
                if let Ok(v) = value.parse() {
                    if let Some(t) = ItemTypeId::from_u8(v) {
                        self.item_type_id = t;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            "item_count" => {
                if let Ok(v) = value.parse() {
                    self.item_count = v;
                    true
                } else {
                    false
                }
            }
            "event_id" => {
                if let Ok(v) = value.parse() {
                    self.event_id = v;
                    true
                } else {
                    false
                }
            }
            "message_id" => {
                if let Ok(v) = value.parse() {
                    self.message_id = v;
                    true
                } else {
                    false
                }
            }
            "is_quest_element" => {
                if let Ok(v) = value.parse() {
                    self.is_quest_element = v;
                    true
                } else {
                    false
                }
            }
            "interactive_element_type" => {
                if let Ok(v) = value.parse() {
                    self.interactive_element_type = v;
                    true
                } else {
                    false
                }
            }
            "visibility" => {
                if let Ok(v) = value.parse() {
                    if let Some(t) = VisibilityType::from_u8(v) {
                        self.visibility = t;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
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
