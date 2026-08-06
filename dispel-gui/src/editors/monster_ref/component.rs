use crate::components::editable::EditableRecord;
use dispel_core::{BooleanFlag, MonsterRef};

use crate::editable_record_fields;

editable_record_fields!(MonsterRef, {
    { file_id = Integer / "File ID:" },
    { mon_id = Lookup("monster_names") / "Monster ID:" },
    { pos_x = Integer / "Position X:" },
    { pos_y = Integer / "Position Y:" },
    { padding1 = DispEnum(BooleanFlag, ["True", "False"]) / "Flag 1:" },
    { padding2 = DispEnum(BooleanFlag, ["True", "False"]) / "Flag 2:" },
    { padding3 = Integer / "Flag 3 (0):" },
    { padding4 = Integer / "Flag 4:" },
    { event_id = Integer / "Event ID:" },
    { loot_item1 = CompositeItem("items") / "Loot 1:" },
    { loot_item2 = CompositeItem("items") / "Loot 2:" },
    { loot_item3 = CompositeItem("items") / "Loot 3:" },
    { padding12 = Integer / "Padding 12:" },
    { padding13 = DispEnum(BooleanFlag, ["True", "False"]) / "Padding 13:" },
});

impl EditableRecord for MonsterRef {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!(
            "[{}] File:{} Monster:{} Pos:({},{})",
            self.index, self.file_id, self.mon_id, self.pos_x, self.pos_y
        )
    }

    fn list_label_with_lookups(
        &self,
        lookups: &std::collections::HashMap<String, Vec<(String, String)>>,
    ) -> String {
        let monster_name = lookups
            .get("monster_names")
            .and_then(|entries| {
                entries
                    .iter()
                    .find(|(id, _)| id == &self.mon_id.to_string())
            })
            .map(|(_, name)| name.as_str())
            .unwrap_or("???");
        format!(
            "[{}] File:{} Monster:{} Pos:({},{})",
            self.index, self.file_id, monster_name, self.pos_x, self.pos_y
        )
    }

    fn detail_title() -> &'static str {
        "Monster Placement Details"
    }
    fn empty_selection_text() -> &'static str {
        "No monster placement selected"
    }
    fn save_button_label() -> &'static str {
        "Save Monster Ref"
    }
    fn detail_width() -> f32 {
        380.0
    }
}
