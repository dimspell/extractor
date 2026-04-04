use super::editable::{EditableRecord, FieldDescriptor, FieldKind};
use super::party_ini_db::PartyIniNpc;

impl EditableRecord for PartyIniNpc {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "name",
                label: "Name:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "unknown1",
                label: "Unknown 1:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown2",
                label: "Unknown 2:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown3",
                label: "Unknown 3:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown4",
                label: "Unknown 4:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown5",
                label: "Unknown 5:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "unknown6",
                label: "Unknown 6:",
                kind: FieldKind::Integer,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "name" => self.name.clone(),
            "unknown1" => self.unknown1.to_string(),
            "unknown2" => self.unknown2.to_string(),
            "unknown3" => self.unknown3.to_string(),
            "unknown4" => self.unknown4.to_string(),
            "unknown5" => self.unknown5.to_string(),
            "unknown6" => self.unknown6.to_string(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "name" => {
                self.name = value;
                true
            }
            "unknown1" => {
                if let Ok(v) = value.parse() {
                    self.unknown1 = v;
                    true
                } else {
                    false
                }
            }
            "unknown2" => {
                if let Ok(v) = value.parse() {
                    self.unknown2 = v;
                    true
                } else {
                    false
                }
            }
            "unknown3" => {
                if let Ok(v) = value.parse() {
                    self.unknown3 = v;
                    true
                } else {
                    false
                }
            }
            "unknown4" => {
                if let Ok(v) = value.parse() {
                    self.unknown4 = v;
                    true
                } else {
                    false
                }
            }
            "unknown5" => {
                if let Ok(v) = value.parse() {
                    self.unknown5 = v;
                    true
                } else {
                    false
                }
            }
            "unknown6" => {
                if let Ok(v) = value.parse() {
                    self.unknown6 = v;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!("[{}] {}", 0, self.name)
    }

    fn detail_title() -> &'static str {
        "Party NPC Details"
    }
    fn empty_selection_text() -> &'static str {
        "No party NPC selected"
    }
    fn save_button_label() -> &'static str {
        "Save Party NPCs"
    }
    fn detail_width() -> f32 {
        300.0
    }
}
