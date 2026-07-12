use iced::mouse::Interaction;
use iced::widget::{button, container, mouse_area, text, Column};
use iced::{Element, Fill};

use crate::editors::save_file_viewer::filter_modal;
use crate::editors::save_file_viewer::message::{SaveFileViewerMessage, TableFilterAction, TableKey};
use gui_widgets::components::modal;
use crate::editors::save_file_viewer::state::{
    GlobalFilterMode, JournalSection, SaveFileViewerState,
};
use crate::message::Message;
use crate::message::MessageExt;
use gui_widgets::components::paragraph_cache::ParagraphCache;
use gui_widgets::{RowFlags, TableWidget};

/// Journal section: sub-tabs (Main/Side/Trade) + table per section.
pub fn view<'a>(state: &'a SaveFileViewerState) -> Element<'a, Message> {
    // Sub-tab bar
    let sections = [
        (JournalSection::Main, "Main"),
        (JournalSection::Side, "Side"),
        (JournalSection::Trade, "Trade"),
    ];

    let mut tab_bar = iced::widget::Row::new().spacing(4).padding(8);
    for (section, label) in &sections {
        let is_active = *section == state.journal_section;
        let mut btn = button(text(*label).size(13));
        if is_active {
            btn = btn.style(iced::widget::button::primary);
        }
        tab_bar = tab_bar.push(
            btn.on_press(Message::save_file_viewer(
                SaveFileViewerMessage::SelectJournalSection(*section),
            ))
            .padding([4, 12]),
        );
    }

    // Table for the active section
    let section = state.journal_section;
    let display_cache = state.journal_display_caches.get(&section);
    let filtered_indices = state.journal_filtered_indices.get(&section);
    let ts = state.journal_table_states.get(&section);
    let resizing = state.journal_resizing.as_ref().map(|d| d.section) == Some(section);

    let table: Element<'a, Message> = match (display_cache, filtered_indices, ts) {
        (Some(cache), Some(indices), Some(ts)) if !cache.is_empty() => {
            // Build columns from the section's default layout, then apply the
            // per-table width overrides, active sort state, and filter badges.
            let mut columns = section.default_columns();
            for (c, w) in columns.iter_mut().zip(&ts.column_widths) {
                c.width_px = *w;
            }
            if let Some(sc) = ts.sort_column {
                if let Some(c) = columns.get_mut(sc) {
                    c.sort = Some(ts.sort_ascending);
                }
            }
            for (i, c) in columns.iter_mut().enumerate() {
                c.has_filter = ts.filter.column_filters.contains_key(&i);
            }

            let selected = ts.selected_orig;
            let filter = &ts.filter;
            let is_highlight = filter.filter_mode == GlobalFilterMode::Highlight;
            let highlighted = &filter.highlighted_indices;
            let current_highlight = filter.current_highlight_orig_idx();
            let row_flags = move |visible_idx: usize| -> RowFlags {
                let orig = indices.get(visible_idx).copied();
                RowFlags {
                    selected: orig == selected,
                    highlighted: is_highlight
                        && orig.map(|o| highlighted.contains(&o)).unwrap_or(false),
                    current_highlight: is_highlight && orig == current_highlight,
                    ..Default::default()
                }
            };

            let key = TableKey::Journal(section);
            let msg_fn = move |action: TableFilterAction| {
                Message::save_file_viewer(SaveFileViewerMessage::TableFilter { key, action })
            };

            let table = TableWidget::new(
                cache,
                indices,
                columns,
                0.0,
                row_flags,
                22.0,
                ParagraphCache::default(),
            )
            .table_state(&ts.table_state)
            .on_select(move |visible_idx| {
                Message::save_file_viewer(SaveFileViewerMessage::JournalTableSelect {
                    section,
                    visible_idx,
                })
            })
            .on_sort(move |col| {
                Message::save_file_viewer(SaveFileViewerMessage::JournalTableSort { section, col })
            })
            .on_start_resize(move |col| {
                Message::save_file_viewer(SaveFileViewerMessage::JournalTableStartResize {
                    section,
                    col,
                })
            })
            .on_reset_column_width(move |col| {
                Message::save_file_viewer(SaveFileViewerMessage::JournalTableResetColumnWidth {
                    section,
                    col,
                })
            })
            .on_scroll(move |x, y, vh| {
                Message::save_file_viewer(SaveFileViewerMessage::JournalTableScroll {
                    section,
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

            // While resizing this table, capture cursor moves / release across
            // the whole table area so the drag isn't interrupted.
            let table_elem: Element<'a, Message> = if resizing {
                mouse_area(table)
                    .on_move(move |p| {
                        Message::save_file_viewer(SaveFileViewerMessage::JournalTableResizeCursor(
                            p.x,
                        ))
                    })
                    .on_release(Message::save_file_viewer(
                        SaveFileViewerMessage::JournalTableEndResize,
                    ))
                    .interaction(Interaction::ResizingHorizontally)
                    .into()
            } else {
                table.into()
            };

            let filter_bar =
                filter_modal::build_filter_bar(filter, cache.len(), indices.len(), msg_fn);
            let wrapped = Column::<Message>::new()
                .push(filter_bar)
                .push(table_elem)
                .spacing(8);

            if filter.active_column_filter.is_some() {
                let col = filter.active_column_filter.unwrap();
                let modal_content = filter_modal::build_column_filter_modal(col, filter, msg_fn);
                modal::modal(
                    wrapped,
                    modal_content,
                    move || msg_fn(TableFilterAction::CloseColumnFilterModal),
                    0.5,
                )
            } else {
                wrapped.into()
            }
        }
        _ => container(text("No entries")).width(Fill).padding(16).into(),
    };

    Column::<Message>::new().push(tab_bar).push(table).into()
}
