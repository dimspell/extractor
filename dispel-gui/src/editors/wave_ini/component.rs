use super::editable::{set_int, set_opt_str, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::WaveIni;

impl EditableRecord for WaveIni {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "id",
                label: "ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "snf_filename",
                label: "SNF Filename:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "unknown_flag",
                label: "Flag:",
                kind: FieldKind::String,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "id" => self.id.to_string(),
            "snf_filename" => self.snf_filename.clone().unwrap_or_default(),
            "unknown_flag" => self.unknown_flag.clone().unwrap_or_default(),
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "id" => set_int(&mut self.id, value),
            "snf_filename" => set_opt_str(&mut self.snf_filename, value),
            "unknown_flag" => set_opt_str(&mut self.unknown_flag, value),
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!(
            "[{}] {} - {}",
            self.id,
            self.snf_filename.as_deref().unwrap_or("null"),
            self.unknown_flag.as_deref().unwrap_or("null")
        )
    }

    fn detail_title() -> &'static str {
        "Wave Details"
    }
    fn empty_selection_text() -> &'static str {
        "No wave selected"
    }
    fn save_button_label() -> &'static str {
        "Save Waves"
    }
}
