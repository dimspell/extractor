use crate::components::editable::EditableRecord;
use dispel_core::EventNpcRef;
use dispel_core::references::enums::{
    BooleanFlag, NpcInteractionMode, NpcLookingDirection, NpcMovementMode, NpcRoleResult,
};

use crate::editable_record_fields;

editable_record_fields!(EventNpcRef, {
    { id = Integer / "ID:" },
    { npc_ini_id = Integer / "NPC ID:" },
    { name = String / "Name:" },
    { role_description = String / "Role Description:" },
    { role_result = Enum(NpcRoleResult, ["NormalDialogue", "PartyMemberDialogue1", "PartyMemberDialogue2", "PartyMemberDialogue3", "PartyMemberDialogue4", "PartyMemberDialogue5", "PartyMemberDialogue6", "PartyMemberDialogue7", "PartyMemberDialogue8", "WeaponShop", "HealMiscShop", "EditItemShop", "Inn"]) / "Role Result:" },
    { show_on_event = Integer / "Show on Event:" },
    { movement_mode = Enum(NpcMovementMode, ["Static", "Waypoints", "RandomInActivationRect"]) / "Movement Mode:" },
    { waypoint1_filled = Enum(BooleanFlag, ["False", "True"]) / "Waypoint 1 Filled:" },
    { waypoint1_x = Integer / "Waypoint 1 X:" },
    { waypoint1_y = Integer / "Waypoint 1 Y:" },
    { waypoint1_wait_time = Integer / "Waypoint 1 Wait Time:" },
    { waypoint1_facing_direction = Enum(NpcLookingDirection, ["Up", "UpRight", "Right", "DownRight", "Down", "DownLeft", "UpLeft"]) / "Waypoint 1 Facing Direction:" },
    { waypoint1_reserved = Integer / "Waypoint 1 Reserved:" },
    { waypoint2_filled = Enum(BooleanFlag, ["False", "True"]) / "Waypoint 2 Filled:" },
    { waypoint2_x = Integer / "Waypoint 2 X:" },
    { waypoint2_y = Integer / "Waypoint 2 Y:" },
    { waypoint2_wait_time = Integer / "Waypoint 2 Wait Time:" },
    { waypoint2_facing_direction = Enum(NpcLookingDirection, ["Up", "UpRight", "Right", "DownRight", "Down", "DownLeft", "UpLeft"]) / "Waypoint 2 Facing Direction:" },
    { waypoint2_reserved = Integer / "Waypoint 2 Reserved:" },
    { waypoint3_filled = Enum(BooleanFlag, ["False", "True"]) / "Waypoint 3 Filled:" },
    { waypoint3_x = Integer / "Waypoint 3 X:" },
    { waypoint3_y = Integer / "Waypoint 3 Y:" },
    { waypoint3_wait_time = Integer / "Waypoint 3 Wait Time:" },
    { waypoint3_facing_direction = Enum(NpcLookingDirection, ["Up", "UpRight", "Right", "DownRight", "Down", "DownLeft", "UpLeft"]) / "Waypoint 3 Facing Direction:" },
    { waypoint3_reserved = Integer / "Waypoint 3 Reserved:" },
    { waypoint4_filled = Enum(BooleanFlag, ["False", "True"]) / "Waypoint 4 Filled:" },
    { waypoint4_x = Integer / "Waypoint 4 X:" },
    { waypoint4_y = Integer / "Waypoint 4 Y:" },
    { waypoint4_wait_time = Integer / "Waypoint 4 Wait Time:" },
    { waypoint4_facing_direction = Enum(NpcLookingDirection, ["Up", "UpRight", "Right", "DownRight", "Down", "DownLeft", "UpLeft"]) / "Waypoint 4 Facing Direction:" },
    { waypoint4_reserved = Integer / "Waypoint 4 Reserved:" },
    { activation_rect_x1 = Integer / "Activation Rect X1:" },
    { activation_rect_y1 = Integer / "Activation Rect Y1:" },
    { activation_rect_x2 = Integer / "Activation Rect X2:" },
    { activation_rect_y2 = Integer / "Activation Rect Y2:" },
    { interaction_mode = Enum(NpcInteractionMode, ["Default", "RandomResult", "ConfiguredThenRandom"]) / "Interaction Mode:" },
    { interaction_result = Integer / "Interaction Result:" },
    { interaction_range_offset = Integer / "Interaction Range Offset:" },
    { dialog_id = Integer / "Dialog ID:" },
    { dialogue_face_sprite_id = Integer / "Face Sprite ID:" },
});

impl EditableRecord for EventNpcRef {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!(
            "[{}] {} (NPC {})",
            self.id,
            self.name.chars().take(20).collect::<String>(),
            self.npc_ini_id
        )
    }

    fn detail_title() -> &'static str {
        "Event NPC Details"
    }
    fn empty_selection_text() -> &'static str {
        "No event NPC selected"
    }
    fn save_button_label() -> &'static str {
        "Save Event NPCs"
    }
    fn detail_width() -> f32 {
        340.0
    }
}
