use crate::app::App;
use crate::message::{editor::drawitem::DrawItemEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_draw_item_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.draw_item_editor,
            &self.state.draw_item_spreadsheet,
            Message::draw_item(DrawItemEditorMessage::LoadCatalog),
            Message::draw_item(DrawItemEditorMessage::Save),
            |idx| Message::draw_item(DrawItemEditorMessage::Select(idx)),
            |idx, field, val| {
                Message::draw_item(DrawItemEditorMessage::FieldChanged(idx, field, val))
            },
            |msg| Message::draw_item(DrawItemEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |msg| Message::draw_item(DrawItemEditorMessage::PaneResized(msg)),
            |pane| Message::draw_item(DrawItemEditorMessage::PaneClicked(pane)),
        )
    }
}
