use crate::app::App;
use crate::message::{editor::npc_ini::NpcIniEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_npc_ini_editor_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.npc_ini_editor,
            &self.state.npc_ini_spreadsheet,
            Message::npc_ini(NpcIniEditorMessage::LoadCatalog),
            Message::npc_ini(NpcIniEditorMessage::Save),
            |idx| Message::npc_ini(NpcIniEditorMessage::Select(idx)),
            |idx, field, val| Message::npc_ini(NpcIniEditorMessage::FieldChanged(idx, field, val)),
            |msg| Message::npc_ini(NpcIniEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |msg| Message::npc_ini(NpcIniEditorMessage::PaneResized(msg)),
            |pane| Message::npc_ini(NpcIniEditorMessage::PaneClicked(pane)),
        )
    }
}
