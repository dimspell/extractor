use super::editable::{
    get_opt_int, get_opt_val, set_int, set_opt_i32_enum, set_opt_int, EditableRecord,
    FieldDescriptor, FieldKind,
};
use dispel_core::{DialogOwner, DialogType, DialogueScript};

impl EditableRecord for DialogueScript {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "id",
                label: "ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "required_event_id",
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
                name: "triggered_event_id",
                label: "Event ID:",
                kind: FieldKind::Integer,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "id" => self.id.to_string(),
            "required_event_id" => get_opt_int(self.required_event_id),
            "next_dialog_to_check" => get_opt_int(self.next_dialog_to_check),
            "dialog_type" => get_opt_val(self.dialog_type, |v| v.value().to_string()),
            "dialog_owner" => get_opt_val(self.dialog_owner, |v| v.value().to_string()),
            "dialog_id" => get_opt_int(self.dialog_id),
            "event_id" => get_opt_int(self.triggered_event_id),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "id" => set_int(&mut self.id, value),
            "required_event_id" => set_opt_int(&mut self.required_event_id, value),
            "next_dialog_to_check" => set_opt_int(&mut self.next_dialog_to_check, value),
            "dialog_type" => set_opt_i32_enum(&mut self.dialog_type, value, DialogType::from_i32),
            "dialog_owner" => {
                set_opt_i32_enum(&mut self.dialog_owner, value, DialogOwner::from_i32)
            }
            "dialog_id" => set_opt_int(&mut self.dialog_id, value),
            "triggered_event_id" => set_opt_int(&mut self.triggered_event_id, value),
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!(
            "[{}] {} -> {}",
            self.id,
            self.dialog_type
                .map(|t| match t {
                    DialogType::Normal => "N",
                    DialogType::Choice => "C",
                })
                .unwrap_or("?"),
            self.dialog_owner
                .map(|o| match o {
                    DialogOwner::Player => "Player",
                    DialogOwner::Npc => "NPC",
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
