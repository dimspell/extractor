use iced::mouse::Interaction;
use iced::widget::{container, mouse_area, text};
use iced::{Element, Fill};

use crate::components::filter::{self, ColumnFilterAction, FilterBarExtras, GlobalFilterMode};
use crate::editors::save_file_viewer::message::{
    SaveFileViewerMessage, TableFilterAction, TableKey,
};
use gui_widgets::components::modal;
use crate::editors::save_file_viewer::state::{
    events_default_columns, SaveFileViewerState,
};
use crate::message::Message;
use crate::message::MessageExt;
use gui_widgets::components::paragraph_cache::ParagraphCache;
use gui_widgets::{RowFlags, TableWidget};

/// Events section: virtualized table of event script records.
pub fn view<'a>(state: &'a SaveFileViewerState) -> Element<'a, Message> {
    let n = state.events_filtered_indices.len();
    if n == 0 {
        return container(text("No events"))
            .width(Fill)
            .height(Fill)
            .padding(16)
            .into();
    }

    let ts = &state.events_table_state;
    let resizing = state.events_resizing.is_some();
    let filter = &ts.filter;
    let key = TableKey::Events;
    let msg_fn = move |action: TableFilterAction| {
        Message::save_file_viewer(SaveFileViewerMessage::TableFilter { key, action })
    };
    let filter_msg_fn = move |action: ColumnFilterAction| msg_fn(action.into());

    // Build columns from the default layout, then apply the per-table width
    // overrides, active sort state, and column-filter badges.
    let mut columns = events_default_columns();
    for (c, w) in columns.iter_mut().zip(&ts.column_widths) {
        c.width_px = *w;
    }
    for (c, has) in columns
        .iter_mut()
        .enumerate()
        .map(|(i, c)| (c, filter.column_filters.contains_key(&i)))
    {
        c.has_filter = has;
    }
    if let Some(sc) = ts.sort_column {
        if let Some(c) = columns.get_mut(sc) {
            c.sort = Some(ts.sort_ascending);
        }
    }

    let selected = ts.selected_orig;
    let highlighted = &filter.highlighted_indices;
    let is_highlight = filter.filter_mode == GlobalFilterMode::Highlight;
    let current_highlight = filter.current_highlight_orig_idx();
    let row_flags = move |visible_idx: usize| -> RowFlags {
        let orig = state.events_filtered_indices.get(visible_idx).copied();
        RowFlags {
            selected: orig == selected,
            highlighted: is_highlight && orig.map(|o| highlighted.contains(&o)).unwrap_or(false),
            current_highlight: is_highlight && orig == current_highlight,
            ..Default::default()
        }
    };

    let table = TableWidget::new(
        &state.events_display_cache,
        &state.events_filtered_indices,
        columns,
        0.0, // no separate id col
        row_flags,
        22.0,
        ParagraphCache::default(),
    )
    .table_state(&ts.table_state)
    .on_select(|visible_idx| {
        Message::save_file_viewer(SaveFileViewerMessage::EventsTableSelect { visible_idx })
    })
    .on_sort(|col| Message::save_file_viewer(SaveFileViewerMessage::EventsTableSort { col }))
    .on_start_resize(|col| {
        Message::save_file_viewer(SaveFileViewerMessage::EventsTableStartResize { col })
    })
    .on_reset_column_width(|col| {
        Message::save_file_viewer(SaveFileViewerMessage::EventsTableResetColumnWidth { col })
    })
    .on_scroll(|x, y, vh| {
        Message::save_file_viewer(SaveFileViewerMessage::EventsTableScroll {
            x,
            y,
            viewport_height: vh,
        })
    })
    .on_open_filter(move |col| {
        msg_fn(TableFilterAction::OpenColumnFilter(col))
    })
    .on_clear_filter(move |col| {
        msg_fn(TableFilterAction::ClearColumnFilter(col))
    })
    .on_quick_filter(move |col, value| {
        msg_fn(TableFilterAction::QuickFilter(col, value))
    })
    .on_next_highlight(move || msg_fn(TableFilterAction::NextHighlight))
    .on_prev_highlight(move || msg_fn(TableFilterAction::PrevHighlight));

    // While resizing this table, capture cursor moves / release across the
    // whole table area so the drag isn't interrupted by the inner widget.
    let table_element: Element<'a, Message> = if resizing {
        mouse_area(table)
            .on_move(|p| {
                Message::save_file_viewer(SaveFileViewerMessage::EventsTableResizeCursor(p.x))
            })
            .on_release(Message::save_file_viewer(
                SaveFileViewerMessage::EventsTableEndResize,
            ))
            .interaction(Interaction::ResizingHorizontally)
            .into()
    } else {
        table.into()
    };

    let filter_bar = filter::build_filter_bar(
        filter.filter_mode,
        &filter.filter_query,
        filter.is_active(),
        &filter.highlighted_indices,
        filter.current_highlight_pos,
        state.events_display_cache.len(),
        state.events_filtered_indices.len(),
        filter_msg_fn,
        FilterBarExtras::default(),
    );

    let content = iced::widget::column![filter_bar, table_element].spacing(8);

    if filter.active_column_filter.is_some() {
        let col = filter.active_column_filter.unwrap();
        let modal_content = filter::build_column_filter_modal(
            col,
            &filter.column_filter_search,
            &filter.column_filter_options,
            &filter.column_filters,
            filter_msg_fn,
        );
        modal::modal(
            content,
            modal_content,
            move || filter_msg_fn(ColumnFilterAction::CloseColumnFilterModal),
            0.5,
        )
    } else {
        content.into()
    }
}
