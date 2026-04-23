use crate::app::App;
use crate::message::{editor::eventnpcref::EventNpcRefEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_event_npc_ref_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.event_npc_ref_editor,
            &self.state.event_npc_ref_spreadsheet,
            Message::event_npc_ref(EventNpcRefEditorMessage::LoadCatalog),
            Message::event_npc_ref(EventNpcRefEditorMessage::Save),
            |idx| Message::event_npc_ref(EventNpcRefEditorMessage::Select(idx)),
            |idx, field, val| {
                Message::event_npc_ref(EventNpcRefEditorMessage::FieldChanged(idx, field, val))
            },
            |msg| Message::event_npc_ref(EventNpcRefEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |msg| Message::event_npc_ref(EventNpcRefEditorMessage::PaneResized(msg)),
            |pane| Message::event_npc_ref(EventNpcRefEditorMessage::PaneClicked(pane)),
        )
    }
}
