use super::editable::{set_i32_enum, set_int, set_str, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::{references::enums::ItemTypeId, NpcLookingDirection, NPC};

fn from_i32_to_item_type_id(value: i32) -> Option<ItemTypeId> {
    ItemTypeId::from_u8(value as u8)
}

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
                name: "looking_direction",
                label: "Direction:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "dialog_id",
                label: "Dialog ID:",
                kind: FieldKind::Integer,
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
                name: "dialogue_face_sprite_id",
                label: "Face Sprite ID:",
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
                name: "unknown_item_id",
                label: "Unknown Item ID:",
                kind: FieldKind::Integer,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "id" => self.id.to_string(),
            "npc_id" => self.npc_id.to_string(),
            "name" => self.name.clone(),
            "party_script_id" => self.party_script_id.to_string(),
            "show_on_event" => self.show_on_event.to_string(),
            "looking_direction" => self.looking_direction.to_string(),
            "dialog_id" => self.dialog_id.to_string(),
            "goto1_x" => self.goto1_x.to_string(),
            "goto1_y" => self.goto1_y.to_string(),
            "goto2_x" => self.goto2_x.to_string(),
            "goto2_y" => self.goto2_y.to_string(),
            "goto3_x" => self.goto3_x.to_string(),
            "goto3_y" => self.goto3_y.to_string(),
            "goto4_x" => self.goto4_x.to_string(),
            "goto4_y" => self.goto4_y.to_string(),
            "dialogue_face_sprite_id" => self.dialogue_face_sprite_id.to_string(),
            "unknown_item_type" => self.unknown_item_type.to_string(),
            "unknown_item_id" => self.unknown_item_id.to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "id" => set_int(&mut self.id, value),
            "npc_id" => set_int(&mut self.npc_id, value),
            "name" => set_str(&mut self.name, value),
            "party_script_id" => set_int(&mut self.party_script_id, value),
            "show_on_event" => set_int(&mut self.show_on_event, value),
            "looking_direction" => set_i32_enum(
                &mut self.looking_direction,
                value,
                NpcLookingDirection::from_i32,
            ),
            "dialog_id" => set_int(&mut self.dialog_id, value),
            "goto1_x" => set_int(&mut self.goto1_x, value),
            "goto1_y" => set_int(&mut self.goto1_y, value),
            "goto2_x" => set_int(&mut self.goto2_x, value),
            "goto2_y" => set_int(&mut self.goto2_y, value),
            "goto3_x" => set_int(&mut self.goto3_x, value),
            "goto3_y" => set_int(&mut self.goto3_y, value),
            "goto4_x" => set_int(&mut self.goto4_x, value),
            "goto4_y" => set_int(&mut self.goto4_y, value),
            "dialogue_face_sprite_id" => set_int(&mut self.dialogue_face_sprite_id, value),
            "unknown_item_type" => {
                set_i32_enum(&mut self.unknown_item_type, value, from_i32_to_item_type_id)
            }
            "unknown_item_id" => set_int(&mut self.unknown_item_id, value),
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
