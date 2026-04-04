use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_message_scr_tab(&self) -> Element<'_, Message> {
        build_editor_view(
            self,
            &self.state.message_scr_editor,
            Message::UmessageUscrOpScanItems,
            Message::UmessageUscrOpSave,
            Message::UmessageUscrOpSelectItem,
            Message::UmessageUscrOpFieldChanged,
            &self.state.lookups,
        )
    }
}
