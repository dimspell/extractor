use crate::components::editable::EditableRecord;
use dispel_core::PartyIniNpc;

use crate::editable_record_fields;

editable_record_fields!(PartyIniNpc, {
    { name = String / "Name:" },
    { unknown1 = Integer / "Unknown 1:" },
    { unknown2 = Integer / "Unknown 2:" },
    { starting_level = Integer / "Starting Level:" },
    { unknown4 = Integer / "Unknown 4:" },
    { unknown5 = Integer / "Unknown 5:" },
    { unknown6 = Integer / "Unknown 6:" },
});

impl EditableRecord for PartyIniNpc {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!("[{}] {}", 0, self.name)
    }

    fn detail_title() -> &'static str {
        "Party NPC Details"
    }
    fn empty_selection_text() -> &'static str {
        "No party NPC selected"
    }
    fn save_button_label() -> &'static str {
        "Save Party NPCs"
    }
    fn detail_width() -> f32 {
        300.0
    }
}
