use crate::components::editable::EditableRecord;
use dispel_core::{
    BooleanFlag, ExtraObjectType, ExtraRef, SmallRange0to3, Special9999Flag, SpecialPatternFlag,
    VisibilityType,
};

use crate::editable_record_fields;

editable_record_fields!(ExtraRef, {
    { id = Integer / "ID:" },
    { unknown1 = Integer / "Unknown 1:" },
    { ext_id = Integer / "Extra ID:" },
    { name = String / "Name:" },
    { object_type = Enum(ExtraObjectType, ["Chest", "Door", "Sign", "Altar", "Interactive", "Magic", "Unknown"]) / "Type:" },
    { x_pos = Integer / "X:" },
    { y_pos = Integer / "Y:" },
    { rotation = Integer / "Rotation:" },
    { unknown2 = HexString / "Unknown 2:" },
    { unknown3 = Integer / "Unknown 3:" },
    { closed = DispEnum(BooleanFlag, ["True", "False"]) / "Closed:" },
    { required_item_type_id = CompositeItem("items", required_item_id) / "Required Item:" },
    { unknown4 = Integer / "Unknown 4:" },
    { required_item_type_id2 = CompositeItem("items", required_item_id2) / "Required Item 2:" },
    { unknown5 = Integer / "Unknown 5:" },
    { unknown6 = DispEnum(Special9999Flag, ["0", "9999"]) / "Unknown 6:" },
    { unknown7 = DispEnum(Special9999Flag, ["0", "9999"]) / "Unknown 7:" },
    { unknown8 = DispEnum(Special9999Flag, ["0", "9999"]) / "Unknown 8:" },
    { unknown9 = DispEnum(Special9999Flag, ["0", "9999"]) / "Unknown 9:" },
    { gold_amount = Integer / "Gold:" },
    { item_type_id = CompositeItem("items", item_id) / "Item:" },
    { unknown10 = Integer / "Unknown 10:" },
    { item_count = Integer / "Item Count:" },
    { unknown11 = DispEnum(SpecialPatternFlag, ["0", "28", "84", "258", "9999"]) / "Unknown 11:" },
    { unknown12 = DispEnum(BooleanFlag, ["True", "False"]) / "Unknown 12:" },
    { unknown13 = DispEnum(Special9999Flag, ["0", "9999"]) / "Unknown 13:" },
    { unknown14 = HexString / "Unknown 14:" },
    { event_id = Integer / "Event ID:" },
    { message_id = Integer / "Message ID:" },
    { unknown15 = DispEnum(SmallRange0to3, ["0", "1", "2", "3"]) / "Unknown 15:" },
    { unknown16 = DispEnum(SmallRange0to3, ["0", "1", "2", "3"]) / "Unknown 16:" },
    { unknown17 = Integer / "Unknown 17:" },
    { interactive_element_type = Integer / "Interactive Type:" },
    { unknown18 = HexString / "Unknown 18:" },
    { is_quest_element = DispEnum(BooleanFlag, ["True", "False"]) / "Quest Element:" },
    { unknown20 = DispEnum(BooleanFlag, ["True", "False"]) / "Unknown 20:" },
    { unknown21 = DispEnum(BooleanFlag, ["True", "False"]) / "Unknown 21:" },
    { unknown22 = Integer / "Unknown 22:" },
    { unknown23 = DispEnum(BooleanFlag, ["True", "False"]) / "Unknown 23:" },
    { visibility = Enum(VisibilityType, ["Visible0", "Visible10"]) / "Visibility:" },
    { unknown24 = DispEnum(BooleanFlag, ["True", "False"]) / "Unknown 24:" },
    { unknown25 = Integer / "Unknown 25:" },
    { unknown26 = DispEnum(BooleanFlag, ["True", "False"]) / "Unknown 26:" },
    { unknown27 = DispEnum(BooleanFlag, ["True", "False"]) / "Unknown 27:" },
});

impl EditableRecord for ExtraRef {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!(
            "[{}] {} @ ({}, {})",
            self.id, self.name, self.x_pos, self.y_pos
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
