use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::editors::party_ini::PartyIniEditorMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(msg: PartyIniEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match msg {
        PartyIniEditorMessage::Spreadsheet(sm) => {
            handle_spreadsheet_messages!(
                app,
                party_ini_spreadsheet,
                party_ini_editor,
                |index, field, value| crate::message::Message::party_ini(
                    PartyIniEditorMessage::FieldChanged(index, field, value)
                ),
                sm
            );
            Task::none()
        }
        msg => crate::components::standard::update::handle(
            msg,
            &mut app.state.party_ini_editor,
            &mut app.state.party_ini_spreadsheet,
            &app.state.shared_game_path.clone(),
            "NpcInGame/PrtIni.db",
            crate::message::Message::party_ini,
        ),
    }
}
