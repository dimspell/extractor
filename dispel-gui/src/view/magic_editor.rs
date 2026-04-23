use crate::app::App;
use crate::message::{editor::magic::MagicEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_magic_editor_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.magic_editor,
            &self.state.magic_spreadsheet,
            Message::magic(MagicEditorMessage::LoadCatalog),
            Message::magic(MagicEditorMessage::Save),
            |idx| Message::magic(MagicEditorMessage::Select(idx)),
            |idx, field, val| Message::magic(MagicEditorMessage::FieldChanged(idx, field, val)),
            |msg| Message::magic(MagicEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |msg| Message::magic(MagicEditorMessage::PaneResized(msg)),
            |pane| Message::magic(MagicEditorMessage::PaneClicked(pane)),
        )
    }
}
