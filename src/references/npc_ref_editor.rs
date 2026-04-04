use super::editable::{EditableRecord, FieldDescriptor, FieldKind};
use super::npc_ref::NPC;

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
            "npc_id" => {
                if let Ok(v) = value.parse() {
                    self.npc_id = v;
                    true
                } else {
                    false
                }
            }
            "name" => {
                self.name = value;
                true
            }
            "party_script_id" => {
                if let Ok(v) = value.parse() {
                    self.party_script_id = v;
                    true
                } else {
                    false
                }
            }
            "show_on_event" => {
                if let Ok(v) = value.parse() {
                    self.show_on_event = v;
                    true
                } else {
                    false
                }
            }
            "looking_direction" => {
                if let Ok(v) = value.parse() {
                    self.looking_direction = super::enums::NpcLookingDirection::from_i32(v)
                        .unwrap_or(self.looking_direction);
                    true
                } else {
                    false
                }
            }
            "dialog_id" => {
                if let Ok(v) = value.parse() {
                    self.dialog_id = v;
                    true
                } else {
                    false
                }
            }
            "goto1_x" => {
                if let Ok(v) = value.parse() {
                    self.goto1_x = v;
                    true
                } else {
                    false
                }
            }
            "goto1_y" => {
                if let Ok(v) = value.parse() {
                    self.goto1_y = v;
                    true
                } else {
                    false
                }
            }
            "goto2_x" => {
                if let Ok(v) = value.parse() {
                    self.goto2_x = v;
                    true
                } else {
                    false
                }
            }
            "goto2_y" => {
                if let Ok(v) = value.parse() {
                    self.goto2_y = v;
                    true
                } else {
                    false
                }
            }
            "goto3_x" => {
                if let Ok(v) = value.parse() {
                    self.goto3_x = v;
                    true
                } else {
                    false
                }
            }
            "goto3_y" => {
                if let Ok(v) = value.parse() {
                    self.goto3_y = v;
                    true
                } else {
                    false
                }
            }
            "goto4_x" => {
                if let Ok(v) = value.parse() {
                    self.goto4_x = v;
                    true
                } else {
                    false
                }
            }
            "goto4_y" => {
                if let Ok(v) = value.parse() {
                    self.goto4_y = v;
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
