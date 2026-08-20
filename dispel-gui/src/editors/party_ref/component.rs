use crate::components::editable::EditableRecord;
use dispel_core::PartyRef;

crate::editable_record_fields!(PartyRef, {
    { id = Integer / "ID:" },
    { full_name = OptStr / "Full Name:" },
    { job_name = OptStr / "Job:" },
    { root_map_id = Integer / "Root Map ID:" },
    { npc_id = Integer / "NPC ID:" },
    { dlg_when_not_in_party = Integer / "Dialog (not in party):" },
    { dlg_when_in_party = Integer / "Dialog (in party):" },
    { is_in_party = Boolean / "In Party:" },
});

impl EditableRecord for PartyRef {
    crate::editable_record_delegate!();

    fn list_label(&self) -> String {
        format!(
            "[{}] {} ({})",
            self.id,
            self.full_name.as_deref().unwrap_or("???"),
            self.job_name.as_deref().unwrap_or("???")
        )
    }
    fn detail_title() -> &'static str {
        "Party Member Details"
    }
    fn empty_selection_text() -> &'static str {
        "No party member selected"
    }
    fn save_button_label() -> &'static str {
        "Save Party Ref"
    }
    fn detail_width() -> f32 {
        340.0
    }
}
