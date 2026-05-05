use super::editable::{set_int, set_opt_str, EditableRecord, FieldDescriptor, FieldKind};
use dispel_core::MapIni;

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
            "id" => set_int(&mut self.id, value),
            "event_id_on_camera_move" => set_int(&mut self.event_id_on_camera_move, value),
            "start_pos_x" => set_int(&mut self.start_pos_x, value),
            "start_pos_y" => set_int(&mut self.start_pos_y, value),
            "map_id" => set_int(&mut self.map_id, value),
            "monsters_filename" => set_opt_str(&mut self.monsters_filename, value),
            "npc_filename" => set_opt_str(&mut self.npc_filename, value),
            "extra_filename" => set_opt_str(&mut self.extra_filename, value),
            "cd_music_track_number" => set_int(&mut self.cd_music_track_number, value),
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
