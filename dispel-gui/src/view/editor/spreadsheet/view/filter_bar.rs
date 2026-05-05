//! Top filter bar — mode toggle, query input, clear button, navigation,
//! row counter, CSV export, scan trigger.

use crate::components::editor::editable::EditableRecord;
use crate::generic_editor::GenericEditorState;
use crate::message::Message;
use crate::style;
use crate::utils::horizontal_space;
use crate::view::editor::spreadsheet::message::SpreadsheetMessage;
use crate::view::editor::spreadsheet::state::{GlobalFilterMode, SpreadsheetState};
use iced::widget::{button, row, text, text_input};
use iced::{Element, Length};

pub fn build_filter_bar<'a, R: EditableRecord>(
    editor: &'a GenericEditorState<R>,
    spreadsheet: &'a SpreadsheetState,
    scan_msg: Message,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> Element<'a, Message> {
    let total = editor.catalog.as_ref().map(|c| c.len()).unwrap_or(0);
    let visible = spreadsheet.filtered_indices.len();
    let highlight_count = spreadsheet.highlighted_indices.len();

    // Filter mode toggle — two mini-buttons that look like a segmented control.
    let mode_btn = |label: &'static str, mode: GlobalFilterMode| {
        let active = spreadsheet.filter_mode == mode;
        button(text(label).size(11))
            .padding([3, 8])
            .on_press(spreadsheet_msg(SpreadsheetMessage::SetFilterMode(mode)))
            .style(if active {
                style::filter_mode_active
            } else {
                style::filter_mode_inactive
            })
    };

    let mode_toggle = row![
        mode_btn("Filter", GlobalFilterMode::FilterOut),
        mode_btn("Highlight", GlobalFilterMode::Highlight),
    ]
    .spacing(2);

    let filter_input = text_input("Search records...", &spreadsheet.filter_query)
        .id(spreadsheet.filter_input_id.clone())
        .on_input(move |q| spreadsheet_msg(SpreadsheetMessage::FilterChanged(q)))
        .on_submit(spreadsheet_msg(SpreadsheetMessage::NavigateNextHighlight))
        .padding(6)
        .width(Length::FillPortion(2))
        .style(style::spreadsheet_filter_input);

    let clear_btn: Element<Message> = if spreadsheet.filter_query.is_empty() {
        Element::from(horizontal_space().width(Length::Fixed(0.0)))
    } else {
        button(text("×").size(14))
            .padding([0, 8])
            .on_press(spreadsheet_msg(SpreadsheetMessage::ClearFilter))
            .style(style::filter_clear_button)
            .into()
    };

    // Mode-dependent readout: counter or navigation pager.
    let status_area: Element<Message> = match spreadsheet.filter_mode {
        GlobalFilterMode::FilterOut => text(format!("{visible} of {total} rows"))
            .size(11)
            .style(style::filter_status_text)
            .into(),
        GlobalFilterMode::Highlight => {
            let current_label = spreadsheet
                .current_highlight_pos
                .map(|p| p + 1)
                .unwrap_or(0);

            let prev_btn = button(text("◀").size(10))
                .padding([2, 6])
                .on_press_maybe(
                    (highlight_count > 0)
                        .then(|| spreadsheet_msg(SpreadsheetMessage::NavigatePrevHighlight)),
                )
                .style(style::nav_button);

            let next_btn = button(text("▶").size(10))
                .padding([2, 6])
                .on_press_maybe(
                    (highlight_count > 0)
                        .then(|| spreadsheet_msg(SpreadsheetMessage::NavigateNextHighlight)),
                )
                .style(style::nav_button);

            let counter = if highlight_count == 0 {
                text("0 matches".to_string())
            } else {
                text(format!("{current_label} / {highlight_count}"))
            }
            .size(11)
            .style(style::filter_status_text);

            row![prev_btn, counter, next_btn]
                .spacing(6)
                .align_y(iced::Alignment::Center)
                .into()
        }
    };

    row![
        text("Filter:").size(12).style(style::subtle_text),
        mode_toggle,
        filter_input,
        clear_btn,
        horizontal_space(),
        status_area,
        horizontal_space().width(12),
        button(text("CSV").size(11))
            .on_press(spreadsheet_msg(SpreadsheetMessage::ExportCsv))
            .style(style::export_button),
        button(text("Scan").size(11))
            .on_press(scan_msg)
            .style(style::browse_button),
    ]
    .padding([8, 12])
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}
