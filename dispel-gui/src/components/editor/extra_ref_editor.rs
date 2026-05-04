use super::editable::{
    fmt_enum, set_enum, set_int, set_str, EditableRecord, FieldDescriptor, FieldKind,
};
use dispel_core::{
    ExtraObjectType, ExtraRef, ItemTypeId, SmallRange0to3, Special9999Flag, SpecialPatternFlag,
    VisibilityType,
};

const ITEM_TYPES: FieldKind = FieldKind::Enum {
    variants: &["Weapon", "Healing", "Edit", "Event", "Misc"],
};

fn hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ")
}

fn parse_hex_string(s: &str) -> Option<Vec<u8>> {
    s.split_whitespace()
        .map(|part| u8::from_str_radix(part, 16).ok())
        .collect()
}

impl EditableRecord for ExtraRef {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "id",
                label: "ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown1",
                label: "Unknown 1:",
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
                    variants: &[
                        "Chest",
                        "Door",
                        "Sign",
                        "Altar",
                        "Interactive",
                        "Magic",
                        "Unknown",
                    ],
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
                name: "unknown2",
                label: "Unknown 2:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "unknown3",
                label: "Unknown 3:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "closed",
                label: "Closed:",
                kind: FieldKind::Enum {
                    variants: &["False", "True"],
                },
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
                name: "unknown4",
                label: "Unknown 4:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "required_item_id2",
                label: "Required Item 2:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "required_item_type_id2",
                label: "Required Type 2:",
                kind: ITEM_TYPES,
            },
            FieldDescriptor {
                name: "unknown5",
                label: "Unknown 5:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown6",
                label: "Unknown 6:",
                kind: FieldKind::Enum {
                    variants: &["0", "9999"],
                },
            },
            FieldDescriptor {
                name: "unknown7",
                label: "Unknown 7:",
                kind: FieldKind::Enum {
                    variants: &["0", "9999"],
                },
            },
            FieldDescriptor {
                name: "unknown8",
                label: "Unknown 8:",
                kind: FieldKind::Enum {
                    variants: &["0", "9999"],
                },
            },
            FieldDescriptor {
                name: "unknown9",
                label: "Unknown 9:",
                kind: FieldKind::Enum {
                    variants: &["0", "9999"],
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
                kind: ITEM_TYPES,
            },
            FieldDescriptor {
                name: "unknown10",
                label: "Unknown 10:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "item_count",
                label: "Item Count:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown11",
                label: "Unknown 11:",
                kind: FieldKind::Enum {
                    variants: &["0", "28", "84", "258", "9999"],
                },
            },
            FieldDescriptor {
                name: "unknown12",
                label: "Unknown 12:",
                kind: FieldKind::Enum {
                    variants: &["False", "True"],
                },
            },
            FieldDescriptor {
                name: "unknown13",
                label: "Unknown 13:",
                kind: FieldKind::Enum {
                    variants: &["0", "9999"],
                },
            },
            FieldDescriptor {
                name: "unknown14",
                label: "Unknown 14:",
                kind: FieldKind::String,
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
                name: "unknown15",
                label: "Unknown 15:",
                kind: FieldKind::Enum {
                    variants: &["0", "1", "2", "3"],
                },
            },
            FieldDescriptor {
                name: "unknown16",
                label: "Unknown 16:",
                kind: FieldKind::Enum {
                    variants: &["0", "1", "2", "3"],
                },
            },
            FieldDescriptor {
                name: "unknown17",
                label: "Unknown 17:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "interactive_element_type",
                label: "Interactive Type:",
                kind: FieldKind::Enum {
                    variants: &["0", "1", "2", "3"],
                },
            },
            FieldDescriptor {
                name: "unknown18",
                label: "Unknown 18:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "is_quest_element",
                label: "Quest Element:",
                kind: FieldKind::Enum {
                    variants: &["False", "True"],
                },
            },
            FieldDescriptor {
                name: "unknown20",
                label: "Unknown 20:",
                kind: FieldKind::Enum {
                    variants: &["False", "True"],
                },
            },
            FieldDescriptor {
                name: "unknown21",
                label: "Unknown 21:",
                kind: FieldKind::Enum {
                    variants: &["False", "True"],
                },
            },
            FieldDescriptor {
                name: "unknown22",
                label: "Unknown 22:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown23",
                label: "Unknown 23:",
                kind: FieldKind::Enum {
                    variants: &["False", "True"],
                },
            },
            FieldDescriptor {
                name: "visibility",
                label: "Visibility:",
                kind: FieldKind::Enum {
                    variants: &["Visible0", "Visible10"],
                },
            },
            FieldDescriptor {
                name: "unknown24",
                label: "Unknown 24:",
                kind: FieldKind::Enum {
                    variants: &["False", "True"],
                },
            },
            FieldDescriptor {
                name: "unknown25",
                label: "Unknown 25:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown26",
                label: "Unknown 26:",
                kind: FieldKind::Enum {
                    variants: &["False", "True"],
                },
            },
            FieldDescriptor {
                name: "unknown27",
                label: "Unknown 27:",
                kind: FieldKind::Enum {
                    variants: &["False", "True"],
                },
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "id" => self.id.to_string(),
            "unknown1" => self.unknown1.to_string(),
            "ext_id" => self.ext_id.to_string(),
            "name" => self.name.clone(),
            "object_type" => fmt_enum(&self.object_type),
            "x_pos" => self.x_pos.to_string(),
            "y_pos" => self.y_pos.to_string(),
            "rotation" => self.rotation.to_string(),
            "unknown2" => hex_string(&self.unknown2),
            "unknown3" => self.unknown3.to_string(),
            "closed" => self.closed.to_string(),
            "required_item_id" => self.required_item_id.to_string(),
            "required_item_type_id" => fmt_enum(&self.required_item_type_id),
            "unknown4" => self.unknown4.to_string(),
            "required_item_id2" => self.required_item_id2.to_string(),
            "required_item_type_id2" => fmt_enum(&self.required_item_type_id2),
            "unknown5" => self.unknown5.to_string(),
            "unknown6" => self.unknown6.to_string(),
            "unknown7" => self.unknown7.to_string(),
            "unknown8" => self.unknown8.to_string(),
            "unknown9" => self.unknown9.to_string(),
            "gold_amount" => self.gold_amount.to_string(),
            "item_id" => self.item_id.to_string(),
            "item_type_id" => fmt_enum(&self.item_type_id),
            "unknown10" => self.unknown10.to_string(),
            "item_count" => self.item_count.to_string(),
            "unknown11" => self.unknown11.to_string(),
            "unknown12" => self.unknown12.to_string(),
            "unknown13" => self.unknown13.to_string(),
            "unknown14" => hex_string(&self.unknown14),
            "event_id" => self.event_id.to_string(),
            "message_id" => self.message_id.to_string(),
            "unknown15" => self.unknown15.to_string(),
            "unknown16" => self.unknown16.to_string(),
            "unknown17" => self.unknown17.to_string(),
            "interactive_element_type" => self.interactive_element_type.to_string(),
            "unknown18" => hex_string(&self.unknown18),
            "is_quest_element" => self.is_quest_element.to_string(),
            "unknown20" => self.unknown20.to_string(),
            "unknown21" => self.unknown21.to_string(),
            "unknown22" => self.unknown22.to_string(),
            "unknown23" => self.unknown23.to_string(),
            "visibility" => fmt_enum(&self.visibility),
            "unknown24" => self.unknown24.to_string(),
            "unknown25" => self.unknown25.to_string(),
            "unknown26" => self.unknown26.to_string(),
            "unknown27" => self.unknown27.to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "id" => set_int(&mut self.id, value),
            "unknown1" => set_int(&mut self.unknown1, value),
            "ext_id" => set_int(&mut self.ext_id, value),
            "name" => set_str(&mut self.name, value),
            "object_type" => set_enum(&mut self.object_type, value, ExtraObjectType::from_name),
            "x_pos" => set_int(&mut self.x_pos, value),
            "y_pos" => set_int(&mut self.y_pos, value),
            "rotation" => set_int(&mut self.rotation, value),
            "unknown2" => parse_hex_string(&value).map_or(false, |v| { self.unknown2 = v; true }),
            "unknown3" => set_int(&mut self.unknown3, value),
            "closed" => set_int(&mut self.closed, value),
            "required_item_id" => set_int(&mut self.required_item_id, value),
            "required_item_type_id" => set_enum(
                &mut self.required_item_type_id,
                value,
                ItemTypeId::from_name,
            ),
            "unknown4" => set_int(&mut self.unknown4, value),
            "required_item_id2" => set_int(&mut self.required_item_id2, value),
            "required_item_type_id2" => set_enum(
                &mut self.required_item_type_id2,
                value,
                ItemTypeId::from_name,
            ),
            "unknown5" => set_int(&mut self.unknown5, value),
            "unknown6" => set_enum(&mut self.unknown6, value, Special9999Flag::from_name),
            "unknown7" => set_enum(&mut self.unknown7, value, Special9999Flag::from_name),
            "unknown8" => set_enum(&mut self.unknown8, value, Special9999Flag::from_name),
            "unknown9" => set_enum(&mut self.unknown9, value, Special9999Flag::from_name),
            "gold_amount" => set_int(&mut self.gold_amount, value),
            "item_id" => set_int(&mut self.item_id, value),
            "item_type_id" => set_enum(&mut self.item_type_id, value, ItemTypeId::from_name),
            "unknown10" => set_int(&mut self.unknown10, value),
            "item_count" => set_int(&mut self.item_count, value),
            "unknown11" => set_enum(&mut self.unknown11, value, SpecialPatternFlag::from_name),
            "unknown12" => set_int(&mut self.unknown12, value),
            "unknown13" => set_enum(&mut self.unknown13, value, Special9999Flag::from_name),
            "unknown14" => parse_hex_string(&value).map_or(false, |v| { self.unknown14 = v; true }),
            "event_id" => set_int(&mut self.event_id, value),
            "message_id" => set_int(&mut self.message_id, value),
            "unknown15" => set_enum(&mut self.unknown15, value, SmallRange0to3::from_name),
            "unknown16" => set_enum(&mut self.unknown16, value, SmallRange0to3::from_name),
            "unknown17" => set_int(&mut self.unknown17, value),
            "interactive_element_type" => set_int(&mut self.interactive_element_type, value),
            "unknown18" => parse_hex_string(&value).map_or(false, |v| { self.unknown18 = v; true }),
            "is_quest_element" => set_int(&mut self.is_quest_element, value),
            "unknown20" => set_int(&mut self.unknown20, value),
            "unknown21" => set_int(&mut self.unknown21, value),
            "unknown22" => set_int(&mut self.unknown22, value),
            "unknown23" => set_int(&mut self.unknown23, value),
            "visibility" => set_enum(&mut self.visibility, value, VisibilityType::from_name),
            "unknown24" => set_int(&mut self.unknown24, value),
            "unknown25" => set_int(&mut self.unknown25, value),
            "unknown26" => set_int(&mut self.unknown26, value),
            "unknown27" => set_int(&mut self.unknown27, value),
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
