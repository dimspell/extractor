use crate::app::App;
use crate::message::{editor::allmapini::AllMapIniEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_all_map_ini_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.state.all_map_ini_editor;
        let spreadsheet = &self.state.all_map_ini_spreadsheet;

        view_spreadsheet(
            editor,
            spreadsheet,
            Message::all_map_ini(AllMapIniEditorMessage::LoadCatalog),
            Message::all_map_ini(AllMapIniEditorMessage::Save),
            |idx| Message::all_map_ini(AllMapIniEditorMessage::SelectMap(idx)),
            |idx, field, val| {
                Message::all_map_ini(AllMapIniEditorMessage::FieldChanged(idx, field, val))
            },
            |msg| Message::all_map_ini(AllMapIniEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |event| Message::all_map_ini(AllMapIniEditorMessage::PaneResized(event)),
            |pane| Message::all_map_ini(AllMapIniEditorMessage::PaneClicked(pane)),
        )
    }
}
