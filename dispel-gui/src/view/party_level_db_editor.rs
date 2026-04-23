use crate::app::App;
use crate::message::editor::partyleveldb::PartyLevelDbEditorMessage;
use crate::message::{Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_party_level_db_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.party_level_db_editor,
            &self.state.party_level_db_spreadsheet,
            Message::party_level_db(PartyLevelDbEditorMessage::LoadCatalog),
            Message::party_level_db(PartyLevelDbEditorMessage::Save),
            |idx| Message::party_level_db(PartyLevelDbEditorMessage::Select(idx)),
            |idx, field, val| {
                Message::party_level_db(PartyLevelDbEditorMessage::FieldChanged(idx, field, val))
            },
            |msg| Message::party_level_db(PartyLevelDbEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |msg| Message::party_level_db(PartyLevelDbEditorMessage::PaneResized(msg)),
            |pane| Message::party_level_db(PartyLevelDbEditorMessage::PaneClicked(pane)),
        )
    }
}
