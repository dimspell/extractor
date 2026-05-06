use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::editors::message_scr::MessageScrEditorMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(msg: MessageScrEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match msg {
        MessageScrEditorMessage::Spreadsheet(sm) => {
            handle_spreadsheet_messages!(
                app,
                message_scr_spreadsheet,
                message_scr_editor,
                |index, field, value| crate::message::Message::message_scr(
                    MessageScrEditorMessage::FieldChanged(index, field, value)
                ),
                sm
            );
            Task::none()
        }
        msg => crate::components::standard::update::handle(
            msg,
            &mut app.state.message_scr_editor,
            &mut app.state.message_scr_spreadsheet,
            &app.state.shared_game_path.clone(),
            "ExtraInGame/Message.scr",
            crate::message::Message::message_scr,
        ),
    }
}
