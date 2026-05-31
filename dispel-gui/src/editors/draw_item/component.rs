use crate::components::editable::EditableRecord;
use dispel_core::DrawItem;

use crate::editable_record_fields;

editable_record_fields!(DrawItem, {
    { map_id = Integer / "Map ID:" },
    { x_coord = Integer / "X:" },
    { y_coord = Integer / "Y:" },
    { item_type = CompositeItem("items", item_id) / "Item:" },
});

impl EditableRecord for DrawItem {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!(
            "[Map {}] ({}, {}) Item: {}",
            self.map_id, self.x_coord, self.y_coord, self.item_id
        )
    }

    fn detail_title() -> &'static str {
        "Draw Item Details"
    }
    fn empty_selection_text() -> &'static str {
        "No draw item selected"
    }
    fn save_button_label() -> &'static str {
        "Save Draw Items"
    }
    fn detail_width() -> f32 {
        280.0
    }
}
