use crate::components::editable::EditableRecord;
use dispel_core::{DialogOwner, DialogType, DialogueScript};

crate::editable_record_fields!(DialogueScript, {
    { id = Integer / "ID:" },
    { required_event_id = OptInt / "Requires Event ID:" },
    { next_dialog_to_check = OptInt / "Next Dialog:" },
    { dialog_type = Opti32Enum(DialogType, ["Normal", "Choice"]) / "Type:" },
    { dialog_owner = Opti32Enum(DialogOwner, ["Player", "NPC"]) / "Owner:" },
    { dialog_id = OptInt / "Dialog ID:" },
    { next_dialog_id1 = OptInt / "Next conversation option 1:" },
    { next_dialog_id2 = OptInt / "Next conversation option 2:" },
    { next_dialog_id3 = OptInt / "Next conversation option 3:" },
    { triggered_event_id = OptInt / "Triggers Event ID:" },
});

impl EditableRecord for DialogueScript {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!(
            "[{}] {} -> {}",
            self.id,
            self.dialog_type
                .map(|t| match t {
                    DialogType::Normal => "N",
                    DialogType::Choice => "C",
                })
                .unwrap_or("?"),
            self.dialog_owner
                .map(|o| match o {
                    DialogOwner::Player => "Player",
                    DialogOwner::Npc => "NPC",
                })
                .unwrap_or("?")
        )
    }
    fn detail_title() -> &'static str {
        "Dialog Details"
    }
    fn empty_selection_text() -> &'static str {
        "No dialog selected"
    }
    fn save_button_label() -> &'static str {
        "Save Dialog"
    }
    fn detail_width() -> f32 {
        340.0
    }
}
