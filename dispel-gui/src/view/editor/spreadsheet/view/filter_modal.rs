//! Column quick-filter modal — a searchable, multi-select dropdown of unique
//! values for one column. Opened via the `▾` icon in the table header.

use crate::message::Message;
use crate::style;
use crate::utils::{horizontal_rule, horizontal_space};
use crate::view::editor::spreadsheet::message::SpreadsheetMessage;
use crate::view::editor::spreadsheet::state::SpreadsheetState;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

pub fn build_column_filter_modal<'a>(
    col: usize,
    spreadsheet: &'a SpreadsheetState,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> Element<'a, Message> {
    let search_lower = spreadsheet.column_filter_search.to_lowercase();
    let filtered_options: Vec<_> = spreadsheet
        .column_filter_options
        .iter()
        .filter(|opt| opt.value.to_lowercase().contains(&search_lower))
        .collect();

    let current_filter = spreadsheet.column_filters.get(&col);
    let option_buttons: Vec<Element<Message>> = filtered_options
        .iter()
        .map(|opt| {
            let is_checked = current_filter
                .map(|s| s.contains(&opt.value))
                .unwrap_or(false);
            let label = if is_checked {
                format!("✓ {} ({})", opt.value, opt.count)
            } else {
                format!("  {} ({})", opt.value, opt.count)
            };
            button(text(label).size(11))
                .on_press(spreadsheet_msg(
                    SpreadsheetMessage::ToggleColumnFilterValue(col, opt.value.clone()),
                ))
                .width(Length::Fill)
                .padding(6)
                .style(if is_checked {
                    style::selected_button
                } else {
                    style::browse_button
                })
                .into()
        })
        .collect();

    let options_scroll = scrollable(column(option_buttons).spacing(2))
        .height(Length::Fixed(200.0))
        .width(Length::Fill);

    let select_all_btn = button(text("Select All").size(11))
        .on_press(spreadsheet_msg(SpreadsheetMessage::SelectAllColumnFilter(
            col,
        )))
        .padding([6, 12])
        .style(style::commit_button);
    let clear_all_btn = button(text("Clear All").size(11))
        .on_press(spreadsheet_msg(SpreadsheetMessage::ClearAllColumnFilter(
            col,
        )))
        .padding([6, 12])
        .style(style::browse_button);

    let header = row![
        text("Filter Column").size(14).style(style::section_header),
        horizontal_space(),
        button(text("✕").size(14))
            .on_press(spreadsheet_msg(SpreadsheetMessage::CloseColumnFilterModal))
            .padding([4, 12])
            .style(style::filter_clear_button),
    ]
    .align_y(iced::Alignment::Center)
    .spacing(8)
    .padding([8, 12]);

    let actions = row![select_all_btn, clear_all_btn]
        .spacing(8)
        .padding([8, 12]);

    container(
        column![
            header,
            horizontal_rule(1),
            text_input("Search options...", &spreadsheet.column_filter_search)
                .on_input(move |q| spreadsheet_msg(SpreadsheetMessage::ColumnFilterSearch(q)))
                .padding(8)
                .width(Length::Fill)
                .style(style::spreadsheet_filter_input),
            options_scroll,
            horizontal_rule(1),
            actions,
        ]
        .spacing(4),
    )
    .width(Length::Fixed(240.0))
    .style(style::modal_container)
    .into()
}
