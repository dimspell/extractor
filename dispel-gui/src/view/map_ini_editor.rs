use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_map_ini_tab(&self) -> Element<'_, Message> {
        build_editor_view(
            self,
            &self.state.map_ini_editor,
            Message::MapIniOpLoadCatalog,
            Message::MapIniOpSave,
            Message::MapIniOpSelectMap,
            Message::MapIniOpFieldChanged,
            &self.state.lookups,
        )
    }
}
