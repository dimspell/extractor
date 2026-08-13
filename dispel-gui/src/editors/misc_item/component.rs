use crate::components::editable::EditableRecord;
use dispel_core::MiscItem;

use crate::editable_record_fields;

editable_record_fields!(MiscItem, {
    { name = String / "Name:" },
    { description = TextArea / "Description:" },
    { base_price = Integer / "Base Price:" },
    { reserved_bytes = HexString / "Reserved bytes:" },
    { runtime_record_index_slot = Integer / "Runtime record index slot:" },
});

impl EditableRecord for MiscItem {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!("[{}] {} - {}g", self.id, self.name, self.base_price)
    }

    fn detail_title() -> &'static str {
        "Misc Item Details"
    }
    fn empty_selection_text() -> &'static str {
        "No misc item selected"
    }
    fn save_button_label() -> &'static str {
        "Save Misc Items"
    }
    fn detail_width() -> f32 {
        320.0
    }
}
