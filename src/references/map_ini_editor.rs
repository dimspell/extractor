use super::editable::{EditableRecord, FieldDescriptor, FieldKind};
use super::map_ini::MapIni;

impl EditableRecord for MapIni {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor {
                name: "id",
                label: "ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "event_id_on_camera_move",
                label: "Camera Move Event:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "start_pos_x",
                label: "Start X:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "start_pos_y",
                label: "Start Y:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "map_id",
                label: "Map ID:",
                kind: FieldKind::Integer,
            },
            FieldDescriptor {
                name: "monsters_filename",
                label: "Monster File:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "npc_filename",
                label: "NPC File:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "extra_filename",
                label: "Extra File:",
                kind: FieldKind::String,
            },
            FieldDescriptor {
                name: "cd_music_track_number",
                label: "CD Track:",
                kind: FieldKind::Integer,
            },
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "id" => self.id.to_string(),
            "event_id_on_camera_move" => self.event_id_on_camera_move.to_string(),
            "start_pos_x" => self.start_pos_x.to_string(),
            "start_pos_y" => self.start_pos_y.to_string(),
            "map_id" => self.map_id.to_string(),
            "monsters_filename" => self.monsters_filename.clone().unwrap_or_default(),
            "npc_filename" => self.npc_filename.clone().unwrap_or_default(),
            "extra_filename" => self.extra_filename.clone().unwrap_or_default(),
            "cd_music_track_number" => self.cd_music_track_number.to_string(),
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
            "event_id_on_camera_move" => {
                if let Ok(v) = value.parse() {
                    self.event_id_on_camera_move = v;
                    true
                } else {
                    false
                }
            }
            "start_pos_x" => {
                if let Ok(v) = value.parse() {
                    self.start_pos_x = v;
                    true
                } else {
                    false
                }
            }
            "start_pos_y" => {
                if let Ok(v) = value.parse() {
                    self.start_pos_y = v;
                    true
                } else {
                    false
                }
            }
            "map_id" => {
                if let Ok(v) = value.parse() {
                    self.map_id = v;
                    true
                } else {
                    false
                }
            }
            "monsters_filename" => {
                self.monsters_filename = if value.is_empty() { None } else { Some(value) };
                true
            }
            "npc_filename" => {
                self.npc_filename = if value.is_empty() { None } else { Some(value) };
                true
            }
            "extra_filename" => {
                self.extra_filename = if value.is_empty() { None } else { Some(value) };
                true
            }
            "cd_music_track_number" => {
                if let Ok(v) = value.parse() {
                    self.cd_music_track_number = v;
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
            "[{}] Map {} (Mon: {}, NPC: {})",
            self.id,
            self.map_id,
            self.monsters_filename.as_deref().unwrap_or("???"),
            self.npc_filename.as_deref().unwrap_or("???")
        )
    }

    fn detail_title() -> &'static str {
        "Map Configuration"
    }
    fn empty_selection_text() -> &'static str {
        "No map selected"
    }
    fn save_button_label() -> &'static str {
        "Save Map Ini"
    }
    fn detail_width() -> f32 {
        340.0
    }
}
