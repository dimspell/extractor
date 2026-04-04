use super::all_map_ini::Map;
use super::editable::{EditableRecord, FieldDescriptor, FieldKind};
use super::enums::MapLighting;

impl EditableRecord for Map {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "id",
                label: "ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "map_filename",
                label: "Map File:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "map_name",
                label: "Map Name:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "pgp_filename",
                label: "Dialogue File:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "dlg_filename",
                label: "Script File:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "lighting",
                label: "Lighting:",
                kind: FieldKind::Enum {
                    variants: &["Dark", "Light"],
                },
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "id" => self.id.to_string(),
            "map_filename" => self.map_filename.clone(),
            "map_name" => self.map_name.clone(),
            "pgp_filename" => self.pgp_filename.clone().unwrap_or_default(),
            "dlg_filename" => self.dlg_filename.clone().unwrap_or_default(),
            "lighting" => i32::from(self.lighting).to_string(),
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
            "map_filename" => {
                self.map_filename = value;
                true
            }
            "map_name" => {
                self.map_name = value;
                true
            }
            "pgp_filename" => {
                self.pgp_filename = if value.is_empty() { None } else { Some(value) };
                true
            }
            "dlg_filename" => {
                self.dlg_filename = if value.is_empty() { None } else { Some(value) };
                true
            }
            "lighting" => {
                if let Ok(v) = value.parse() {
                    if let Some(t) = MapLighting::from_i32(v) {
                        self.lighting = t;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!("[{}] {}", self.id, self.map_name)
    }

    fn detail_title() -> &'static str {
        "Map Details"
    }

    fn empty_selection_text() -> &'static str {
        "No map selected"
    }

    fn save_button_label() -> &'static str {
        "Save Map"
    }

    fn detail_width() -> f32 {
        340.0
    }
}
