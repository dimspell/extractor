use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::view::editor::view_spreadsheet;
use crate::view::generic_editor::build_editor_view;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Fill};

impl App {
    pub fn view_misc_item_editor_tab(&self) -> Element<'_, Message> {
        if self.state.misc_item_spreadsheet.active {
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
        let list_view = build_editor_view(
            self,
            &self.state.misc_item_editor,
            Message::MiscItemOpScanItems,
            Message::MiscItemOpSave,
            Message::MiscItemOpSelectItem,
            Message::MiscItemOpFieldChanged,
            &self.state.lookups,
        );
        container(
            column![
                row![button(text("Spreadsheet").size(11))
                    .on_press(Message::MiscItemSpreadsheet(
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
