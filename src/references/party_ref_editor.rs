use super::editable::{EditableRecord, FieldDescriptor, FieldKind};
use super::party_ref::PartyRef;

impl EditableRecord for PartyRef {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "id",
                label: "ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "full_name",
                label: "Full Name:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "job_name",
                label: "Job:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "root_map_id",
                label: "Root Map ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "npc_id",
                label: "NPC ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "dlg_when_not_in_party",
                label: "Dialog (not in party):",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "dlg_when_in_party",
                label: "Dialog (in party):",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "ghost_face_id",
                label: "Ghost Face ID:",
                kind: FieldKind::Integer,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "id" => self.id.to_string(),
            "full_name" => self.full_name.clone().unwrap_or_default(),
            "job_name" => self.job_name.clone().unwrap_or_default(),
            "root_map_id" => self.root_map_id.to_string(),
            "npc_id" => self.npc_id.to_string(),
            "dlg_when_not_in_party" => self.dlg_when_not_in_party.to_string(),
            "dlg_when_in_party" => self.dlg_when_in_party.to_string(),
            "ghost_face_id" => self.ghost_face_id.value().to_string(),
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
            "full_name" => {
                self.full_name = if value.is_empty() { None } else { Some(value) };
                true
            }
            "job_name" => {
                self.job_name = if value.is_empty() { None } else { Some(value) };
                true
            }
            "root_map_id" => {
                if let Ok(v) = value.parse() {
                    self.root_map_id = v;
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
            "dlg_when_not_in_party" => {
                if let Ok(v) = value.parse() {
                    self.dlg_when_not_in_party = v;
                    true
                } else {
                    false
                }
            }
            "dlg_when_in_party" => {
                if let Ok(v) = value.parse() {
                    self.dlg_when_in_party = v;
                    true
                } else {
                    false
                }
            }
            "ghost_face_id" => {
                if let Ok(v) = value.parse() {
                    if let Some(f) = super::enums::GhostFaceId::from_i32(v) {
                        self.ghost_face_id = f;
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!(
            "[{}] {} ({})",
            self.id,
            self.full_name.as_deref().unwrap_or("???"),
            self.job_name.as_deref().unwrap_or("???")
        )
    }

    fn detail_title() -> &'static str {
        "Party Member Details"
    }
    fn empty_selection_text() -> &'static str {
        "No party member selected"
    }
    fn save_button_label() -> &'static str {
        "Save Party Ref"
    }
    fn detail_width() -> f32 {
        340.0
    }
}
