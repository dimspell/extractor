//! Table content — wraps the custom `TableWidget` with an empty-catalog
//! placeholder and the resize-drag `mouse_area`.

use crate::components::editor::editable::{EditableRecord, FieldDescriptor};
use crate::message::Message;
use crate::style;
use crate::view::editor::spreadsheet::constants::{ID_COL_WIDTH_PX, ROW_HEIGHT};
use crate::view::editor::spreadsheet::message::SpreadsheetMessage;
use crate::view::editor::spreadsheet::state::{GlobalFilterMode, SpreadsheetState};
use crate::view::editor::table_widget::{RowFlags, TableColumn, TableWidget};
use iced::widget::{container, text};
use iced::{Element, Fill};

pub fn build_table_content<'a, R: EditableRecord>(
    descriptors: &'a [FieldDescriptor],
    catalog: Option<&'a Vec<R>>,
    spreadsheet: &'a SpreadsheetState,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> Element<'a, Message> {
    if catalog.is_none() {
        return container(
            text("No data loaded. Click Scan to load records.")
                .size(13)
                .style(style::subtle_text),
        )
        .width(Fill)
        .padding(20)
        .into();
    }

    build_table_content_widget(descriptors, spreadsheet, spreadsheet_msg)
}

fn build_table_content_widget<'a>(
    descriptors: &'a [FieldDescriptor],
    spreadsheet: &'a SpreadsheetState,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> Element<'a, Message> {
    let columns: Vec<TableColumn> = (0..descriptors.len())
        .map(|c| TableColumn {
            width_px: spreadsheet.column_width(c),
            label: descriptors[c].label.to_string(),
            sort: if spreadsheet.sort_column == Some(c) {
                Some(spreadsheet.sort_ascending)
            } else {
                None
            },
            has_filter: spreadsheet.column_filters.contains_key(&c),
        })
        .collect();

    let current_highlight_orig = spreadsheet.current_highlight_orig_idx();
    let is_highlight_mode = spreadsheet.filter_mode == GlobalFilterMode::Highlight;
    let selected_orig = spreadsheet.selected_orig;
    let highlighted = &spreadsheet.highlighted_indices;

    let row_flags = move |visible_idx: usize| -> RowFlags {
        let Some(&orig_idx) = spreadsheet.filtered_indices.get(visible_idx) else {
            return RowFlags::default();
        };
        RowFlags {
            selected: selected_orig == Some(orig_idx),
            highlighted: is_highlight_mode && highlighted.contains(&orig_idx),
            current_highlight: Some(orig_idx) == current_highlight_orig,
        }
    };

    let body: Element<Message> = TableWidget::new(
        &spreadsheet.display_cache,
        &spreadsheet.filtered_indices,
        columns,
        ID_COL_WIDTH_PX,
        row_flags,
        ROW_HEIGHT,
        spreadsheet.paragraph_cache.clone(),
    )
    .external_offset(
        spreadsheet.horizontal_scroll_offset,
        spreadsheet.vertical_scroll_offset,
    )
    .on_select(move |visible_idx| spreadsheet_msg(SpreadsheetMessage::SelectRow(visible_idx)))
    .on_scroll(move |x, y, vh| {
        spreadsheet_msg(SpreadsheetMessage::BodyScrolled(
            iced::widget::scrollable::AbsoluteOffset { x, y },
            vh,
        ))
    })
    .on_sort(move |c| spreadsheet_msg(SpreadsheetMessage::SortColumn(c)))
    .on_open_filter(move |c| spreadsheet_msg(SpreadsheetMessage::OpenColumnFilter(c)))
    .on_clear_filter(move |c| spreadsheet_msg(SpreadsheetMessage::ClearColumnFilter(c)))
    .on_start_resize(move |c| spreadsheet_msg(SpreadsheetMessage::StartResizeColumn(c)))
    .on_reset_column_width(move |c| spreadsheet_msg(SpreadsheetMessage::ResetColumnWidth(c)))
    .on_next_highlight(move || spreadsheet_msg(SpreadsheetMessage::NavigateNextHighlight))
    .on_prev_highlight(move || spreadsheet_msg(SpreadsheetMessage::NavigatePrevHighlight))
    .on_escape(move || spreadsheet_msg(SpreadsheetMessage::ClearFilter))
    .on_quick_filter(move |col, value| spreadsheet_msg(SpreadsheetMessage::QuickFilter(col, value)))
    .into();

    if spreadsheet.resizing_column.is_some() {
        iced::widget::mouse_area(body)
            .on_move(move |p| spreadsheet_msg(SpreadsheetMessage::ResizeColumnCursor(p.x)))
            .on_release(spreadsheet_msg(SpreadsheetMessage::EndResizeColumn))
            .interaction(iced::mouse::Interaction::ResizingHorizontally)
            .into()
    } else {
        body
    }
}
