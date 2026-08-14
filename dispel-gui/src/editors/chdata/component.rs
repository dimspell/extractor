use crate::components::editable::EditableRecord;
use dispel_core::ChData;

use crate::editable_record_fields;

editable_record_fields!(ChData, {
    { unused_name = String / "Name:" },
    { warrior_strength = Integer / "Warrior STR:" },
    { warrior_constitution = Integer / "Warrior CON:" },
    { warrior_wisdom = Integer / "Warrior WIS:" },
    { warrior_agility = Integer / "Warrior AGI:" },
    { knight_strength = Integer / "Knight STR:" },
    { knight_constitution = Integer / "Knight CON:" },
    { knight_wisdom = Integer / "Knight WIS:" },
    { knight_agility = Integer / "Knight AGI:" },
    { archer_strength = Integer / "Archer STR:" },
    { archer_constitution = Integer / "Archer CON:" },
    { archer_wisdom = Integer / "Archer WIS:" },
    { archer_agility = Integer / "Archer AGI:" },
    { mage_strength = Integer / "Mage STR:" },
    { mage_constitution = Integer / "Mage CON:" },
    { mage_wisdom = Integer / "Mage WIS:" },
    { mage_agility = Integer / "Mage AGI:" },
    { reserved_stat = Integer / "Reserved:" },
    { warrior_offense_bonus = Integer / "Warrior Offense:" },
    { knight_defense_bonus = Integer / "Knight Defense:" },
    { archer_dodge_bonus = Integer / "Archer Dodge:" },
    { archer_hit_bonus = Integer / "Archer Hit:" },
    { mage_magic_power_bonus = Integer / "Mage Magic Power:" },
});

impl EditableRecord for ChData {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!("[0] {}", self.unused_name)
    }

    fn detail_title() -> &'static str {
        "Character Data"
    }
    fn empty_selection_text() -> &'static str {
        "No character data loaded"
    }
    fn save_button_label() -> &'static str {
        "Save ChData"
    }
    fn detail_width() -> f32 {
        380.0
    }
}
