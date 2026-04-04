use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_multi_file_editor_view;
use iced::Element;

impl App {
    pub fn view_monster_ref_editor_tab(&self) -> Element<'_, Message> {
        build_multi_file_editor_view(
            self,
            &self.monster_ref_editor,
            Message::MonsterRefOpScanFiles,
            Message::MonsterRefOpSelectFile,
            Message::MonsterRefOpSave,
            Message::MonsterRefOpAddEntry,
            Message::MonsterRefOpRemoveEntry,
            Message::MonsterRefOpSelectEntry,
            Message::MonsterRefOpFieldChanged,
            &self.lookups,
        )
    }
}
