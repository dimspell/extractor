use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_party_ref_tab(&self) -> Element<'_, Message> {
        build_editor_view(
            self,
            &self.state.party_ref_editor,
            Message::PartyRefOpLoadCatalog,
            Message::PartyRefOpSave,
            Message::PartyRefOpSelectMember,
            Message::PartyRefOpFieldChanged,
            &self.state.lookups,
        )
    }
}
