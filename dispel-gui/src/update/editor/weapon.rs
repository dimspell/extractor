use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::weapon::WeaponEditorMessage;
use crate::message::{Message, MessageExt};
use iced::Task;

pub fn handle(msg: WeaponEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match msg {
        WeaponEditorMessage::Spreadsheet(sm) => {
            handle_spreadsheet_messages!(
                app,
                weapon_spreadsheet,
                weapon_editor,
                |index, field, value| Message::weapon(WeaponEditorMessage::FieldChanged(
                    index, field, value
                )),
                sm
            );
            Task::none()
        }
        msg => super::standard::handle(
            msg,
            &mut app.state.weapon_editor,
            &mut app.state.weapon_spreadsheet,
            &app.state.shared_game_path.clone(),
            "CharacterInGame/weaponItem.db",
            crate::message::Message::weapon,
        ),
    }
}
