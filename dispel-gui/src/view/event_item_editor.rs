use crate::app::App;
use crate::message::{editor::eventitem::EventItemEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_event_item_editor_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.event_item_editor,
            &self.state.event_item_spreadsheet,
            Message::event_item(EventItemEditorMessage::ScanItems),
            Message::event_item(EventItemEditorMessage::Save),
            |idx| Message::event_item(EventItemEditorMessage::SelectItem(idx)),
            |idx, field, val| {
                Message::event_item(EventItemEditorMessage::FieldChanged(idx, field, val))
            },
            |msg| Message::event_item(EventItemEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |msg| Message::event_item(EventItemEditorMessage::PaneResized(msg)),
            |pane| Message::event_item(EventItemEditorMessage::PaneClicked(pane)),
        )
    }
}
