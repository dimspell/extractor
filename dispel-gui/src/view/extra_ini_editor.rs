use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_extra_ini_tab(&self) -> Element<'_, Message> {
        build_editor_view(
            self,
            &self.state.extra_ini_editor,
            Message::UextraUiniOpScanItems,
            Message::UextraUiniOpSave,
            Message::UextraUiniOpSelectItem,
            Message::UextraUiniOpFieldChanged,
            &self.state.lookups,
        )
    }
}
