use crate::components::editable::EditableRecord;
use dispel_core::{BooleanFlag, MonsterRef};

use crate::editable_record_fields;

editable_record_fields!(MonsterRef, {
    { placement_id = Integer / "Placement ID:" },
    { monster_db_id = Lookup("monster_names") / "Monster ID:" },
    { map_x = Integer / "Spawn X:" },
    { map_y = Integer / "Spawn Y:" },
    { initial_patrol_countdown = DispEnum(BooleanFlag, ["True", "False"]) / "Initial Patrol Countdown:" },
    { skip_ai_action = DispEnum(BooleanFlag, ["True", "False"]) / "Skip AI Action:" },
    { initial_active_flag = Integer / "Initial Active Flag:" },
    { ai_type_override = Integer / "AI Type Override:" },
    { event_id_on_kill = Integer / "Event on Kill:" },
    { loot_item_1 = CompositeItem("items") / "Loot 1:" },
    { loot_item_2 = CompositeItem("items") / "Loot 2:" },
    { loot_item_3 = CompositeItem("items") / "Loot 3:" },
    { drop_all_loot = Integer / "Drop All Loot:" },
    { force_ai_update = DispEnum(BooleanFlag, ["True", "False"]) / "Force AI Update:" },
});

impl EditableRecord for MonsterRef {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!(
            "[{}] File:{} Monster:{} Pos:({},{})",
            self.index, self.placement_id, self.monster_db_id, self.map_x, self.map_y
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
                    .find(|(id, _)| id == &self.monster_db_id.to_string())
            })
            .map(|(_, name)| name.as_str())
            .unwrap_or("???");
        format!(
            "[{}] File:{} Monster:{} Pos:({},{})",
            self.index, self.placement_id, monster_name, self.map_x, self.map_y
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
