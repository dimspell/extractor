use crate::components::editable::EditableRecord;
use dispel_core::MonsterIni;

use crate::editable_record_fields;

editable_record_fields!(MonsterIni, {
    { id = Integer / "ID:" },
    { name = OptStr / "Name:" },
    { sprite_filename = OptStr / "Sprite:" },
    { attack = Integer / "Attack Seq:" },
    { hit = Integer / "Hit Seq:" },
    { death = Integer / "Death Seq:" },
    { walking = Integer / "Walking Seq:" },
    { casting_magic = Integer / "Casting Seq:" },
});

impl EditableRecord for MonsterIni {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        match &self.name {
            Some(name) => format!("[{}] {}", self.id, name),
            None => format!("[{}]", self.id),
        }
    }

    fn detail_title() -> &'static str {
        "Monster Details"
    }
    fn empty_selection_text() -> &'static str {
        "Select a monster to view details"
    }
    fn save_button_label() -> &'static str {
        "Save Monster Ini"
    }
}
