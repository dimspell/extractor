use crate::app::App;
use crate::message::{editor::mapini::MapIniEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_map_ini_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.map_ini_editor,
            &self.state.map_ini_spreadsheet,
            Message::map_ini(MapIniEditorMessage::LoadCatalog),
            Message::map_ini(MapIniEditorMessage::Save),
            |idx| Message::map_ini(MapIniEditorMessage::Select(idx)),
            |idx, field, val| Message::map_ini(MapIniEditorMessage::FieldChanged(idx, field, val)),
            |msg| Message::map_ini(MapIniEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |msg| Message::map_ini(MapIniEditorMessage::PaneResized(msg)),
            |pane| Message::map_ini(MapIniEditorMessage::PaneClicked(pane)),
        )
    }
}
