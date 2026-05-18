use crate::components::editable::EditableRecord;
use dispel_core::{Event, EventType};

use crate::editable_record_fields;

editable_record_fields!(Event, {
    { event_id = Integer / "Event ID:" },
    { required_event_id = Integer / "Required Event ID:" },
    { event_type = i32Enum(EventType, ["Unknown", "Conditional", "ContinueOnUnsatisfied", "ExecuteOnSatisfied"]) / "Event Type:" },
    { event_filename = OptStr / "Script Filename:" },
    { counter = Integer / "Counter:" },
});

impl EditableRecord for Event {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!(
            "[{}] Type: {:?} (prev: {})",
            self.event_id, self.event_type, self.required_event_id
        )
    }

    fn detail_title() -> &'static str {
        "Event Details"
    }
    fn empty_selection_text() -> &'static str {
        "No event selected"
    }
    fn save_button_label() -> &'static str {
        "Save Events"
    }
    fn detail_width() -> f32 {
        320.0
    }
}
