use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_magic_editor_tab(&self) -> Element<'_, Message> {
        build_editor_view(
            self,
            &self.state.magic_editor,
            Message::MagicOpScanSpells,
            Message::MagicOpSave,
            Message::MagicOpSelectSpell,
            Message::MagicOpFieldChanged,
            &self.state.lookups,
        )
    }
}
