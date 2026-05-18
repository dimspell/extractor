use crate::components::editable::EditableRecord;
use dispel_core::DialogueParagraph;

use crate::editable_record_fields;

editable_record_fields!(DialogueParagraph, {
    { id = Integer / "ID:" },
    { text = TextArea / "Text:" },
    { comment = TextArea / "Comment:" },
    { param1 = Integer / "Param 1:" },
    { wave_ini_entry_id = Integer / "Wave ID:" },
});

impl EditableRecord for DialogueParagraph {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!("[{}] {}", self.id, self.text)
    }

    fn detail_title() -> &'static str {
        "Dialogue Text Details"
    }
    fn empty_selection_text() -> &'static str {
        "No dialogue text selected"
    }
    fn save_button_label() -> &'static str {
        "Save Dialogue Text"
    }
    fn detail_width() -> f32 {
        340.0
    }
}
