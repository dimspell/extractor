use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::editors::quest_scr::QuestScrEditorMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(msg: QuestScrEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match msg {
        QuestScrEditorMessage::Spreadsheet(sm) => {
            handle_spreadsheet_messages!(
                app,
                quest_scr_spreadsheet,
                quest_scr_editor,
                |index, field, value| crate::message::Message::quest_scr(
                    QuestScrEditorMessage::FieldChanged(index, field, value)
                ),
                sm
            );
            Task::none()
        }
        msg => crate::components::standard::update::handle(
            msg,
            &mut app.state.quest_scr_editor,
            &mut app.state.quest_scr_spreadsheet,
            &app.state.shared_game_path.clone(),
            "ExtraInGame/Quest.scr",
            crate::message::Message::quest_scr,
        ),
    }
}
