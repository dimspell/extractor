use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::event_npc_ref::EventNpcRefEditorMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(msg: EventNpcRefEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match msg {
        EventNpcRefEditorMessage::Spreadsheet(sm) => {
            handle_spreadsheet_messages!(
                app,
                event_npc_ref_spreadsheet,
                event_npc_ref_editor,
                |index, field, value| crate::message::Message::event_npc_ref(
                    EventNpcRefEditorMessage::FieldChanged(index, field, value)
                ),
                sm
            );
            Task::none()
        }
        msg => crate::components::standard::update::handle(
            msg,
            &mut app.state.event_npc_ref_editor,
            &mut app.state.event_npc_ref_spreadsheet,
            &app.state.shared_game_path.clone(),
            "NpcInGame/Eventnpc.ref",
            crate::message::Message::event_npc_ref,
        ),
    }
}
