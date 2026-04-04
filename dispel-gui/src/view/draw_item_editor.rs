use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_draw_item_tab(&self) -> Element<'_, Message> {
        build_editor_view(
            self,
            &self.state.draw_item_editor,
            Message::UdrawUitemOpScanItems,
            Message::UdrawUitemOpSave,
            Message::UdrawUitemOpSelectItem,
            Message::UdrawUitemOpFieldChanged,
            &self.state.lookups,
        )
    }
}
