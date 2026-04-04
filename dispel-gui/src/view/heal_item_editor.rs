use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_heal_item_editor_tab(&self) -> Element<'_, Message> {
        build_editor_view(
            self,
            &self.state.heal_item_editor,
            Message::HealItemOpBrowseGamePath,
            Message::HealItemOpSave,
            Message::HealItemOpSelectItem,
            Message::HealItemOpFieldChanged,
            &self.state.lookups,
        )
    }
}
