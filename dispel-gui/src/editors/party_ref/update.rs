use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::party_ref::PartyRefEditorMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(msg: PartyRefEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match msg {
        PartyRefEditorMessage::Spreadsheet(sm) => {
            handle_spreadsheet_messages!(
                app,
                party_ref_spreadsheet,
                party_ref_editor,
                |index, field, value| crate::message::Message::party_ref(
                    PartyRefEditorMessage::FieldChanged(index, field, value)
                ),
                sm
            );
            Task::none()
        }
        msg => crate::components::standard::update::handle(
            msg,
            &mut app.state.party_ref_editor,
            &mut app.state.party_ref_spreadsheet,
            &app.state.shared_game_path.clone(),
            "Ref/PartyRef.ref",
            crate::message::Message::party_ref,
        ),
    }
}
