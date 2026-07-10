use crate::components::editable::EditableRecord;
use dispel_core::WeaponItem;

use crate::editable_record_fields;

editable_record_fields!(WeaponItem, {
    { name = String / "Name:" },
    { description = TextArea / "Description:" },
    { base_price = Integer / "Base Price (gold):" },
    { health_points = Integer / "HP Bonus:" },
    { mana_points = Integer / "MP Bonus:" },
    { strength = Integer / "STR Bonus:" },
    { agility = Integer / "AGI Bonus:" },
    { wisdom = Integer / "WIS Bonus:" },
    { constitution = Integer / "CON Bonus:" },
    { to_dodge = Integer / "Dodge Bonus:" },
    { to_hit = Integer / "Hit Bonus:" },
    { attack = Integer / "Attack:" },
    { defense = Integer / "Defense:" },
    { magical_strength = Integer / "Magic Strength:" },
    { durability = Integer / "Durability:" },
    { padding2 = Integer / "Padding 2:" },
    { padding3 = Integer / "Padding 3:" },
    { req_strength = Integer / "Required STR:" },
    { padding4 = Integer / "Padding 4:" },
    { req_agility = Integer / "Required AGI:" },
    { padding5 = Integer / "Padding 5:" },
    { req_wisdom = Integer / "Required WIS:" },
    { padding6 = Integer / "Padding 6:" },
    { padding7 = Integer / "Padding 7:" },
    { padding8 = Integer / "Padding 8:" },
});

impl EditableRecord for WeaponItem {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!(
            "[{}] {} - {}g\n  ATK:{}/DEF:{}/MAG:{}\n  STR:{}/AGI:{}/WIS:{}",
            self.id,
            self.name,
            self.base_price,
            self.attack,
            self.defense,
            self.magical_strength,
            self.req_strength,
            self.req_agility,
            self.req_wisdom
        )
    }

    fn detail_title() -> &'static str {
        "Weapon Details"
    }
    fn empty_selection_text() -> &'static str {
        "No weapon selected"
    }
    fn save_button_label() -> &'static str {
        "Save Weapons"
    }
    fn detail_width() -> f32 {
        280.0
    }
}
