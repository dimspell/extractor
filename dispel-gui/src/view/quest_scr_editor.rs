use crate::app::App;
use crate::message::{editor::questscr::QuestScrEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_quest_scr_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.quest_scr_editor,
            &self.state.quest_scr_spreadsheet,
            Message::quest_scr(QuestScrEditorMessage::LoadCatalog),
            Message::quest_scr(QuestScrEditorMessage::Save),
            |idx| Message::quest_scr(QuestScrEditorMessage::SelectQuest(idx)),
            |idx, field, val| {
                Message::quest_scr(QuestScrEditorMessage::FieldChanged(idx, field, val))
            },
            |msg| Message::quest_scr(QuestScrEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |msg| Message::quest_scr(QuestScrEditorMessage::PaneResized(msg)),
            |pane| Message::quest_scr(QuestScrEditorMessage::PaneClicked(pane)),
        )
    }
}
