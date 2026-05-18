use crate::components::editable::EditableRecord;
use dispel_core::{MagicSchool, MagicSpell, MagicSpellConstant, MagicSpellFlag, SpellTargetType};

use crate::editable_record_fields;

editable_record_fields!(MagicSpell, {
    { enabled = Enum(MagicSpellFlag, ["Disabled", "Enabled"]) / "Enabled:" },
    { flag1 = Enum(MagicSpellFlag, ["Disabled", "Enabled"]) / "Flag 1:" },
    { mana_cost = Integer / "Mana Cost:" },
    { success_rate = Integer / "Success Rate:" },
    { base_damage = Integer / "Base Damage:" },
    { reserved1 = Integer / "Reserved 1:" },
    { reserved2 = Integer / "Reserved 2:" },
    { flag2 = Enum(MagicSpellFlag, ["Disabled", "Enabled"]) / "Flag 2:" },
    { range = Integer / "Range:" },
    { reserved3 = Integer / "Reserved 3:" },
    { level_required = Integer / "Level Required:" },
    { constant1 = Enum(MagicSpellConstant, ["Invalid", "Standard"]) / "Constant 1:" },
    { effect_value = Integer / "Effect Value:" },
    { effect_type = Integer / "Effect Type:" },
    { effect_modifier = Integer / "Effect Modifier:" },
    { reserved4 = Integer / "Reserved 4:" },
    { magic_school = Enum(MagicSchool, ["Unknown", "School1", "School2", "School3", "School4", "School5", "School6"]) / "Magic School:" },
    { flag3 = Enum(MagicSpellFlag, ["Disabled", "Enabled"]) / "Flag 3:" },
    { animation_id = Integer / "Animation ID:" },
    { visual_id = Integer / "Visual ID:" },
    { icon_id = Integer / "Icon ID:" },
    { target_type = Enum(SpellTargetType, ["Single", "SelfTarget", "AreaOfEffect", "MultiTarget"]) / "Target Type:" },
});

impl EditableRecord for MagicSpell {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!(
            "[{}] School:{:?} Mana:{} DMG:{} Lv:{}",
            self.id, self.magic_school, self.mana_cost, self.base_damage, self.level_required
        )
    }

    fn detail_title() -> &'static str {
        "Spell Details"
    }
    fn empty_selection_text() -> &'static str {
        "No spell selected"
    }
    fn save_button_label() -> &'static str {
        "Save Spells"
    }
    fn detail_width() -> f32 {
        380.0
    }
}
