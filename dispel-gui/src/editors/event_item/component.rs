use crate::components::editable::EditableRecord;
use dispel_core::EventItem;

crate::editable_record_fields!(EventItem, {
    { name = String / "Name:" },
    { description = TextArea / "Description:" },
    { base_price = Integer / "Base Price:" },
    { padding = Integer / "Padding:" },
});

impl EditableRecord for EventItem {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!("[{}] {}", self.id, self.name)
    }
    fn detail_title() -> &'static str {
        "Event Item Details"
    }
    fn empty_selection_text() -> &'static str {
        "No event item selected"
    }
    fn save_button_label() -> &'static str {
        "Save Event Items"
    }
    fn detail_width() -> f32 {
        320.0
    }
}
