use crate::components::editable::EditableRecord;
use crate::editable_record_fields;
use dispel_core::ChestSpawn;

editable_record_fields!(ChestSpawn, {
    { serial = Integer / "Serial:" },
    { x = Integer / "X:" },
    { y = Integer / "Y:" },
    { direction = Integer / "Direction:" },
    { item_group = Integer / "Item Group:" },
    { gold_class = Integer / "Gold Class:" },
});

impl EditableRecord for ChestSpawn {
    crate::editable_record_delegate!();
    fn list_label(&self) -> String {
        format!(
            "Chest spawn (serial={}, x={}, y={}, dir={}, group={}, gold={})",
            self.serial, self.x, self.y, self.direction, self.item_group, self.gold_class
        )
    }
    fn detail_title() -> &'static str {
        "Chest Spawn Details"
    }
    fn empty_selection_text() -> &'static str {
        "No chest spawn selected"
    }
    fn save_button_label() -> &'static str {
        "Save Chest Spawns"
    }
}
