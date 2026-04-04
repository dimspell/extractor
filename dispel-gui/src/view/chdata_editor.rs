use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_chdata_tab(&self) -> Element<'_, Message> {
        build_editor_view(
            self,
            &self.state.chdata_editor,
            Message::ChDataOpLoadCatalog,
            Message::ChDataOpSave,
            Message::ChDataOpSelectData,
            Message::ChDataOpFieldChanged,
            &self.state.lookups,
        )
    }
}
