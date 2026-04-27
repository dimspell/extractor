use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::npc_ini::NpcIniEditorMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(msg: NpcIniEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match msg {
        NpcIniEditorMessage::Spreadsheet(sm) => {
            handle_spreadsheet_messages!(
                app,
                npc_ini_spreadsheet,
                npc_ini_editor,
                |index, field, value| crate::message::Message::npc_ini(
                    NpcIniEditorMessage::FieldChanged(index, field, value)
                ),
                sm
            );
            Task::none()
        }
        msg => super::standard::handle(
            msg,
            &mut app.state.npc_ini_editor,
            &mut app.state.npc_ini_spreadsheet,
            &app.state.shared_game_path.clone(),
            "Npc.ini",
            crate::message::Message::npc_ini,
        ),
    }
}
