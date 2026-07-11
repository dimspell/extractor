//! Column quick-filter modal + global filter bar for the save file viewer,
//! mirroring `view/editor/spreadsheet/view/filter_modal.rs` / `filter_bar.rs`.
//! The callbacks are generic over a `TableKey` so the same UI drives every
//! viewer table (maps, inventory, events, journal).

use crate::components::utils::{horizontal_rule, horizontal_space};
use crate::editors::save_file_viewer::message::TableFilterAction;
use crate::editors::save_file_viewer::state::{GlobalFilterMode, TableFilterState};
use crate::message::Message;
use crate::style;
use gui_widgets::lucide::{icon_char, LUCIDE_FONT};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};
use lucide_icons::Icon;

/// Build the searchable multi-select dropdown for one column's distinct
/// values. `msg_fn` maps a `TableFilterAction` to the viewer message with the
/// correct `TableKey` already bound.
pub fn build_column_filter_modal<'a, F>(
    col: usize,
    filter: &'a TableFilterState,
    msg_fn: F,
) -> Element<'a, Message>
where
    F: Fn(TableFilterAction) -> Message + Copy + 'a,
{
    let search_lower = filter.column_filter_search.to_lowercase();
    let filtered_options: Vec<_> = filter
        .column_filter_options
        .iter()
        .filter(|opt| opt.value.to_lowercase().contains(&search_lower))
        .collect();

    let current_filter = filter.column_filters.get(&col);
    let option_buttons: Vec<Element<Message>> = filtered_options
        .iter()
        .map(|opt| {
            let is_checked = current_filter
                .map(|s| s.contains(&opt.value))
                .unwrap_or(false);
            let content: Element<Message> = if is_checked {
                row![
                    text(icon_char(Icon::Check)).font(LUCIDE_FONT).size(11),
                    text(format!(" {} ({})", opt.value, opt.count)).size(11),
                ]
                .spacing(2)
                .into()
            } else {
                text(format!("  {} ({})", opt.value, opt.count))
                    .size(11)
                    .into()
            };
            button(content)
                .on_press(msg_fn(TableFilterAction::ToggleColumnFilterValue(
                    col,
                    opt.value.clone(),
                )))
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
        .on_press(msg_fn(TableFilterAction::SelectAllColumnFilter(col)))
        .padding([6, 12])
        .style(style::commit_button);
    let clear_all_btn = button(text("Clear All").size(11))
        .on_press(msg_fn(TableFilterAction::ClearAllColumnFilter(col)))
        .padding([6, 12])
        .style(style::browse_button);

    let header = row![
        text("Filter Column").size(14).style(style::section_header),
        horizontal_space(),
        button(text(icon_char(Icon::X)).font(LUCIDE_FONT).size(14))
            .on_press(msg_fn(TableFilterAction::CloseColumnFilterModal))
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
            text_input("Search options...", &filter.column_filter_search)
                .on_input(move |q| msg_fn(TableFilterAction::ColumnFilterSearch(q)))
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

/// Build the top filter bar: mode toggle, global query input, clear button,
/// and (in Highlight mode) a prev/next pager over the highlighted matches.
pub fn build_filter_bar<'a, F>(
    filter: &'a TableFilterState,
    total: usize,
    visible: usize,
    msg_fn: F,
) -> Element<'a, Message>
where
    F: Fn(TableFilterAction) -> Message + Copy + 'a,
{
    let highlight_count = filter.highlighted_indices.len();

    let mode_btn = |label: &'static str, mode: GlobalFilterMode| {
        let active = filter.filter_mode == mode;
        button(text(label).size(11))
            .padding([3, 8])
            .on_press(msg_fn(TableFilterAction::SetMode(mode)))
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

    let filter_input = text_input("Search records...", &filter.filter_query)
        .on_input(move |q| msg_fn(TableFilterAction::QueryChanged(q)))
        .on_submit(msg_fn(TableFilterAction::NextHighlight))
        .padding(6)
        .width(Length::FillPortion(2))
        .style(style::spreadsheet_filter_input);

    let clear_btn: Element<Message> = if filter.is_active() {
        button(text("×").size(14))
            .padding([0, 8])
            .on_press(msg_fn(TableFilterAction::ClearAllFilters))
            .style(style::filter_clear_button)
            .into()
    } else {
        horizontal_space().width(Length::Fixed(0.0)).into()
    };

    let status_area: Element<Message> = match filter.filter_mode {
        GlobalFilterMode::FilterOut => text(format!("{visible} of {total} rows"))
            .size(11)
            .style(style::filter_status_text)
            .into(),
        GlobalFilterMode::Highlight => {
            let current_label = filter
                .current_highlight_pos
                .map(|p| p + 1)
                .unwrap_or(0);

            let prev_btn = button(text(icon_char(Icon::ChevronLeft)).font(LUCIDE_FONT).size(10))
                .padding([2, 6])
                .on_press_maybe(
                    (highlight_count > 0)
                        .then(|| msg_fn(TableFilterAction::PrevHighlight)),
                )
                .style(style::nav_button);

            let next_btn = button(text(icon_char(Icon::ChevronRight)).font(LUCIDE_FONT).size(10))
                .padding([2, 6])
                .on_press_maybe(
                    (highlight_count > 0)
                        .then(|| msg_fn(TableFilterAction::NextHighlight)),
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
    ]
    .padding([8, 12])
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .accessible_label("Filter bar")
    .into()
}
