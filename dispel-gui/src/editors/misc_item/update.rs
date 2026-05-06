use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::misc_item::MiscItemEditorMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(msg: MiscItemEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match msg {
        MiscItemEditorMessage::Spreadsheet(sm) => {
            handle_spreadsheet_messages!(
                app,
                misc_item_spreadsheet,
                misc_item_editor,
                |index, field, value| crate::message::Message::misc_item(
                    MiscItemEditorMessage::FieldChanged(index, field, value)
                ),
                sm
            );
            Task::none()
        }
        msg => crate::components::standard::update::handle(
            msg,
            &mut app.state.misc_item_editor,
            &mut app.state.misc_item_spreadsheet,
            &app.state.shared_game_path.clone(),
            "CharacterInGame/MiscItem.db",
            crate::message::Message::misc_item,
        ),
    }
}
