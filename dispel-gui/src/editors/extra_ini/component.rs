use crate::components::editable::EditableRecord;
use dispel_core::Extra;

use crate::editable_record_fields;

editable_record_fields!(Extra, {
    { id = Integer / "ID:" },
    { sprite_filename = OptStr / "Sprite:" },
    { unknown = Integer / "Unknown:" },
    { description = OptStr / "Description:" },
});

impl EditableRecord for Extra {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!(
            "[{}] {}",
            self.id,
            self.sprite_filename.as_deref().unwrap_or("???")
        )
    }

    fn detail_title() -> &'static str {
        "Extra Object Details"
    }
    fn empty_selection_text() -> &'static str {
        "No extra object selected"
    }
    fn save_button_label() -> &'static str {
        "Save Extras"
    }
    fn detail_width() -> f32 {
        280.0
    }
}
