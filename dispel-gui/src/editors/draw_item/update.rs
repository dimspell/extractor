use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::draw_item::DrawItemEditorMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(msg: DrawItemEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match msg {
        DrawItemEditorMessage::Spreadsheet(sm) => {
            handle_spreadsheet_messages!(
                app,
                draw_item_spreadsheet,
                draw_item_editor,
                |index, field, value| crate::message::Message::draw_item(
                    DrawItemEditorMessage::FieldChanged(index, field, value)
                ),
                sm
            );
            Task::none()
        }
        msg => crate::components::standard::update::handle(
            msg,
            &mut app.state.draw_item_editor,
            &mut app.state.draw_item_spreadsheet,
            &app.state.shared_game_path.clone(),
            "Ref/DRAWITEM.ref",
            crate::message::Message::draw_item,
        ),
    }
}
