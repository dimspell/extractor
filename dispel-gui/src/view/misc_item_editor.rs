use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_misc_item_editor_tab(&self) -> Element<'_, Message> {
        build_editor_view(
            self,
            &self.state.misc_item_editor,
            Message::MiscItemOpBrowseGamePath,
            Message::MiscItemOpSave,
            Message::MiscItemOpSelectItem,
            Message::MiscItemOpFieldChanged,
            &self.state.lookups,
        )
    }
}
