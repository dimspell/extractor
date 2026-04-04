use super::editable::{EditableRecord, FieldDescriptor, FieldKind};
use super::wave_ini::WaveIni;

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
            "id" => {
                if let Ok(v) = value.parse() {
                    self.id = v;
                    true
                } else {
                    false
                }
            }
            "snf_filename" => {
                self.snf_filename = if value.is_empty() { None } else { Some(value) };
                true
            }
            "unknown_flag" => {
                self.unknown_flag = if value.is_empty() { None } else { Some(value) };
                true
            }
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
