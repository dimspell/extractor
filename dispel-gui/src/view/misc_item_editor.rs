use crate::app::App;
use crate::message::{editor::miscitem::MiscItemEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_misc_item_editor_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.misc_item_editor,
            &self.state.misc_item_spreadsheet,
            Message::misc_item(MiscItemEditorMessage::ScanItems),
            Message::misc_item(MiscItemEditorMessage::Save),
            |idx| Message::misc_item(MiscItemEditorMessage::SelectItem(idx)),
            |idx, field, val| {
                Message::misc_item(MiscItemEditorMessage::FieldChanged(idx, field, val))
            },
            |msg| Message::misc_item(MiscItemEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |msg| Message::misc_item(MiscItemEditorMessage::PaneResized(msg)),
            |pane| Message::misc_item(MiscItemEditorMessage::PaneClicked(pane)),
        )
    }
}
