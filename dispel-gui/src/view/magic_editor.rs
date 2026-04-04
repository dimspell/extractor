use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::view::editor::view_spreadsheet;
use crate::view::generic_editor::build_editor_view;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Fill, Length};

impl App {
    pub fn view_magic_editor_tab(&self) -> Element<'_, Message> {
        if self.state.magic_spreadsheet.active {
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
        let list_view = build_editor_view(
            self,
            &self.state.magic_editor,
            Message::MagicOpScanSpells,
            Message::MagicOpSave,
            Message::MagicOpSelectSpell,
            Message::MagicOpFieldChanged,
            &self.state.lookups,
        );
        container(
            column![
                row![button(text("Spreadsheet").size(11))
                    .on_press(Message::MagicSpreadsheet(
                        crate::view::editor::SpreadsheetMessage::ToggleActive,
                    ))
                    .padding([4, 8])
                    .style(style::browse_button),]
                .padding([8, 16])
                .spacing(8),
                list_view,
            ]
            .spacing(0),
        )
        .width(Fill)
        .height(Fill)
        .into()
    }
}
