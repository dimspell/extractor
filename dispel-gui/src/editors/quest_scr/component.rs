use crate::components::editable::EditableRecord;
use dispel_core::Quest;

use crate::editable_record_fields;

editable_record_fields!(Quest, {
    { id = Integer / "ID:" },
    { type_id = Integer / "Type:" },
    { title = TextArea / "Title:" },
    { description = TextArea / "Description:" },
});

impl EditableRecord for Quest {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        let title = self.title.as_str();
        let label = if title.is_empty() || title == "null" {
            "???"
        } else {
            title
        };
        format!(
            "[{}] {}",
            self.id,
            &label.chars().take(40).collect::<String>()
        )
    }

    fn detail_title() -> &'static str {
        "Quest Details"
    }
    fn empty_selection_text() -> &'static str {
        "No quest selected"
    }
    fn save_button_label() -> &'static str {
        "Save Quests"
    }
    fn detail_width() -> f32 {
        340.0
    }
}
