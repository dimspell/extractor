use crate::app::App;
use crate::message::{editor::wave_ini::WaveIniEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_wave_ini_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.wave_ini_editor,
            &self.state.wave_ini_spreadsheet,
            Message::wave_ini(WaveIniEditorMessage::LoadCatalog),
            Message::wave_ini(WaveIniEditorMessage::Save),
            |idx| Message::wave_ini(WaveIniEditorMessage::Select(idx)),
            |idx, field, value| {
                Message::wave_ini(WaveIniEditorMessage::FieldChanged(idx, field, value))
            },
            |msg| Message::wave_ini(WaveIniEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |event| Message::wave_ini(WaveIniEditorMessage::PaneResized(event)),
            |pane| Message::wave_ini(WaveIniEditorMessage::PaneClicked(pane)),
        )
    }
}
