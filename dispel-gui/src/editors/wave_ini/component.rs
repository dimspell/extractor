use crate::components::editable::EditableRecord;
use dispel_core::WaveIni;

use crate::editable_record_fields;

editable_record_fields!(WaveIni, {
    { id = Integer / "ID:" },
    { snf_filename = OptStr / "SNF Filename:" },
    { unknown_flag = OptStr / "Flag:" },
});

impl EditableRecord for WaveIni {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!(
            "[{}] {} - {}",
            self.id,
            self.snf_filename.as_deref().unwrap_or("null"),
            self.unknown_flag.as_deref().unwrap_or("null")
        )
    }

    fn detail_title() -> &'static str {
        "Wave Details"
    }
    fn empty_selection_text() -> &'static str {
        "No wave selected"
    }
    fn save_button_label() -> &'static str {
        "Save Waves"
    }
}
