use crate::app::App;
use crate::message::{editor::extraini::ExtraIniEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_extra_ini_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.extra_ini_editor,
            &self.state.extra_ini_spreadsheet,
            Message::extra_ini(ExtraIniEditorMessage::LoadCatalog),
            Message::extra_ini(ExtraIniEditorMessage::Save),
            |idx| Message::extra_ini(ExtraIniEditorMessage::SelectExtra(idx)),
            |idx, field, val| {
                Message::extra_ini(ExtraIniEditorMessage::FieldChanged(idx, field, val))
            },
            |msg| Message::extra_ini(ExtraIniEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |msg| Message::extra_ini(ExtraIniEditorMessage::PaneResized(msg)),
            |pane| Message::extra_ini(ExtraIniEditorMessage::PaneClicked(pane)),
        )
    }
}
