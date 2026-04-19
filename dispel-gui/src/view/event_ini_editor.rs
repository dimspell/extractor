use crate::app::App;
use crate::message::{editor::eventini::EventIniEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_event_ini_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.event_ini_editor,
            &self.state.event_ini_spreadsheet,
            Message::event_ini(EventIniEditorMessage::LoadCatalog),
            Message::event_ini(EventIniEditorMessage::Save),
            |idx| Message::event_ini(EventIniEditorMessage::SelectEvent(idx)),
            |idx, field, val| {
                Message::event_ini(EventIniEditorMessage::FieldChanged(idx, field, val))
            },
            |msg| Message::event_ini(EventIniEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |msg| Message::event_ini(EventIniEditorMessage::PaneResized(msg)),
            |pane| Message::event_ini(EventIniEditorMessage::PaneClicked(pane)),
        )
    }
}
