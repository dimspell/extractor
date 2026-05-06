use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::editors::map_ini::MapIniEditorMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(msg: MapIniEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match msg {
        MapIniEditorMessage::Spreadsheet(sm) => {
            handle_spreadsheet_messages!(
                app,
                map_ini_spreadsheet,
                map_ini_editor,
                |index, field, value| crate::message::Message::map_ini(
                    MapIniEditorMessage::FieldChanged(index, field, value)
                ),
                sm
            );
            Task::none()
        }
        msg => crate::components::standard::update::handle(
            msg,
            &mut app.state.map_ini_editor,
            &mut app.state.map_ini_spreadsheet,
            &app.state.shared_game_path.clone(),
            "Ref/Map.ini",
            crate::message::Message::map_ini,
        ),
    }
}
