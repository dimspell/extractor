use crate::components::editable::EditableRecord;
use dispel_core::MapIni;

use crate::editable_record_fields;

editable_record_fields!(MapIni, {
    { id = Integer / "ID:" },
    { event_id_on_camera_move = Integer / "Camera Move Event:" },
    { start_pos_x = Integer / "Start X:" },
    { start_pos_y = Integer / "Start Y:" },
    { map_id = Integer / "Map ID:" },
    { monsters_filename = OptStr / "Monster File:" },
    { npc_filename = OptStr / "NPC File:" },
    { extra_filename = OptStr / "Extra File:" },
    { cd_music_track_number = Integer / "CD Track:" },
});

impl EditableRecord for MapIni {
    crate::editable_record_delegate!();

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
