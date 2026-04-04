use crate::app::App;
use crate::message::Message;
use crate::view::editor::view_spreadsheet;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_magic_editor_tab(&self) -> Element<'_, Message> {
        if self.state.magic_spreadsheet.show_inspector
            || self.state.magic_spreadsheet.sort_column.is_some()
            || !self.state.magic_spreadsheet.filter_query.is_empty()
        {
            return view_spreadsheet(
                &self.state.magic_editor,
                &self.state.magic_spreadsheet,
                Message::MagicOpScanSpells,
                Message::MagicOpSave,
                Message::MagicOpSelectSpell,
                Message::MagicOpFieldChanged,
                Message::MagicSpreadsheet,
                &self.state.lookups,
            );
        }
        build_editor_view(
            self,
            &self.state.magic_editor,
            Message::MagicOpScanSpells,
            Message::MagicOpSave,
            Message::MagicOpSelectSpell,
            Message::MagicOpFieldChanged,
            &self.state.lookups,
        )
    }
}
