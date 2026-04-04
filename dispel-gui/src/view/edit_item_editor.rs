use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_edit_item_editor_tab(&self) -> Element<'_, Message> {
        build_editor_view(
            self,
            &self.state.edit_item_editor,
            Message::EditItemOpBrowseGamePath,
            Message::EditItemOpSave,
            Message::EditItemOpSelectItem,
            Message::EditItemOpFieldChanged,
            &self.state.lookups,
        )
    }
}
