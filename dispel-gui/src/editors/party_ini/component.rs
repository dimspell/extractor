use crate::components::editable::EditableRecord;
use dispel_core::PartyIniNpc;

use crate::editable_record_fields;

editable_record_fields!(PartyIniNpc, {
    { name = String / "Name:" },
    { reserved_0x14 = Integer / "Reserved (0x14):" },
    { class_id = Integer / "Class ID:" },
    { starting_level = Integer / "Starting Level:" },
    { pathfinding_mode = Integer / "Pathfinding Mode:" },
    { character_variant = Integer / "Character Variant:" },
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
