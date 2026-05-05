use crate::app::App;
use crate::message::{editor::chdata::ChDataEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_chdata_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.chdata_editor,
            &self.state.chdata_spreadsheet,
            Message::ch_data(ChDataEditorMessage::LoadCatalog),
            Message::ch_data(ChDataEditorMessage::Save),
            |idx| Message::ch_data(ChDataEditorMessage::Select(idx)),
            |idx, field, value| {
                Message::ch_data(ChDataEditorMessage::FieldChanged(idx, field, value))
            },
            |msg| Message::ch_data(ChDataEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |event| Message::ch_data(ChDataEditorMessage::PaneResized(event)),
            |pane| Message::ch_data(ChDataEditorMessage::PaneClicked(pane)),
        )
    }
}
