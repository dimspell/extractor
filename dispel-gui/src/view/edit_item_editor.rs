use crate::app::App;
use crate::message::{editor::edititem::EditItemEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_edit_item_editor_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.edit_item_editor,
            &self.state.edit_item_spreadsheet,
            Message::edit_item(EditItemEditorMessage::ScanItems),
            Message::edit_item(EditItemEditorMessage::Save),
            |idx| Message::edit_item(EditItemEditorMessage::SelectItem(idx)),
            |idx, field, val| {
                Message::edit_item(EditItemEditorMessage::FieldChanged(idx, field, val))
            },
            |msg| Message::edit_item(EditItemEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |msg| Message::edit_item(EditItemEditorMessage::PaneResized(msg)),
            |pane| Message::edit_item(EditItemEditorMessage::PaneClicked(pane)),
        )
    }
}
