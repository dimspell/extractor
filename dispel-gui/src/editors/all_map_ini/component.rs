use crate::components::editable::EditableRecord;
use dispel_core::{Map, MapLighting};

use crate::editable_record_fields;

editable_record_fields!(Map, {
    { id = Integer / "ID:" },
    { map_filename = String / "Map File:" },
    { map_name = String / "Map Name:" },
    { pgp_filename = OptStr / "Dialogue File:" },
    { dlg_filename = OptStr / "Script File:" },
    { lighting = i32Enum(MapLighting, ["Dark", "Light"]) / "Lighting:" },
});

impl EditableRecord for Map {
    crate::editable_record_delegate!();

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
