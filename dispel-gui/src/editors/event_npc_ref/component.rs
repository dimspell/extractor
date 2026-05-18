use crate::components::editable::EditableRecord;
use dispel_core::EventNpcRef;

use crate::editable_record_fields;

editable_record_fields!(EventNpcRef, {
    { id = Integer / "ID:" },
    { event_id = Integer / "Event ID:" },
    { name = String / "Name:" },
});

impl EditableRecord for EventNpcRef {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!("[{}] {} (Event: {})", self.id, self.name, self.event_id)
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
        280.0
    }
}
