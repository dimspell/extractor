use crate::components::editable::EditableRecord;
use dispel_core::references::enums::{
    BooleanFlag, NpcLookingDirection, Unknown0110, Unknown012, Unknown0to7,
};
use dispel_core::NPC;

use crate::editable_record_fields;

editable_record_fields!(NPC, {
    { id = Integer / "ID:" },
    { npc_id = Integer / "NPC ID:" },
    { name = String / "Name:" },
    { description = String / "Description:" },
    { party_script_id = Integer / "Party Script ID:" },
    { show_on_event = Integer / "Show on Event:" },
    { unknown_1 = DispEnum(Unknown012, ["0", "1", "2"]) / "Unknown 1:" },
    { goto1_filled = Enum(BooleanFlag, ["False", "True"]) / "Waypoint 1 Filled:" },
    { goto2_filled = Enum(BooleanFlag, ["False", "True"]) / "Waypoint 2 Filled:" },
    { goto3_filled = Enum(BooleanFlag, ["False", "True"]) / "Waypoint 3 Filled:" },
    { goto4_filled = Enum(BooleanFlag, ["False", "True"]) / "Waypoint 4 Filled:" },
    { goto1_x = Integer / "Waypoint 1 X:" },
    { goto1_y = Integer / "Waypoint 1 Y:" },
    { goto2_x = Integer / "Waypoint 2 X:" },
    { goto2_y = Integer / "Waypoint 2 Y:" },
    { goto3_x = Integer / "Waypoint 3 X:" },
    { goto3_y = Integer / "Waypoint 3 Y:" },
    { goto4_x = Integer / "Waypoint 4 X:" },
    { goto4_y = Integer / "Waypoint 4 Y:" },
    { unknown_2 = Integer / "Unknown 2:" },
    { unknown_3 = Integer / "Unknown 3:" },
    { unknown_4 = Integer / "Unknown 4:" },
    { unknown_5 = Integer / "Unknown 5:" },
    { looking_direction = Enum(NpcLookingDirection, ["Up", "UpRight", "Right", "DownRight", "Down", "DownLeft", "UpLeft"]) / "Direction:" },
    { unknown_6 = DispEnum(Unknown0to7, ["0", "1", "2", "3", "4", "5", "6", "7"]) / "Unknown 6:" },
    { unknown_7 = DispEnum(Unknown0to7, ["0", "1", "2", "3", "4", "5", "6", "7"]) / "Unknown 7:" },
    { unknown_8 = DispEnum(Unknown0to7, ["0", "1", "2", "3", "4", "5", "6", "7"]) / "Unknown 8:" },
    { unknown_9 = Integer / "Unknown 9:" },
    { unknown_10 = Integer / "Unknown 10:" },
    { unknown_11 = Integer / "Unknown 11:" },
    { unknown_12 = Integer / "Unknown 12:" },
    { unknown_13 = Integer / "Unknown 13:" },
    { unknown_14 = Integer / "Unknown 14:" },
    { unknown_15 = Integer / "Unknown 15:" },
    { unknown_16 = Integer / "Unknown 16:" },
    { unknown_17 = DispEnum(Unknown012, ["0", "1", "2"]) / "Unknown 17:" },
    { unknown_18 = Integer / "Unknown 18:" },
    { unknown_item = CompositeItem("items") / "Unknown Item:" },
    { unknown_19 = DispEnum(Unknown0110, ["0", "1", "10"]) / "Unknown 19:" },
    { dialog_id = Integer / "Dialog ID:" },
    { dialogue_face_sprite_id = Integer / "Face Sprite ID:" },
});

impl EditableRecord for NPC {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!(
            "[{}] {} (NPC {})",
            self.id,
            &self.name.chars().take(20).collect::<String>(),
            self.npc_id
        )
    }

    fn detail_title() -> &'static str {
        "NPC Details"
    }
    fn empty_selection_text() -> &'static str {
        "No NPC selected"
    }
    fn save_button_label() -> &'static str {
        "Save NPCs"
    }
    fn detail_width() -> f32 {
        340.0
    }
}
