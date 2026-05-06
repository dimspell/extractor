use crate::app::App;
use crate::editors::wave_ini::WaveIniEditorMessage;
use crate::message::{Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

pub fn view(app: &App) -> Element<'_, Message> {
    view_spreadsheet(
        &app.state.wave_ini_editor,
        &app.state.wave_ini_editor.spreadsheet,
        Message::wave_ini(WaveIniEditorMessage::LoadCatalog),
        Message::wave_ini(WaveIniEditorMessage::Save),
        |idx| Message::wave_ini(WaveIniEditorMessage::Select(idx)),
        |idx, field, value| {
            Message::wave_ini(WaveIniEditorMessage::FieldChanged(idx, field, value))
        },
        |msg| Message::wave_ini(WaveIniEditorMessage::Spreadsheet(msg)),
        &app.state.lookups,
        |event| Message::wave_ini(WaveIniEditorMessage::PaneResized(event)),
        |pane| Message::wave_ini(WaveIniEditorMessage::PaneClicked(pane)),
    )
}
