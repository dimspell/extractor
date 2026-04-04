use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_party_ini_tab(&self) -> Element<'_, Message> {
        build_editor_view(
            self,
            &self.state.party_ini_editor,
            Message::UpartyUiniOpScanItems,
            Message::UpartyUiniOpSave,
            Message::UpartyUiniOpSelectItem,
            Message::UpartyUiniOpFieldChanged,
            &self.state.lookups,
        )
    }
}
