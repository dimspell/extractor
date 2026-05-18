use crate::components::editable::{EditableRecord, FieldKind};
use dispel_core::{
    ExtraObjectType, ExtraRef, ItemTypeId, SmallRange0to3, Special9999Flag, SpecialPatternFlag,
    VisibilityType,
};

use crate::editable_record_fields;

const ITEM_TYPES: FieldKind = FieldKind::Enum {
    variants: &["Weapon", "Healing", "Edit", "Event", "Misc"],
};

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
    { closed = Integer / "Closed:" },
    { required_item_id = Integer / "Required Item:" },
    { required_item_type_id = Enum(ItemTypeId, Shared(ITEM_TYPES)) / "Required Type:" },
    { unknown4 = Integer / "Unknown 4:" },
    { required_item_id2 = Integer / "Required Item 2:" },
    { required_item_type_id2 = Enum(ItemTypeId, Shared(ITEM_TYPES)) / "Required Type 2:" },
    { unknown5 = Integer / "Unknown 5:" },
    { unknown6 = DispEnum(Special9999Flag, ["0", "9999"]) / "Unknown 6:" },
    { unknown7 = DispEnum(Special9999Flag, ["0", "9999"]) / "Unknown 7:" },
    { unknown8 = DispEnum(Special9999Flag, ["0", "9999"]) / "Unknown 8:" },
    { unknown9 = DispEnum(Special9999Flag, ["0", "9999"]) / "Unknown 9:" },
    { gold_amount = Integer / "Gold:" },
    { item_id = Integer / "Item ID:" },
    { item_type_id = Enum(ItemTypeId, Shared(ITEM_TYPES)) / "Item Type:" },
    { unknown10 = Integer / "Unknown 10:" },
    { item_count = Integer / "Item Count:" },
    { unknown11 = DispEnum(SpecialPatternFlag, ["0", "28", "84", "258", "9999"]) / "Unknown 11:" },
    { unknown12 = Integer / "Unknown 12:" },
    { unknown13 = DispEnum(Special9999Flag, ["0", "9999"]) / "Unknown 13:" },
    { unknown14 = HexString / "Unknown 14:" },
    { event_id = Integer / "Event ID:" },
    { message_id = Integer / "Message ID:" },
    { unknown15 = DispEnum(SmallRange0to3, ["0", "1", "2", "3"]) / "Unknown 15:" },
    { unknown16 = DispEnum(SmallRange0to3, ["0", "1", "2", "3"]) / "Unknown 16:" },
    { unknown17 = Integer / "Unknown 17:" },
    { interactive_element_type = Integer / "Interactive Type:" },
    { unknown18 = HexString / "Unknown 18:" },
    { is_quest_element = Integer / "Quest Element:" },
    { unknown20 = Integer / "Unknown 20:" },
    { unknown21 = Integer / "Unknown 21:" },
    { unknown22 = Integer / "Unknown 22:" },
    { unknown23 = Integer / "Unknown 23:" },
    { visibility = Enum(VisibilityType, ["Visible0", "Visible10"]) / "Visibility:" },
    { unknown24 = Integer / "Unknown 24:" },
    { unknown25 = Integer / "Unknown 25:" },
    { unknown26 = Integer / "Unknown 26:" },
    { unknown27 = Integer / "Unknown 27:" },
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
