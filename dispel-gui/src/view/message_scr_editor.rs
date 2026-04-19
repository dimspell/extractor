use crate::app::App;
use crate::message::{editor::messagescr::MessageScrEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_message_scr_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.message_scr_editor,
            &self.state.message_scr_spreadsheet,
            Message::message_scr(MessageScrEditorMessage::LoadCatalog),
            Message::message_scr(MessageScrEditorMessage::Save),
            |idx| Message::message_scr(MessageScrEditorMessage::SelectMessage(idx)),
            |idx, field, val| {
                Message::message_scr(MessageScrEditorMessage::FieldChanged(idx, field, val))
            },
            |msg| Message::message_scr(MessageScrEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |msg| Message::message_scr(MessageScrEditorMessage::PaneResized(msg)),
            |pane| Message::message_scr(MessageScrEditorMessage::PaneClicked(pane)),
        )
    }
}
