use crate::components::editable::EditableRecord;
use dispel_core::{EditItem, EditItemEffect, EditItemModification};

use crate::editable_record_fields;

editable_record_fields!(EditItem, {
    { name = String / "Name:" },
    { description = TextArea / "Description:" },
    { base_price = Integer / "Base Price:" },
    { padding1 = Integer / "Padding 2:" },
    { padding2 = Integer / "Padding 3:" },
    { health_points = Integer / "HP:" },
    { mana_points = Integer / "MP:" },
    { strength = Integer / "STR:" },
    { agility = Integer / "AGI:" },
    { wisdom = Integer / "WIS:" },
    { constitution = Integer / "CON:" },
    { to_dodge = Integer / "Dodge:" },
    { to_hit = Integer / "Hit:" },
    { offense = Integer / "Offense:" },
    { defense = Integer / "Defense:" },
    { magical_power = Integer / "Magic Power:" },
    { item_destroying_power = Integer / "Durability Cost:" },
    { padding4 = Integer / "Padding 4:" },
    { modifies_item = Enum(EditItemModification, ["DoesNotModify", "CanModify"]) / "Modifies Item:" },
    { additional_effect = Enum(EditItemEffect, ["None", "Fire", "ManaDrain"]) / "Effect:" },
});

impl EditableRecord for EditItem {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!(
            "[{}] {} - {}g (ATK:{}/DEF:{})",
            self.index, self.name, self.base_price, self.offense, self.defense
        )
    }

    fn detail_title() -> &'static str {
        "Edit Item Details"
    }
    fn empty_selection_text() -> &'static str {
        "No edit item selected"
    }
    fn save_button_label() -> &'static str {
        "Save Edit Items"
    }
    fn detail_width() -> f32 {
        340.0
    }
}
