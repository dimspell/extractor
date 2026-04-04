use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_chdata_tab(&self) -> Element<'_, Message> {
        build_editor_view(
            self,
            &self.state.chdata_editor,
            Message::UchdataOpScanItems,
            Message::UchdataOpSave,
            Message::UchdataOpSelectItem,
            Message::UchdataOpFieldChanged,
            &self.state.lookups,
        )
    }
}
