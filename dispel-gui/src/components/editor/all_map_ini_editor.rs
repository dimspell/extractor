use super::editable::{
    set_i32_enum, set_int, set_opt_str, set_str, EditableRecord, FieldDescriptor, FieldKind,
};
use dispel_core::{Map, MapLighting};

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
            "id" => set_int(&mut self.id, value),
            "map_filename" => set_str(&mut self.map_filename, value),
            "map_name" => set_str(&mut self.map_name, value),
            "pgp_filename" => set_opt_str(&mut self.pgp_filename, value),
            "dlg_filename" => set_opt_str(&mut self.dlg_filename, value),
            "lighting" => set_i32_enum(&mut self.lighting, value, MapLighting::from_i32),
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
