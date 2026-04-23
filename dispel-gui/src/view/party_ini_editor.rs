use crate::app::App;
use crate::message::{editor::partyini::PartyIniEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_party_ini_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.party_ini_editor,
            &self.state.party_ini_spreadsheet,
            Message::party_ini(PartyIniEditorMessage::LoadCatalog),
            Message::party_ini(PartyIniEditorMessage::Save),
            |idx| Message::party_ini(PartyIniEditorMessage::Select(idx)),
            |idx, field, val| {
                Message::party_ini(PartyIniEditorMessage::FieldChanged(idx, field, val))
            },
            |msg| Message::party_ini(PartyIniEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |msg| Message::party_ini(PartyIniEditorMessage::PaneResized(msg)),
            |pane| Message::party_ini(PartyIniEditorMessage::PaneClicked(pane)),
        )
    }
}
