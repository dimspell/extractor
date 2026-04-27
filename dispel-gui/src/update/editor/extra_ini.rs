use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::extra_ini::ExtraIniEditorMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(msg: ExtraIniEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match msg {
        ExtraIniEditorMessage::Spreadsheet(sm) => {
            handle_spreadsheet_messages!(
                app,
                extra_ini_spreadsheet,
                extra_ini_editor,
                |index, field, value| crate::message::Message::extra_ini(
                    ExtraIniEditorMessage::FieldChanged(index, field, value)
                ),
                sm
            );
            Task::none()
        }
        msg => super::standard::handle(
            msg,
            &mut app.state.extra_ini_editor,
            &mut app.state.extra_ini_spreadsheet,
            &app.state.shared_game_path.clone(),
            "Extra.ini",
            crate::message::Message::extra_ini,
        ),
    }
}
