use super::editable::{set_int, set_str, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::PartyIniNpc;

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
            "name" => set_str(&mut self.name, value),
            "unknown1" => set_int(&mut self.unknown1, value),
            "unknown2" => set_int(&mut self.unknown2, value),
            "unknown3" => set_int(&mut self.unknown3, value),
            "unknown4" => set_int(&mut self.unknown4, value),
            "unknown5" => set_int(&mut self.unknown5, value),
            "unknown6" => set_int(&mut self.unknown6, value),
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
