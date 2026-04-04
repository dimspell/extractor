use super::dialog::Dialog;
use super::editable::{EditableRecord, FieldDescriptor, FieldKind};

impl EditableRecord for Dialog {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "id",
                label: "ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "previous_event_id",
                label: "Previous Event:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "next_dialog_to_check",
                label: "Next Dialog:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "dialog_type",
                label: "Type:",
                kind: FieldKind::Enum {
                    variants: &["Normal", "Choice"],
                },
            },
            FieldDescriptor {
                name: "dialog_owner",
                label: "Owner:",
                kind: FieldKind::Enum {
                    variants: &["Player", "NPC"],
                },
            },
            FieldDescriptor {
                name: "dialog_id",
                label: "Dialog ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "event_id",
                label: "Event ID:",
                kind: FieldKind::Integer,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "id" => self.id.to_string(),
            "previous_event_id" => self
                .previous_event_id
                .map_or(String::new(), |v| v.to_string()),
            "next_dialog_to_check" => self
                .next_dialog_to_check
                .map_or(String::new(), |v| v.to_string()),
            "dialog_type" => self
                .dialog_type
                .map_or(String::new(), |v| v.value().to_string()),
            "dialog_owner" => self
                .dialog_owner
                .map_or(String::new(), |v| v.value().to_string()),
            "dialog_id" => self.dialog_id.map_or(String::new(), |v| v.to_string()),
            "event_id" => self.event_id.map_or(String::new(), |v| v.to_string()),
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
            "previous_event_id" => {
                if value.is_empty() {
                    self.previous_event_id = None;
                    true
                } else if let Ok(v) = value.parse() {
                    self.previous_event_id = Some(v);
                    true
                } else {
                    false
                }
            }
            "next_dialog_to_check" => {
                if value.is_empty() {
                    self.next_dialog_to_check = None;
                    true
                } else if let Ok(v) = value.parse() {
                    self.next_dialog_to_check = Some(v);
                    true
                } else {
                    false
                }
            }
            "dialog_type" => {
                if value.is_empty() {
                    self.dialog_type = None;
                    true
                } else if let Ok(v) = value.parse() {
                    if let Some(t) = super::enums::DialogType::from_i32(v) {
                        self.dialog_type = Some(t);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            "dialog_owner" => {
                if value.is_empty() {
                    self.dialog_owner = None;
                    true
                } else if let Ok(v) = value.parse() {
                    if let Some(t) = super::enums::DialogOwner::from_i32(v) {
                        self.dialog_owner = Some(t);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            "dialog_id" => {
                if value.is_empty() {
                    self.dialog_id = None;
                    true
                } else if let Ok(v) = value.parse() {
                    self.dialog_id = Some(v);
                    true
                } else {
                    false
                }
            }
            "event_id" => {
                if value.is_empty() {
                    self.event_id = None;
                    true
                } else if let Ok(v) = value.parse() {
                    self.event_id = Some(v);
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
            "[{}] {} -> {}",
            self.id,
            self.dialog_type
                .map(|t| match t {
                    super::enums::DialogType::Normal => "N",
                    super::enums::DialogType::Choice => "C",
                })
                .unwrap_or("?"),
            self.dialog_owner
                .map(|o| match o {
                    super::enums::DialogOwner::Player => "Player",
                    super::enums::DialogOwner::Npc => "NPC",
                })
                .unwrap_or("?")
        )
    }

    fn detail_title() -> &'static str {
        "Dialog Details"
    }

    fn empty_selection_text() -> &'static str {
        "No dialog selected"
    }

    fn save_button_label() -> &'static str {
        "Save Dialog"
    }

    fn detail_width() -> f32 {
        340.0
    }
}
