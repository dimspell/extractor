use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_event_npc_ref_tab(&self) -> Element<'_, Message> {
        build_editor_view(
            self,
            &self.state.event_npc_ref_editor,
            Message::EventNpcRefOpLoadCatalog,
            Message::EventNpcRefOpSave,
            Message::EventNpcRefOpSelectNpc,
            Message::EventNpcRefOpFieldChanged,
            &self.state.lookups,
        )
    }
}
