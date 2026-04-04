use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_party_level_db_tab(&self) -> Element<'_, Message> {
        build_editor_view(
            self,
            &self.state.party_level_db_editor,
            Message::PartyLevelDbOpLoadCatalog,
            Message::PartyLevelDbOpSave,
            Message::PartyLevelDbOpSelectRecord,
            Message::PartyLevelDbOpFieldChanged,
            &self.state.lookups,
        )
    }
}
