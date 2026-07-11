use iced::mouse::Interaction;
use iced::widget::{container, mouse_area, text};
use iced::{Element, Fill};

use crate::editors::save_file_viewer::state::{events_default_columns, SaveFileViewerState};
use crate::editors::save_file_viewer::SaveFileViewerMessage;
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

    // Build columns from the default layout, then apply the per-table width
    // overrides and active sort state.
    let mut columns = events_default_columns();
    for (c, w) in columns.iter_mut().zip(&ts.column_widths) {
        c.width_px = *w;
    }
    if let Some(sc) = ts.sort_column {
        if let Some(c) = columns.get_mut(sc) {
            c.sort = Some(ts.sort_ascending);
        }
    }

    let selected = ts.selected_orig;
    let scroll = ts.scroll_offset;
    let row_flags = move |visible_idx: usize| -> RowFlags {
        let orig = state.events_filtered_indices.get(visible_idx).copied();
        RowFlags {
            selected: orig == selected,
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
    .external_offset(scroll.0, scroll.1)
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
    });

    // While resizing this table, capture cursor moves / release across the
    // whole table area so the drag isn't interrupted by the inner widget.
    if resizing {
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
    }
}
