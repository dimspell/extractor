use crate::components::editable::EditableRecord;
use dispel_core::{MagicSchool, MagicSpell, MagicSpellFlag, SpellTargetType};

use crate::editable_record_fields;

editable_record_fields!(MagicSpell, {
    { enabled = Enum(MagicSpellFlag, ["Disabled", "Enabled"]) / "Enabled:" },
    { effect_visual_blends_with_background = Boolean / "Effect Visual Blends:" },
    { base_damage = Integer / "Base Damage:" },
    { base_success_rate = Integer / "Base Success Rate:" },
    { mana_cost = Integer / "Mana Cost:" },
    { reserved_0x14 = Integer / "Reserved (0x14):" },
    { reserved_0x18 = Integer / "Reserved (0x18):" },
    { effect_animation_repeats = Boolean / "Effect Animation Repeats:" },
    { range = Integer / "Range:" },
    { reserved_0x24 = Integer / "Reserved (0x24):" },
    { cast_duration = Integer / "Cast Duration:" },
    { animation_data_index = Integer / "Animation Data Index:" },
    { effect_value = Integer / "Effect Value:" },
    { effect_type = Integer / "Effect Type:" },
    { effect_modifier = Integer / "Effect Modifier:" },
    { reserved_0x3c = Integer / "Reserved (0x3C):" },
    { magic_school = Enum(MagicSchool, ["Unknown", "School1", "School2", "School3", "School4", "School5", "School6"]) / "Magic School:" },
    { target_animation_blends_with_background = Boolean / "Target Animation Blends:" },
    { animation_set_id = Integer / "Animation Set ID:" },
    { effect_visual_id = Integer / "Effect Visual ID:" },
    { icon_id = Integer / "Icon ID:" },
    { targeting_mode = Enum(SpellTargetType, ["Single", "SelfTarget", "AreaOfEffect", "MultiTarget"]) / "Targeting Mode:" },
});

impl EditableRecord for MagicSpell {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!(
            "[{}] School:{:?} Mana:{} DMG:{} Duration:{}",
            self.id, self.magic_school, self.mana_cost, self.base_damage, self.cast_duration
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
