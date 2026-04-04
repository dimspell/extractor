use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_multi_file_editor_view;
use iced::Element;

impl App {
    pub fn view_npc_ref_tab(&self) -> Element<'_, Message> {
        build_multi_file_editor_view(
            self,
            &self.state.npc_ref_editor,
            Message::NpcRefOpScanFiles,
            Message::NpcRefOpSelectMapFile,
            Message::NpcRefOpSave,
            Message::NpcRefOpAddEntry,
            Message::NpcRefOpRemoveEntry,
            Message::NpcRefOpSelectNpc,
            Message::NpcRefOpFieldChanged,
            &self.state.lookups,
        )
    }
}
