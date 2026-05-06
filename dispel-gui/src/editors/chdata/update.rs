use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::chdata::ChDataEditorMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(msg: ChDataEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match msg {
        ChDataEditorMessage::Spreadsheet(sm) => {
            handle_spreadsheet_messages!(
                app,
                chdata_spreadsheet,
                chdata_editor,
                |index, field, value| crate::message::Message::ch_data(
                    ChDataEditorMessage::FieldChanged(index, field, value)
                ),
                sm
            );
            Task::none()
        }
        msg => crate::components::standard::update::handle(
            msg,
            &mut app.state.chdata_editor,
            &mut app.state.chdata_spreadsheet,
            &app.state.shared_game_path.clone(),
            "CharacterInGame/ChData.db",
            crate::message::Message::ch_data,
        ),
    }
}
