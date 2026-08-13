use crate::components::editable::EditableRecord;
use dispel_core::{ActivationEffectId, BooleanFlag, ExtraObjectType, ExtraRef, SmallRange0to3};

use crate::editable_record_fields;

editable_record_fields!(ExtraRef, {
    { record_index = Integer / "Record index:" },
    { map_object_id = Integer / "Map object ID:" },
    { extra_definition_id = Integer / "Extra definition ID:" },
    { object_name = String / "Name:" },
    { object_type = Enum(ExtraObjectType, ["Chest", "Door", "Sign", "Altar", "Interactive", "Magic", "Unknown"]) / "Type:" },
    { map_x = Integer / "X:" },
    { map_y = Integer / "Y:" },
    { direction = Integer / "Direction:" },
    { direction_padding = HexString / "Direction padding:" },
    { interaction_state = Integer / "Interaction state:" },
    { requires_key = DispEnum(BooleanFlag, ["True", "False"]) / "Requires key:" },
    { required_item = CompositeItem("items") / "Required Item:" },
    { requirement_range_1_padding = Integer / "Requirement padding:" },
    { required_item2 = CompositeItem("items") / "Required Item 2:" },
    // { unknown5 = Integer / "Unknown 5:" },
    { requirement_range_2_start = Integer / "Requirement range 2 start:" },
    { requirement_range_2_end = Integer / "Requirement range 2 end:" },
    { requirement_range_3_start = Integer / "Requirement range 3 start:" },
    { requirement_range_3_end = Integer / "Requirement range 3 end:" },
    { gold_amount = Integer / "Gold:" },
    { loot_item = CompositeItem("items") / "Loot item:" },
    { loot_item_padding = Integer / "Loot item padding:" },
    { loot_item_count = Integer / "Loot item count:" },
    { additional_loot_1 = Integer / "Additional loot 1:" },
    { additional_loot_1_count = Integer / "Additional loot 1 count:" },
    { additional_loot_2 = Integer / "Additional loot 2:" },
    { additional_loot_2_count_and_config = HexString / "Additional loot 2 count and config:" },
    { interaction_event_id = Integer / "Interaction event ID:" },
    { interaction_message_id = Integer / "Interaction message ID:" },
    { footprint_width = DispEnum(SmallRange0to3, ["0", "1", "2", "3"]) / "Footprint width:" },
    { footprint_height = DispEnum(SmallRange0to3, ["0", "1", "2", "3"]) / "Footprint height:" },
    { footprint_orientation = Integer / "Footprint orientation:" },
    { interaction_range = Integer / "Interaction range:" },
    { interaction_range_padding = HexString / "Interaction range padding:" },
    { is_quest_element = DispEnum(BooleanFlag, ["True", "False"]) / "Quest Element:" },
    { post_activation_tile_flag = DispEnum(BooleanFlag, ["True", "False"]) / "Post-activation tile flag:" },
    { post_activation_footprint_mode = DispEnum(BooleanFlag, ["True", "False"]) / "Post-activation footprint mode:" },
    { preserve_final_sprite_frame = Integer / "Preserve final sprite frame:" },
    { alternate_render_mode = DispEnum(BooleanFlag, ["True", "False"]) / "Alternate render mode:" },
    { activation_effect_id = Enum(ActivationEffectId, ["None", "Effect10"]) / "Activation effect:" },
    { unresolved_activation_effect_flag = DispEnum(BooleanFlag, ["True", "False"]) / "Unresolved activation-effect flag:" },
    { activation_effect_padding = Integer / "Activation effect padding:" },
    { active_overlay_enabled = DispEnum(BooleanFlag, ["True", "False"]) / "Active overlay enabled:" },
    { map_object_active = DispEnum(BooleanFlag, ["True", "False"]) / "Map object active:" },
});

impl EditableRecord for ExtraRef {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!(
            "[{}] {} @ ({}, {})",
            self.record_index, self.object_name, self.map_x, self.map_y
        )
    }

    fn detail_title() -> &'static str {
        "ExtraRef Details"
    }
    fn empty_selection_text() -> &'static str {
        "No extra ref selected"
    }
    fn save_button_label() -> &'static str {
        "Save ExtraRef"
    }
    fn detail_width() -> f32 {
        340.0
    }
}
