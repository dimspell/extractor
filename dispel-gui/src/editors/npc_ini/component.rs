use crate::components::editable::EditableRecord;
use dispel_core::NpcIni;

use crate::editable_record_fields;

editable_record_fields!(NpcIni, {
    { id = Integer / "ID:" },
    { sprite_filename = OptStr / "Sprite:" },
    { description = TextArea / "Description:" },
});

impl EditableRecord for NpcIni {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!("[{}] {}", self.id, self.description)
    }

    fn detail_title() -> &'static str {
        "NPC Details"
    }
    fn empty_selection_text() -> &'static str {
        "Select an NPC to view details"
    }
    fn save_button_label() -> &'static str {
        "Save NPC Ini"
    }
}
