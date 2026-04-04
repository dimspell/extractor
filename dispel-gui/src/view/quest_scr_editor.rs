use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_quest_scr_tab(&self) -> Element<'_, Message> {
        build_editor_view(
            self,
            &self.state.quest_scr_editor,
            Message::UquestUscrOpScanItems,
            Message::UquestUscrOpSave,
            Message::UquestUscrOpSelectItem,
            Message::UquestUscrOpFieldChanged,
            &self.state.lookups,
        )
    }
}
