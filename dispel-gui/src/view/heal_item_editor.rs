use crate::app::App;
use crate::message::{editor::heal_item::HealItemEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_heal_item_editor_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.heal_item_editor,
            &self.state.heal_item_spreadsheet,
            Message::heal_item(HealItemEditorMessage::LoadCatalog),
            Message::heal_item(HealItemEditorMessage::Save),
            |idx| Message::heal_item(HealItemEditorMessage::SelectItem(idx)),
            |idx, field, val| {
                Message::heal_item(HealItemEditorMessage::FieldChanged(idx, field, val))
            },
            |msg| Message::heal_item(HealItemEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |msg| Message::heal_item(HealItemEditorMessage::PaneResized(msg)),
            |pane| Message::heal_item(HealItemEditorMessage::PaneClicked(pane)),
        )
    }
}
