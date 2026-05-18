use crate::components::editable::EditableRecord;
use dispel_core::Quest;

use crate::editable_record_fields;

editable_record_fields!(Quest, {
    { id = Integer / "ID:" },
    { type_id = Integer / "Type:" },
    { title = OptStr / "Title:" },
    { description = OptStr / "Description:" },
});

impl EditableRecord for Quest {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        let title = self.title.as_deref().unwrap_or("???");
        format!(
            "[{}] {}",
            self.id,
            &title.chars().take(40).collect::<String>()
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
