use crate::components::editable::EditableRecord;
use dispel_core::{Monster, MonsterAiType, PropertyFlag};

crate::editable_record_fields!(Monster, {
    { id = Integer / "ID:" },
    { name = String / "Name:" },
    { health_points_max = Integer / "HP Max:" },
    { health_points_min = Integer / "HP Min:" },
    { mana_points_max = Integer / "MP Max:" },
    { mana_points_min = Integer / "MP Min:" },
    { walk_speed = Integer / "Walk Speed:" },
    { to_hit_max = Integer / "To Hit Max:" },
    { to_hit_min = Integer / "To Hit Min:" },
    { to_dodge_max = Integer / "Dodge Max:" },
    { to_dodge_min = Integer / "Dodge Min:" },
    { offense_max = Integer / "Offense Max:" },
    { offense_min = Integer / "Offense Min:" },
    { defense_max = Integer / "Defense Max:" },
    { defense_min = Integer / "Defense Min:" },
    { magic_attack_max = Integer / "Magic Atk Max:" },
    { magic_attack_min = Integer / "Magic Atk Min:" },
    { is_undead = FlagBool(PropertyFlag) / "Undead:" },
    { has_blood = FlagBool(PropertyFlag) / "Has Blood:" },
    { ai_type = i32Enum(MonsterAiType, ["Aggressor", "HitAndFlee", "FleeWhenApproached", "AttackWhenOutnumbered", "TeleportTactic", "AttackWhenProvoked", "RunAwayWhenAttacked"]) / "AI Type:" },
    { exp_gain_max = Integer / "EXP Max:" },
    { exp_gain_min = Integer / "EXP Min:" },
    { gold_drop_max = Integer / "Gold Max:" },
    { gold_drop_min = Integer / "Gold Min:" },
    { detection_sight_size = Integer / "Sight Range:" },
    { distance_range_size = Integer / "Attack Range:" },
    { known_spell_slot1 = Integer / "Spell Slot 1:" },
    { known_spell_slot2 = Integer / "Spell Slot 2:" },
    { known_spell_slot3 = Integer / "Spell Slot 3:" },
    { is_oversize = Integer / "Oversize:" },
    { magic_level = Integer / "Magic Level:" },
    { special_attack = Integer / "Special Attack:" },
    { special_attack_chance = Integer / "Special Atk Chance:" },
    { special_attack_duration = Integer / "Special Atk Duration:" },
    { boldness = Integer / "Boldness:" },
    { attack_speed = Integer / "Attack Speed:" },
});

impl EditableRecord for Monster {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!("[{}] {}", self.id, self.name)
    }
    fn detail_title() -> &'static str {
        "Monster Details"
    }
    fn empty_selection_text() -> &'static str {
        "No monster selected"
    }
    fn save_button_label() -> &'static str {
        "Save Monster"
    }
    fn detail_width() -> f32 {
        340.0
    }
}
