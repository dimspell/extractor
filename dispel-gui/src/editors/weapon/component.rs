use crate::components::editable::EditableRecord;
use dispel_core::WeaponItem;

use crate::editable_record_fields;

editable_record_fields!(WeaponItem, {
    { name = String / "Name:" },
    { description = TextArea / "Description:" },
    { base_price = Integer / "Base Price (gold):" },
    { weapon_item_id = Integer / "Runtime Item ID:" },
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
    { reserved_0x108 = Integer / "Reserved (0x108):" },
    { reserved_0x10a = Integer / "Reserved (0x10A):" },
    { req_strength = Integer / "Required STR:" },
    { reserved_0x10e = Integer / "Reserved (0x10E):" },
    { req_agility = Integer / "Required AGI:" },
    { reserved_0x112 = Integer / "Reserved (0x112):" },
    { req_wisdom = Integer / "Required WIS:" },
    { reserved_0x116 = Integer / "Reserved (0x116):" },
    { reserved_0x118 = Integer / "Reserved (0x118):" },
    { reserved_0x11a = Integer / "Reserved (0x11A):" },
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
