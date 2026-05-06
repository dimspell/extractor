use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::editors::monster_ini::MonsterIniEditorMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(msg: MonsterIniEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match msg {
        MonsterIniEditorMessage::Spreadsheet(sm) => {
            handle_spreadsheet_messages!(
                app,
                monster_ini_spreadsheet,
                monster_ini_editor,
                |index, field, value| crate::message::Message::monster_ini(
                    MonsterIniEditorMessage::FieldChanged(index, field, value)
                ),
                sm
            );
            Task::none()
        }
        msg => crate::components::standard::update::handle(
            msg,
            &mut app.state.monster_ini_editor,
            &mut app.state.monster_ini_spreadsheet,
            &app.state.shared_game_path.clone(),
            "Monster.ini",
            crate::message::Message::monster_ini,
        ),
    }
}
