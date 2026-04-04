use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_event_ini_tab(&self) -> Element<'_, Message> {
        build_editor_view(
            self,
            &self.state.event_ini_editor,
            Message::UeventUiniOpScanItems,
            Message::UeventUiniOpSave,
            Message::UeventUiniOpSelectItem,
            Message::UeventUiniOpFieldChanged,
            &self.state.lookups,
        )
    }
}
