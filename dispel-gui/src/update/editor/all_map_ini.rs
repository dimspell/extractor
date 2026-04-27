use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::all_map_ini::AllMapIniEditorMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(msg: AllMapIniEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match msg {
        AllMapIniEditorMessage::Spreadsheet(sm) => {
            handle_spreadsheet_messages!(
                app,
                all_map_ini_spreadsheet,
                all_map_ini_editor,
                |index, field, value| crate::message::Message::all_map_ini(
                    AllMapIniEditorMessage::FieldChanged(index, field, value)
                ),
                sm
            );
            Task::none()
        }
        msg => super::standard::handle(
            msg,
            &mut app.state.all_map_ini_editor,
            &mut app.state.all_map_ini_spreadsheet,
            &app.state.shared_game_path.clone(),
            "AllMap.ini",
            crate::message::Message::all_map_ini,
        ),
    }
}
