use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::edit_item::EditItemEditorMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(msg: EditItemEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match msg {
        EditItemEditorMessage::Spreadsheet(sm) => {
            handle_spreadsheet_messages!(
                app,
                edit_item_spreadsheet,
                edit_item_editor,
                |index, field, value| crate::message::Message::edit_item(
                    EditItemEditorMessage::FieldChanged(index, field, value)
                ),
                sm
            );
            Task::none()
        }
        msg => super::standard::handle(
            msg,
            &mut app.state.edit_item_editor,
            &mut app.state.edit_item_spreadsheet,
            &app.state.shared_game_path.clone(),
            "CharacterInGame/EditItem.db",
            crate::message::Message::edit_item,
        ),
    }
}
