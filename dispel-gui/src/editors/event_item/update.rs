use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::editors::event_item::EventItemEditorMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(msg: EventItemEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match msg {
        EventItemEditorMessage::Spreadsheet(sm) => {
            handle_spreadsheet_messages!(
                app,
                event_item_spreadsheet,
                event_item_editor,
                |index, field, value| crate::message::Message::event_item(
                    EventItemEditorMessage::FieldChanged(index, field, value)
                ),
                sm
            );
            Task::none()
        }
        msg => crate::components::standard::update::handle(
            msg,
            &mut app.state.event_item_editor,
            &mut app.state.event_item_spreadsheet,
            &app.state.shared_game_path.clone(),
            "CharacterInGame/EventItem.db",
            crate::message::Message::event_item,
        ),
    }
}
