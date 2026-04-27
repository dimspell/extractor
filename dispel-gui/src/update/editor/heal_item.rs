use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::heal_item::HealItemEditorMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(msg: HealItemEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match msg {
        HealItemEditorMessage::Spreadsheet(sm) => {
            handle_spreadsheet_messages!(
                app,
                heal_item_spreadsheet,
                heal_item_editor,
                |index, field, value| crate::message::Message::heal_item(
                    HealItemEditorMessage::FieldChanged(index, field, value)
                ),
                sm
            );
            Task::none()
        }
        msg => super::standard::handle(
            msg,
            &mut app.state.heal_item_editor,
            &mut app.state.heal_item_spreadsheet,
            &app.state.shared_game_path.clone(),
            "CharacterInGame/healItem.db",
            crate::message::Message::heal_item,
        ),
    }
}
