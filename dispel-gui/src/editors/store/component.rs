use crate::components::editable::EditableRecord;
use dispel_core::Store;

use crate::editable_record_fields;

editable_record_fields!(Store, {
    { index = Integer / "Index:" },
    { store_name = String / "Store Name:" },
    { inn_night_cost = Integer / "Inn Cost:" },
    { price_modifier = Integer / "Price Modifier (%):" },
    { invitation = String / "Invitation:" },
    { haggle_success = String / "Haggle Success:" },
    { haggle_fail = String / "Haggle Fail:" },
});

impl EditableRecord for Store {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!("[{}] {}", self.index, self.store_name)
    }

    fn detail_title() -> &'static str {
        "Store Details"
    }
    fn empty_selection_text() -> &'static str {
        "No store selected"
    }
    fn save_button_label() -> &'static str {
        "Save Store"
    }
    fn detail_width() -> f32 {
        340.0
    }
}
