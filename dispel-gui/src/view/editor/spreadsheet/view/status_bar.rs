//! Bottom status bar — status message, loading indicator, inspector toggle,
//! save button.

use crate::components::editable::EditableRecord;
use crate::components::generic_editor::GenericEditorState;
use crate::message::Message;
use crate::style;
use crate::components::utils::horizontal_space;
use crate::view::editor::spreadsheet::message::SpreadsheetMessage;
use crate::view::editor::spreadsheet::state::SpreadsheetState;
use iced::widget::{button, column, container, progress_bar, row, text};
use iced::{Element, Fill, Length};

pub fn build_status_bar<'a, R: EditableRecord>(
    editor: &'a GenericEditorState<R>,
    spreadsheet: &'a SpreadsheetState,
    save_msg: Message,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> Element<'a, Message> {
    let loading: Element<Message> = if spreadsheet.is_loading {
        container(
            column![
                text("Loading spreadsheet...")
                    .size(11)
                    .style(style::subtle_text),
                container(progress_bar(0.0..=100.0, 50.0))
                    .width(Fill)
                    .height(Length::Fixed(6.0)),
            ]
            .spacing(4)
            .width(Length::Fixed(160.0)),
        )
        .into()
    } else {
        Element::from(text(""))
    };

    container(
        row![
            text(&editor.status_msg).size(13).style(style::subtle_text),
            horizontal_space(),
            loading,
            horizontal_space().width(20),
            button(text("Inspector").size(11))
                .on_press(spreadsheet_msg(SpreadsheetMessage::ToggleInspector))
                .style(style::browse_button),
            button(text(R::save_button_label()).size(11))
                .on_press(save_msg)
                .style(style::commit_button),
        ]
        .padding([8, 20])
        .spacing(4)
        .align_y(iced::Alignment::Center),
    )
    .width(Fill)
    .style(style::status_bar)
    .into()
}
