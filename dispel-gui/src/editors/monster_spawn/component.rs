use crate::components::editable::EditableRecord;
use crate::editable_record_fields;
use dispel_core::MonsterSpawn;

editable_record_fields!(MonsterSpawn, {
    { serial = Integer / "Serial:" },
    { monster_no = Integer / "Monster No:" },
    { x = Integer / "X:" },
    { y = Integer / "Y:" },
    { persistent = i32Enum(dispel_core::BooleanFlag, ["False", "True"]) / "Persistent:" },
    { burrow = i32Enum(dispel_core::BooleanFlag, ["False", "True"]) / "Burrow:" },
    { item_group = Integer / "Item Group:" },
    { drop_attribute = Integer / "Drop Attr:" },
});

impl EditableRecord for MonsterSpawn {
    crate::editable_record_delegate!();
    fn list_label(&self) -> String {
        format!(
            "Monster spawn (serial={}, species={}, x={}, y={})",
            self.serial, self.monster_no, self.x, self.y
        )
    }
    fn detail_title() -> &'static str {
        "Monster Spawn Details"
    }
    fn empty_selection_text() -> &'static str {
        "No monster spawn selected"
    }
    fn save_button_label() -> &'static str {
        "Save Monster Spawns"
    }
}
