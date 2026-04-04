use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_wave_ini_tab(&self) -> Element<'_, Message> {
        build_editor_view(
            self,
            &self.state.wave_ini_editor,
            Message::WaveIniOpLoadCatalog,
            Message::WaveIniOpSave,
            Message::WaveIniOpSelectWave,
            Message::WaveIniOpFieldChanged,
            &self.state.lookups,
        )
    }
}
