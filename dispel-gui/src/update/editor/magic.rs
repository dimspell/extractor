use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::magic::MagicEditorMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(msg: MagicEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match msg {
        MagicEditorMessage::Spreadsheet(sm) => {
            handle_spreadsheet_messages!(
                app,
                magic_spreadsheet,
                magic_editor,
                |index, field, value| crate::message::Message::magic(
                    MagicEditorMessage::FieldChanged(index, field, value)
                ),
                sm
            );
            Task::none()
        }
        msg => super::standard::handle(
            msg,
            &mut app.state.magic_editor,
            &mut app.state.magic_spreadsheet,
            &app.state.shared_game_path.clone(),
            "MagicInGame/Magic.db",
            crate::message::Message::magic,
        ),
    }
}
