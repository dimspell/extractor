use crate::app::App;
use crate::message::{editor::monster::MonsterEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_monster_editor_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.monster_editor,
            &self.state.monster_spreadsheet,
            Message::monster(MonsterEditorMessage::LoadCatalog),
            Message::monster(MonsterEditorMessage::Save),
            |idx| Message::monster(MonsterEditorMessage::Select(idx)),
            |idx, field, val| Message::monster(MonsterEditorMessage::FieldChanged(idx, field, val)),
            |msg| Message::monster(MonsterEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |msg| Message::monster(MonsterEditorMessage::PaneResized(msg)),
            |pane| Message::monster(MonsterEditorMessage::PaneClicked(pane)),
        )
    }
}
