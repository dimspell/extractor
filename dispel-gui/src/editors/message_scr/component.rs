use crate::components::editable::EditableRecord;
use dispel_core::Message as ScrMessage;

use crate::editable_record_fields;

editable_record_fields!(ScrMessage, {
    { id = Integer / "ID:" },
    { line1 = OptStr / "Line 1:" },
    { line2 = OptStr / "Line 2:" },
    { line3 = OptStr / "Line 3:" },
});

impl EditableRecord for ScrMessage {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        let text = self.line1.as_deref().unwrap_or("");
        format!(
            "[{}] {}",
            self.id,
            text.chars().take(40).collect::<String>()
        )
    }

    fn detail_title() -> &'static str {
        "Message Details"
    }
    fn empty_selection_text() -> &'static str {
        "No message selected"
    }
    fn save_button_label() -> &'static str {
        "Save Messages"
    }
    fn detail_width() -> f32 {
        320.0
    }
}
