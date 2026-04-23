use crate::app::App;
use crate::message::{editor::monster_db::MonsterEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_monster_editor_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.monster_editor,
            &self.state.monster_spreadsheet,
            Message::monster_db(MonsterEditorMessage::LoadCatalog),
            Message::monster_db(MonsterEditorMessage::Save),
            |idx| Message::monster_db(MonsterEditorMessage::Select(idx)),
            |idx, field, val| {
                Message::monster_db(MonsterEditorMessage::FieldChanged(idx, field, val))
            },
            |msg| Message::monster_db(MonsterEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |msg| Message::monster_db(MonsterEditorMessage::PaneResized(msg)),
            |pane| Message::monster_db(MonsterEditorMessage::PaneClicked(pane)),
        )
    }
}
