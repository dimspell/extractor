use iced::mouse::Interaction;
use iced::widget::{Column, Row, button, container, mouse_area, text};
use iced::{Element, Fill};

use crate::components::filter::{self, ColumnFilterAction, FilterBarExtras, GlobalFilterMode};
use crate::editors::save_file_viewer::message::{
    SaveFileViewerMessage, TableFilterAction, TableKey,
};
use crate::editors::save_file_viewer::state::{
    CharacterTableKind, SaveFileViewerState, TableFilterState,
};
use crate::message::Message;
use crate::message::MessageExt;
use crate::style;
use gui_widgets::components::modal;
use gui_widgets::{RowFlags, TableWidget};

/// Character section: kind buttons + TableWidget per character table.
pub fn view<'a>(state: &'a SaveFileViewerState) -> Element<'a, Message> {
    let active = state.character_kind;

    // Kind buttons row
    let mut buttons = Row::<Message>::new().spacing(4).padding(8);
    for kind in CharacterTableKind::all() {
        let is_active = active == Some(*kind);
        let count = state
            .character_display_caches
            .get(kind)
            .map(|c| c.len())
            .unwrap_or(0);
        let label = format!("{} ({})", kind.label(), count);
        let mut btn = button(text(label).size(12));
        if is_active {
            btn = btn.style(style::active_tab_button);
        } else {
            btn = btn.style(style::tab_button);
        }
        buttons = buttons.push(
            btn.on_press(Message::save_file_viewer(
                SaveFileViewerMessage::SelectCharacterKind(*kind),
            ))
            .padding([4, 8]),
        );
    }

    // Content: TableWidget for the selected kind
    let body: Element<'a, Message> = match active {
        Some(kind) => character_table(state, kind),
        None => container(text("Select a character table above"))
            .width(Fill)
            .height(Fill)
            .padding(16)
            .into(),
    };

    Column::<Message>::new().push(buttons).push(body).into()
}

fn character_table<'a>(
    state: &'a SaveFileViewerState,
    kind: CharacterTableKind,
) -> Element<'a, Message> {
    let ts = state.character_table_states.get(&kind);
    let is_resizing = state.resizing.as_ref().is_some_and(|d| {
        matches!(d.key, crate::editors::save_file_viewer::message::TableKey::Character(k) if k == kind)
    });

    // Build columns from the kind's default layout, then apply the
    // per-table width overrides, active sort state, and column-filter badges.
    let mut columns = kind.default_columns();
    let filter_ref: Option<&TableFilterState> = ts.map(|t| &t.filter);
    if let Some(ts) = ts {
        for (c, w) in columns.iter_mut().zip(&ts.column_widths) {
            c.width_px = *w;
        }
        if let Some(sc) = ts.sort_column
            && let Some(c) = columns.get_mut(sc)
        {
            c.sort = Some(ts.sort_ascending);
        }
        for (i, c) in columns.iter_mut().enumerate() {
            c.has_filter = ts.filter.column_filters.contains_key(&i);
        }
    }

    let display_cache = match state.character_display_caches.get(&kind) {
        Some(c) if !c.is_empty() => c,
        _ => {
            return container(text("(empty)"))
                .width(Fill)
                .height(Fill)
                .padding(16)
                .into();
        }
    };

    let filtered_indices = match state.character_filtered_indices.get(&kind) {
        Some(i) => i,
        None => {
            return container(text("(empty)"))
                .width(Fill)
                .height(Fill)
                .padding(16)
                .into();
        }
    };

    let selected = ts.and_then(|t| t.selected_orig);
    let is_highlight = filter_ref
        .map(|f| f.filter_mode == GlobalFilterMode::Highlight)
        .unwrap_or(false);
    let highlighted = filter_ref.map(|f| &f.highlighted_indices);
    let current_highlight = filter_ref.and_then(|f| f.current_highlight_orig_idx());
    let row_flags = move |visible_idx: usize| -> RowFlags {
        let orig = filtered_indices.get(visible_idx).copied();
        RowFlags {
            selected: orig == selected,
            highlighted: is_highlight
                && orig
                    .map(|o| highlighted.map(|h| h.contains(&o)).unwrap_or(false))
                    .unwrap_or(false),
            current_highlight: is_highlight && orig == current_highlight,
        }
    };

    let paragraph_cache = state.paragraph_cache.clone();
    let key = TableKey::Character(kind);
    let msg_fn = move |action: TableFilterAction| {
        Message::save_file_viewer(SaveFileViewerMessage::TableFilter { key, action })
    };
    let filter_msg_fn = move |action: ColumnFilterAction| msg_fn(action.into());

    let mut table = TableWidget::new(
        display_cache,
        filtered_indices,
        columns,
        0.0,
        row_flags,
        22.0,
        paragraph_cache.clone(),
    )
    .on_select(move |visible_idx| {
        Message::save_file_viewer(SaveFileViewerMessage::CharacterTableSelect { kind, visible_idx })
    })
    .on_sort(move |col| {
        Message::save_file_viewer(SaveFileViewerMessage::CharacterTableSort { kind, col })
    })
    .on_start_resize(move |col| {
        Message::save_file_viewer(SaveFileViewerMessage::CharacterTableStartResize { kind, col })
    })
    .on_reset_column_width(move |col| {
        Message::save_file_viewer(SaveFileViewerMessage::CharacterTableResetColumnWidth {
            kind,
            col,
        })
    })
    .on_scroll(move |x, y, vh| {
        Message::save_file_viewer(SaveFileViewerMessage::CharacterTableScroll {
            kind,
            x,
            y,
            viewport_height: vh,
        })
    })
    .on_open_filter(move |col| msg_fn(TableFilterAction::OpenColumnFilter(col)))
    .on_clear_filter(move |col| msg_fn(TableFilterAction::ClearColumnFilter(col)))
    .on_quick_filter(move |col, value| msg_fn(TableFilterAction::QuickFilter(col, value)))
    .on_next_highlight(move || msg_fn(TableFilterAction::NextHighlight))
    .on_prev_highlight(move || msg_fn(TableFilterAction::PrevHighlight));

    if let Some(ts) = ts {
        table = table.table_state(&ts.table_state);
    }

    // While resizing this table, capture cursor moves / release across the
    // whole table area so the drag isn't interrupted by the inner widget.
    let table_elem: Element<'a, Message> = if is_resizing {
        mouse_area(table)
            .on_move(move |p| {
                Message::save_file_viewer(SaveFileViewerMessage::InventoryTableResizeCursor(p.x))
            })
            .on_release(Message::save_file_viewer(
                SaveFileViewerMessage::InventoryTableEndResize,
            ))
            .interaction(Interaction::ResizingHorizontally)
            .into()
    } else {
        table.into()
    };

    let content: Element<'a, Message> = if let Some(filter) = filter_ref {
        let filter_bar = filter::build_filter_bar(
            filter.filter_mode,
            &filter.filter_query,
            filter.is_active(),
            &filter.highlighted_indices,
            filter.current_highlight_pos,
            display_cache.len(),
            filtered_indices.len(),
            filter_msg_fn,
            FilterBarExtras {
                export_csv: Some(Message::save_file_viewer(SaveFileViewerMessage::ExportCsv(
                    TableKey::Character(kind),
                ))),
                ..FilterBarExtras::default()
            },
        );
        let wrapped = Column::<Message>::new()
            .push(filter_bar)
            .push(table_elem)
            .spacing(8);

        if let Some(col) = filter.active_column_filter {
            let modal_content = filter::build_column_filter_modal(
                col,
                &filter.column_filter_search,
                &filter.column_filter_options,
                &filter.column_filters,
                filter_msg_fn,
            );
            modal::modal(
                wrapped,
                modal_content,
                move || filter_msg_fn(ColumnFilterAction::CloseColumnFilterModal),
                0.5,
            )
        } else {
            wrapped.into()
        }
    } else {
        table_elem
    };

    content
}
