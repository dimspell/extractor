use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_event_item_editor_tab(&self) -> Element<'_, Message> {
        build_editor_view(
            self,
            &self.state.event_item_editor,
            Message::EventItemOpBrowseGamePath,
            Message::EventItemOpSave,
            Message::EventItemOpSelectItem,
            Message::EventItemOpFieldChanged,
            &self.state.lookups,
        )
    }
}
