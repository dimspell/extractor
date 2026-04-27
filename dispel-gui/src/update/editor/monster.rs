use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::monster_db::MonsterEditorMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(msg: MonsterEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match msg {
        MonsterEditorMessage::Spreadsheet(sm) => {
            handle_spreadsheet_messages!(
                app,
                monster_spreadsheet,
                monster_editor,
                |index, field, value| crate::message::Message::monster_db(
                    MonsterEditorMessage::FieldChanged(index, field, value)
                ),
                sm
            );
            Task::none()
        }
        msg => super::standard::handle(
            msg,
            &mut app.state.monster_editor,
            &mut app.state.monster_spreadsheet,
            &app.state.shared_game_path.clone(),
            "MonsterInGame/Monster.db",
            crate::message::Message::monster_db,
        ),
    }
}
