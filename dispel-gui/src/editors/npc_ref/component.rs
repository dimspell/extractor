use crate::components::editable::{
    set_enum, set_int, set_str, EditableRecord, FieldDescriptor, FieldKind,
};
use dispel_core::references::enums::{
    BooleanFlag, ItemTypeId, NpcLookingDirection, Unknown0110, Unknown012, Unknown0to7,
};
use dispel_core::NPC;

impl EditableRecord for NPC {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "id",
                label: "ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "npc_id",
                label: "NPC ID:",
                kind: FieldKind::Integer,
            },
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
                name: "party_script_id",
                label: "Party Script ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "show_on_event",
                label: "Show on Event:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown_1",
                label: "Unknown 1:",
                kind: FieldKind::Enum {
                    variants: &["0", "1", "2"],
                },
            },
            FieldDescriptor {
                name: "goto1_filled",
                label: "Waypoint 1 Filled:",
                kind: FieldKind::Enum {
                    variants: &["No", "Yes"],
                },
            },
            FieldDescriptor {
                name: "goto2_filled",
                label: "Waypoint 2 Filled:",
                kind: FieldKind::Enum {
                    variants: &["No", "Yes"],
                },
            },
            FieldDescriptor {
                name: "goto3_filled",
                label: "Waypoint 3 Filled:",
                kind: FieldKind::Enum {
                    variants: &["No", "Yes"],
                },
            },
            FieldDescriptor {
                name: "goto4_filled",
                label: "Waypoint 4 Filled:",
                kind: FieldKind::Enum {
                    variants: &["No", "Yes"],
                },
            },
            FieldDescriptor {
                name: "goto1_x",
                label: "Waypoint 1 X:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "goto1_y",
                label: "Waypoint 1 Y:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "goto2_x",
                label: "Waypoint 2 X:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "goto2_y",
                label: "Waypoint 2 Y:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "goto3_x",
                label: "Waypoint 3 X:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "goto3_y",
                label: "Waypoint 3 Y:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "goto4_x",
                label: "Waypoint 4 X:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "goto4_y",
                label: "Waypoint 4 Y:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown_2",
                label: "Unknown 2:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown_3",
                label: "Unknown 3:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown_4",
                label: "Unknown 4:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown_5",
                label: "Unknown 5:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "looking_direction",
                label: "Direction:",
                kind: FieldKind::Enum {
                    variants: &[
                        "Up",
                        "UpRight",
                        "Right",
                        "DownRight",
                        "Down",
                        "DownLeft",
                        "UpLeft",
                    ],
                },
            },
            FieldDescriptor {
                name: "unknown_6",
                label: "Unknown 6:",
                kind: FieldKind::Enum {
                    variants: &["0", "1", "2", "3", "4", "5", "6", "7"],
                },
            },
            FieldDescriptor {
                name: "unknown_7",
                label: "Unknown 7:",
                kind: FieldKind::Enum {
                    variants: &["0", "1", "2", "3", "4", "5", "6", "7"],
                },
            },
            FieldDescriptor {
                name: "unknown_8",
                label: "Unknown 8:",
                kind: FieldKind::Enum {
                    variants: &["0", "1", "2", "3", "4", "5", "6", "7"],
                },
            },
            FieldDescriptor {
                name: "unknown_9",
                label: "Unknown 9:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown_10",
                label: "Unknown 10:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown_11",
                label: "Unknown 11:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown_12",
                label: "Unknown 12:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown_13",
                label: "Unknown 13:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown_14",
                label: "Unknown 14:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown_15",
                label: "Unknown 15:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown_16",
                label: "Unknown 16:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown_17",
                label: "Unknown 17:",
                kind: FieldKind::Enum {
                    variants: &["0", "1", "2"],
                },
            },
            FieldDescriptor {
                name: "unknown_18",
                label: "Unknown 18:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown_item_id",
                label: "Unknown Item ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown_item_type",
                label: "Unknown Item Type:",
                kind: FieldKind::Enum {
                    variants: &["Weapon", "Healing", "Edit", "Event", "Misc", "Other"],
                },
            },
            FieldDescriptor {
                name: "unknown_19",
                label: "Unknown 19:",
                kind: FieldKind::Enum {
                    variants: &["0", "1", "10"],
                },
            },
            FieldDescriptor {
                name: "dialog_id",
                label: "Dialog ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "dialogue_face_sprite_id",
                label: "Face Sprite ID:",
                kind: FieldKind::Integer,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "id" => self.id.to_string(),
            "npc_id" => self.npc_id.to_string(),
            "name" => self.name.clone(),
            "description" => self.description.clone(),
            "party_script_id" => self.party_script_id.to_string(),
            "show_on_event" => self.show_on_event.to_string(),
            "unknown_1" => self.unknown_1.to_string(),
            "goto1_filled" => self.goto1_filled.to_string(),
            "goto2_filled" => self.goto2_filled.to_string(),
            "goto3_filled" => self.goto3_filled.to_string(),
            "goto4_filled" => self.goto4_filled.to_string(),
            "goto1_x" => self.goto1_x.to_string(),
            "goto1_y" => self.goto1_y.to_string(),
            "goto2_x" => self.goto2_x.to_string(),
            "goto2_y" => self.goto2_y.to_string(),
            "goto3_x" => self.goto3_x.to_string(),
            "goto3_y" => self.goto3_y.to_string(),
            "goto4_x" => self.goto4_x.to_string(),
            "goto4_y" => self.goto4_y.to_string(),
            "unknown_2" => self.unknown_2.to_string(),
            "unknown_3" => self.unknown_3.to_string(),
            "unknown_4" => self.unknown_4.to_string(),
            "unknown_5" => self.unknown_5.to_string(),
            "looking_direction" => self.looking_direction.to_string(),
            "unknown_6" => self.unknown_6.to_string(),
            "unknown_7" => self.unknown_7.to_string(),
            "unknown_8" => self.unknown_8.to_string(),
            "unknown_9" => self.unknown_9.to_string(),
            "unknown_10" => self.unknown_10.to_string(),
            "unknown_11" => self.unknown_11.to_string(),
            "unknown_12" => self.unknown_12.to_string(),
            "unknown_13" => self.unknown_13.to_string(),
            "unknown_14" => self.unknown_14.to_string(),
            "unknown_15" => self.unknown_15.to_string(),
            "unknown_16" => self.unknown_16.to_string(),
            "unknown_17" => self.unknown_17.to_string(),
            "unknown_18" => self.unknown_18.to_string(),
            "unknown_item_id" => self.unknown_item_id.to_string(),
            "unknown_item_type" => self.unknown_item_type.to_string(),
            "unknown_19" => self.unknown_19.to_string(),
            "dialog_id" => self.dialog_id.to_string(),
            "dialogue_face_sprite_id" => self.dialogue_face_sprite_id.to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "id" => set_int(&mut self.id, value),
            "npc_id" => set_int(&mut self.npc_id, value),
            "name" => set_str(&mut self.name, value),
            "description" => set_str(&mut self.description, value),
            "party_script_id" => set_int(&mut self.party_script_id, value),
            "show_on_event" => set_int(&mut self.show_on_event, value),
            "unknown_1" => set_enum(&mut self.unknown_1, value, Unknown012::from_name),
            "goto1_filled" => set_enum(&mut self.goto1_filled, value, BooleanFlag::from_name),
            "goto2_filled" => set_enum(&mut self.goto2_filled, value, BooleanFlag::from_name),
            "goto3_filled" => set_enum(&mut self.goto3_filled, value, BooleanFlag::from_name),
            "goto4_filled" => set_enum(&mut self.goto4_filled, value, BooleanFlag::from_name),
            "goto1_x" => set_int(&mut self.goto1_x, value),
            "goto1_y" => set_int(&mut self.goto1_y, value),
            "goto2_x" => set_int(&mut self.goto2_x, value),
            "goto2_y" => set_int(&mut self.goto2_y, value),
            "goto3_x" => set_int(&mut self.goto3_x, value),
            "goto3_y" => set_int(&mut self.goto3_y, value),
            "goto4_x" => set_int(&mut self.goto4_x, value),
            "goto4_y" => set_int(&mut self.goto4_y, value),
            "unknown_2" => set_int(&mut self.unknown_2, value),
            "unknown_3" => set_int(&mut self.unknown_3, value),
            "unknown_4" => set_int(&mut self.unknown_4, value),
            "unknown_5" => set_int(&mut self.unknown_5, value),
            "looking_direction" => set_enum(
                &mut self.looking_direction,
                value,
                NpcLookingDirection::from_name,
            ),
            "unknown_6" => set_enum(&mut self.unknown_6, value, Unknown0to7::from_name),
            "unknown_7" => set_enum(&mut self.unknown_7, value, Unknown0to7::from_name),
            "unknown_8" => set_enum(&mut self.unknown_8, value, Unknown0to7::from_name),
            "unknown_9" => set_int(&mut self.unknown_9, value),
            "unknown_10" => set_int(&mut self.unknown_10, value),
            "unknown_11" => set_int(&mut self.unknown_11, value),
            "unknown_12" => set_int(&mut self.unknown_12, value),
            "unknown_13" => set_int(&mut self.unknown_13, value),
            "unknown_14" => set_int(&mut self.unknown_14, value),
            "unknown_15" => set_int(&mut self.unknown_15, value),
            "unknown_16" => set_int(&mut self.unknown_16, value),
            "unknown_17" => set_enum(&mut self.unknown_17, value, Unknown012::from_name),
            "unknown_18" => set_int(&mut self.unknown_18, value),
            "unknown_item_id" => set_int(&mut self.unknown_item_id, value),
            "unknown_item_type" => {
                set_enum(&mut self.unknown_item_type, value, ItemTypeId::from_name)
            }
            "unknown_19" => set_enum(&mut self.unknown_19, value, Unknown0110::from_name),
            "dialog_id" => set_int(&mut self.dialog_id, value),
            "dialogue_face_sprite_id" => set_int(&mut self.dialogue_face_sprite_id, value),
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!(
            "[{}] {} (NPC {})",
            self.id,
            &self.name.chars().take(20).collect::<String>(),
            self.npc_id
        )
    }

    fn detail_title() -> &'static str {
        "NPC Details"
    }
    fn empty_selection_text() -> &'static str {
        "No NPC selected"
    }
    fn save_button_label() -> &'static str {
        "Save NPCs"
    }
    fn detail_width() -> f32 {
        340.0
    }
}
