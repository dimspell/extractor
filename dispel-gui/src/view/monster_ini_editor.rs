use crate::app::App;
use crate::message::{editor::monsterini::MonsterIniEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_monster_ini_editor_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.monster_ini_editor,
            &self.state.monster_ini_spreadsheet,
            Message::monster_ini(MonsterIniEditorMessage::ScanMonsters),
            Message::monster_ini(MonsterIniEditorMessage::Save),
            |idx| Message::monster_ini(MonsterIniEditorMessage::SelectMonster(idx)),
            |idx, field, val| {
                Message::monster_ini(MonsterIniEditorMessage::FieldChanged(idx, field, val))
            },
            |msg| Message::monster_ini(MonsterIniEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |msg| Message::monster_ini(MonsterIniEditorMessage::PaneResized(msg)),
            |pane| Message::monster_ini(MonsterIniEditorMessage::PaneClicked(pane)),
        )
    }
}
