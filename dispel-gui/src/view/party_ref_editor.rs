use crate::app::App;
use crate::message::editor::partyref::PartyRefEditorMessage;
use crate::message::{Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_party_ref_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.party_ref_editor,
            &self.state.party_ref_spreadsheet,
            Message::party_ref(PartyRefEditorMessage::LoadCatalog),
            Message::party_ref(PartyRefEditorMessage::Save),
            |idx| Message::party_ref(PartyRefEditorMessage::Select(idx)),
            |idx, field, val| {
                Message::party_ref(PartyRefEditorMessage::FieldChanged(idx, field, val))
            },
            |msg| Message::party_ref(PartyRefEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |msg| Message::party_ref(PartyRefEditorMessage::PaneResized(msg)),
            |pane| Message::party_ref(PartyRefEditorMessage::PaneClicked(pane)),
        )
    }
}
