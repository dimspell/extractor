use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::view::editor::view_spreadsheet;
use crate::view::generic_editor::build_editor_view;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Fill};

impl App {
    pub fn view_heal_item_editor_tab(&self) -> Element<'_, Message> {
        if self.state.heal_item_spreadsheet.active {
            return view_spreadsheet(
                &self.state.heal_item_editor,
                &self.state.heal_item_spreadsheet,
                Message::HealItemOpScanItems,
                Message::HealItemOpSave,
                Message::HealItemOpSelectItem,
                Message::HealItemOpFieldChanged,
                Message::HealItemSpreadsheet,
                &self.state.lookups,
            );
        }
        let list_view = build_editor_view(
            self,
            &self.state.heal_item_editor,
            Message::HealItemOpScanItems,
            Message::HealItemOpSave,
            Message::HealItemOpSelectItem,
            Message::HealItemOpFieldChanged,
            &self.state.lookups,
        );
        container(
            column![
                row![button(text("Spreadsheet").size(11))
                    .on_press(Message::HealItemSpreadsheet(
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
