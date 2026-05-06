use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::editors::event_ini::EventIniEditorMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(msg: EventIniEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match msg {
        EventIniEditorMessage::Spreadsheet(sm) => {
            handle_spreadsheet_messages!(
                app,
                event_ini_spreadsheet,
                event_ini_editor,
                |index, field, value| crate::message::Message::event_ini(
                    EventIniEditorMessage::FieldChanged(index, field, value)
                ),
                sm
            );
            Task::none()
        }
        msg => crate::components::standard::update::handle(
            msg,
            &mut app.state.event_ini_editor,
            &mut app.state.event_ini_spreadsheet,
            &app.state.shared_game_path.clone(),
            "Event.ini",
            crate::message::Message::event_ini,
        ),
    }
}
