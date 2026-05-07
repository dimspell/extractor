use crate::app::App;
use crate::editors::hex_editor::HexEditorMessage;
use iced::Task;

pub fn handle(message: HexEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);
    let Some(editor) = app.state.hex_editors.get_mut(&tab_id) else {
        return Task::none();
    };

    match message {
        HexEditorMessage::SetBytesPerRow(n) => {
            if matches!(n, 8 | 16 | 32) {
                editor.bytes_per_row = n;
            }
        }
    }
    Task::none()
}
