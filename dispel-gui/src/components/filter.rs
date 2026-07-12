//! Shared column-filter modal + filter bar, composable via
//! [`ColumnFilterAction`].  Both the spreadsheet editor and the save file
//! viewer use these builders with a `msg_fn: Fn(ColumnFilterAction) -> Message`
//! closure so the UI code is written exactly once.

use std::collections::{HashMap, HashSet};

use crate::components::utils::{horizontal_rule, horizontal_space};
use crate::message::Message;
use crate::style;
use gui_widgets::lucide::{icon_char, LUCIDE_FONT};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};
use lucide_icons::Icon;

// ── Shared types ───────────────────────────────────────────────────────

/// How the global query behaves when matching rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GlobalFilterMode {
    /// Rows that do not match the query are hidden from the view.
    #[default]
    FilterOut,
    /// Non-matching rows remain visible; matching rows are highlighted and
    /// navigable with prev/next.
    Highlight,
}

/// A single distinct value offered in a column's filter dropdown, with the
/// number of table rows that carry this value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColumnFilterOption {
    pub value: String,
    pub count: usize,
}

/// Actions emitted by the shared [`build_column_filter_modal`] and
/// [`build_filter_bar`].  Each action maps to a variant in the consumer's own
/// message enum via `From<ColumnFilterAction>`.
#[derive(Debug, Clone)]
pub enum ColumnFilterAction {
    /// Toggle whether a single value is selected in the active column filter.
    ToggleColumnFilterValue(usize, String),
    /// Select every (search-filtered) value in the active column filter.
    SelectAllColumnFilter(usize),
    /// Clear every (search-filtered) value in the active column filter.
    ClearAllColumnFilter(usize),
    /// Close the column filter modal without applying (state is kept).
    CloseColumnFilterModal,
    /// Update the search box text inside the column filter modal.
    ColumnFilterSearch(String),
    /// Switch between FilterOut and Highlight global query behaviour.
    SetMode(GlobalFilterMode),
    /// Update the free-text global query box.
    QueryChanged(String),
    /// Clear every column filter and the global query.
    ClearAllFilters,
    /// Jump to the next highlighted row (Highlight mode).
    NextHighlight,
    /// Jump to the previous highlighted row (Highlight mode).
    PrevHighlight,
}

/// Optional editor-specific controls rendered at the right end of the filter
/// bar (CSV export, Scan, Add record, Remove record).  Pass [`Default`] /
/// all-`None` when these are not applicable.
#[derive(Debug, Clone, Default)]
pub struct FilterBarExtras {
    pub export_csv: Option<Message>,
    pub scan: Option<Message>,
    pub add: Option<Message>,
    pub remove: Option<Message>,
}

// ── Column filter modal ────────────────────────────────────────────────

/// Searchable multi-select dropdown for one column's distinct values.
///
/// `msg_fn` translates a [`ColumnFilterAction`] into the consumer's own
/// `Message` type (e.g. via `.into()`).
pub fn build_column_filter_modal<'a, F>(
    col: usize,
    search: &str,
    options: &[ColumnFilterOption],
    filters: &HashMap<usize, HashSet<String>>,
    msg_fn: F,
) -> Element<'a, Message>
where
    F: Fn(ColumnFilterAction) -> Message + Copy + 'a,
{
    let search_lower = search.to_lowercase();
    let filtered_options: Vec<_> = options
        .iter()
        .filter(|opt| opt.value.to_lowercase().contains(&search_lower))
        .collect();

    let current_filter = filters.get(&col);
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
                .on_press(msg_fn(ColumnFilterAction::ToggleColumnFilterValue(
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
        .on_press(msg_fn(ColumnFilterAction::SelectAllColumnFilter(col)))
        .padding([6, 12])
        .style(style::commit_button);
    let clear_all_btn = button(text("Clear All").size(11))
        .on_press(msg_fn(ColumnFilterAction::ClearAllColumnFilter(col)))
        .padding([6, 12])
        .style(style::browse_button);

    let header = row![
        text("Filter Column").size(14).style(style::section_header),
        horizontal_space(),
        button(text(icon_char(Icon::X)).font(LUCIDE_FONT).size(14))
            .on_press(msg_fn(ColumnFilterAction::CloseColumnFilterModal))
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
            text_input("Search options...", search)
                .on_input(move |q| msg_fn(ColumnFilterAction::ColumnFilterSearch(q)))
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

// ── Filter bar ─────────────────────────────────────────────────────────

/// Top filter bar: mode toggle, global query input, clear button, row
/// counter / highlight pager, and optional editor extras (CSV, Scan, Add,
/// Remove).
///
/// `is_active` controls whether the "×" clear button is shown; pass
/// `!filter_query.is_empty()` or `state.is_active()` depending on whether
/// you want to show it only when the query is non-empty or whenever *any*
/// filter (column + query) is active.
#[allow(clippy::too_many_arguments)]
pub fn build_filter_bar<'a, F>(
    filter_mode: GlobalFilterMode,
    filter_query: &str,
    is_active: bool,
    highlighted_indices: &[usize],
    current_highlight_pos: Option<usize>,
    total: usize,
    visible: usize,
    msg_fn: F,
    extras: FilterBarExtras,
) -> Element<'a, Message>
where
    F: Fn(ColumnFilterAction) -> Message + Copy + 'a,
{
    let highlight_count = highlighted_indices.len();

    let mode_btn = |label: &'static str, mode: GlobalFilterMode| {
        let active = filter_mode == mode;
        button(text(label).size(11))
            .padding([3, 8])
            .on_press(msg_fn(ColumnFilterAction::SetMode(mode)))
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

    let filter_input = text_input("Search records...", filter_query)
        .on_input(move |q| msg_fn(ColumnFilterAction::QueryChanged(q)))
        .on_submit(msg_fn(ColumnFilterAction::NextHighlight))
        .padding(6)
        .width(Length::FillPortion(2))
        .style(style::spreadsheet_filter_input);

    let clear_btn: Element<Message> = if is_active {
        button(text("×").size(14))
            .padding([0, 8])
            .on_press(msg_fn(ColumnFilterAction::ClearAllFilters))
            .style(style::filter_clear_button)
            .into()
    } else {
        horizontal_space().width(Length::Fixed(0.0)).into()
    };

    let status_area: Element<Message> = match filter_mode {
        GlobalFilterMode::FilterOut => text(format!("{visible} of {total} rows"))
            .size(11)
            .style(style::filter_status_text)
            .into(),
        GlobalFilterMode::Highlight => {
            let current_label = current_highlight_pos
                .map(|p| p + 1)
                .unwrap_or(0);

            let prev_btn = button(text(icon_char(Icon::ChevronLeft)).font(LUCIDE_FONT).size(10))
                .padding([2, 6])
                .on_press_maybe(
                    (highlight_count > 0)
                        .then(|| msg_fn(ColumnFilterAction::PrevHighlight)),
                )
                .style(style::nav_button);

            let next_btn = button(text(icon_char(Icon::ChevronRight)).font(LUCIDE_FONT).size(10))
                .padding([2, 6])
                .on_press_maybe(
                    (highlight_count > 0)
                        .then(|| msg_fn(ColumnFilterAction::NextHighlight)),
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

    // Optional editor extras — only the spreadsheet passes these; the
    // save-file-viewer passes all-None (which renders zero-width spacers).
    let add_btn: Element<'a, Message> = match extras.add {
        Some(msg) => button(text("+").size(14))
            .on_press(msg)
            .style(style::browse_button)
            .into(),
        None => horizontal_space().width(0).into(),
    };
    let remove_btn: Element<'a, Message> = match extras.remove {
        Some(msg) => button(text("−").size(14))
            .on_press(msg)
            .style(style::browse_button)
            .into(),
        None => horizontal_space().width(0).into(),
    };
    let csv_btn: Element<'a, Message> = match extras.export_csv {
        Some(msg) => button(text("CSV").size(11))
            .on_press(msg)
            .style(style::export_button)
            .into(),
        None => horizontal_space().width(0).into(),
    };
    let scan_btn: Element<'a, Message> = match extras.scan {
        Some(msg) => button(text("Scan").size(11))
            .on_press(msg)
            .style(style::browse_button)
            .into(),
        None => horizontal_space().width(0).into(),
    };

    row![
        text("Filter:").size(12).style(style::subtle_text),
        mode_toggle,
        filter_input,
        clear_btn,
        horizontal_space(),
        status_area,
        horizontal_space().width(12),
        add_btn,
        remove_btn,
        csv_btn,
        scan_btn,
    ]
    .padding([8, 12])
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .accessible_label("Filter bar")
    .into()
}
