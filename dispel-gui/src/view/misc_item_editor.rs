use crate::app::App;
use crate::message::Message;
use crate::view::editor::view_spreadsheet;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_misc_item_editor_tab(&self) -> Element<'_, Message> {
        if self.state.misc_item_spreadsheet.show_inspector
            || self.state.misc_item_spreadsheet.sort_column.is_some()
            || !self.state.misc_item_spreadsheet.filter_query.is_empty()
        {
            return view_spreadsheet(
                &self.state.misc_item_editor,
                &self.state.misc_item_spreadsheet,
                Message::MiscItemOpScanItems,
                Message::MiscItemOpSave,
                Message::MiscItemOpSelectItem,
                Message::MiscItemOpFieldChanged,
                Message::MiscItemSpreadsheet,
                &self.state.lookups,
            );
        }
        build_editor_view(
            self,
            &self.state.misc_item_editor,
            Message::MiscItemOpScanItems,
            Message::MiscItemOpSave,
            Message::MiscItemOpSelectItem,
            Message::MiscItemOpFieldChanged,
            &self.state.lookups,
        )
    }
}
