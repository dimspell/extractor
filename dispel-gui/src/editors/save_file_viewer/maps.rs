use iced::widget::{Column, Row, button, container, mouse_area, text};
use iced::{Element, Fill, Length, Padding};

use crate::components::filter::{self, ColumnFilterAction, FilterBarExtras, GlobalFilterMode};
use crate::editors::save_file_viewer::message::{
    SaveFileViewerMessage, TableFilterAction, TableKey,
};
use crate::editors::save_file_viewer::state::{
    MapsTableKind, SaveFileViewerState, TableInteractionState,
};
use crate::message::Message;
use crate::message::MessageExt;
use crate::style;
use gui_widgets::components::modal;
use gui_widgets::{RowFlags, TableWidget};
use iced::mouse::Interaction;

/// Maps section: sidebar of map IDs + sub-nav to pick one entity table per map.
pub fn view<'a>(state: &'a SaveFileViewerState) -> Element<'a, Message> {
    let sf = match state.save_file.as_ref() {
        Some(sf) => sf,
        None => return container(text("No save file loaded")).into(),
    };

    if sf.maps.is_empty() {
        return container(text("No maps"))
            .width(Fill)
            .height(Fill)
            .padding(16)
            .into();
    }

    // ── Left sidebar: map ID buttons ─────────────────────────────────────
    let sidebar: Element<'a, Message> = {
        let mut col = iced::widget::Column::<Message>::new()
            .spacing(2)
            .padding(8)
            .width(120);
        for (i, map) in sf.maps.iter().enumerate() {
            let is_active = state.selected_map == Some(i);
            let label = state
                .map_name_lookup
                .get(&map.map_id)
                .map(|name| format!("[{}] {}", map.map_id, name))
                .unwrap_or_else(|| format!("Map {}", map.map_id));
            let mut btn = button(text(label).size(12));
            if is_active {
                btn = btn.style(style::selected_button);
            }
            col = col.push(
                btn.on_press(Message::save_file_viewer(SaveFileViewerMessage::SelectMap(
                    i,
                )))
                .padding([4, 8])
                .width(Fill),
            );
        }
        iced::widget::Scrollable::<Message>::new(col)
            .height(Fill)
            .into()
    };

    // ── Right panel: entity tables or map preview ─────────────────────────
    let main: Element<'a, Message> = if let Some(idx) = state.selected_map {
        let map = &sf.maps[idx];

        // Sub-navigation bar with Preview + entity type tabs
        let sub_nav = if state.show_preview {
            build_sub_nav_preview(map)
        } else {
            build_sub_nav(state.selected_entity_kind, map)
        };

        if state.show_preview {
            // Show map preview canvas
            match &state.map_preview {
                Some(preview) => {
                    let preview_view =
                        crate::editors::save_file_viewer::map_preview::view_preview(preview);
                    let content = Column::<Message>::new()
                        .push(sub_nav)
                        .push(preview_view)
                        .spacing(8);
                    content.into()
                }
                None => {
                    let mut start_btn = button(text("Load Preview").size(13));
                    // Always render clickable; TogglePreview guards game_path
                    start_btn = start_btn.on_press(Message::save_file_viewer(
                        SaveFileViewerMessage::TogglePreview,
                    ));
                    let content = Column::<Message>::new()
                        .push(sub_nav)
                        .push(container(start_btn).width(Fill).height(Fill).padding(16))
                        .spacing(8);
                    content.into()
                }
            }
        } else {
            let caches = state.maps_display_caches.get(idx);
            let ts_map = state.maps_table_states.get(idx);
            let kind = state.selected_entity_kind;
            let is_resizing = state.resizing.as_ref().is_some_and(|d| {
                matches!(d.key, crate::editors::save_file_viewer::message::TableKey::Map(i, k) if i == idx && k == kind)
            });
            let paragraph_cache = state.paragraph_cache.clone();

            // Render only the selected entity table
            let table = match (caches, ts_map) {
                (Some(caches), Some(ts_map)) => {
                    let (rows, indices) = table_rows(caches, kind);
                    if let Some(ts) = ts_map.get(&kind) {
                        if !rows.is_empty() {
                            entity_table(idx, kind, rows, indices, ts, is_resizing, paragraph_cache)
                        } else {
                            empty_text("(none)")
                        }
                    } else {
                        empty_text("(caches not ready)")
                    }
                }
                _ => empty_text("(caches not ready)"),
            };

            let content = Column::<Message>::new()
                .push(sub_nav)
                .push(table)
                .spacing(8);
            content.into()
        }
    } else {
        container(text("Select a map from the sidebar"))
            .width(Fill)
            .height(Fill)
            .padding(16)
            .into()
    };

    iced::widget::Row::<Message>::new()
        .push(sidebar)
        .push(main)
        .into()
}

/// Iterable list of (MapsTableKind, label) as they appear in the sub-nav.
const ALL_KINDS: [(MapsTableKind, &str); 8] = [
    (MapsTableKind::Monsters, "Monsters"),
    (MapsTableKind::Npcs, "NPCs"),
    (MapsTableKind::ExtraObjects, "Extra Objects"),
    (MapsTableKind::Weapon, "Weapons"),
    (MapsTableKind::Heal, "Heals"),
    (MapsTableKind::Edit, "Edits"),
    (MapsTableKind::Misc, "Misc"),
    (MapsTableKind::Event, "Events"),
];

/// Build the sub-navigation row of tab-like buttons.
fn build_sub_nav<'a>(
    active: MapsTableKind,
    map: &dispel_core::references::save_file::MapSectionData,
) -> Row<'a, Message> {
    let mut nav = Row::new().spacing(4).padding(8);
    for (kind, base_label) in &ALL_KINDS {
        let is_active = *kind == active;
        let count = kind_count(map, *kind);
        let label = format!("{} ({})", base_label, count);
        let mut btn = button(text(label).size(12));
        if is_active {
            btn = btn.style(style::active_tab_button);
        } else {
            btn = btn.style(style::tab_button);
        }
        nav = nav.push(
            btn.on_press(Message::save_file_viewer(
                SaveFileViewerMessage::SelectEntityKind(*kind),
            ))
            .padding([4, 8]),
        );
    }
    // Preview button (always visible; TogglePreview guards game_path)
    nav = nav.push(
        button(text("🗺 Map Preview").size(12))
            .on_press(Message::save_file_viewer(
                SaveFileViewerMessage::TogglePreview,
            ))
            .padding([4, 8]),
    );
    nav
}

/// Sub-nav bar shown when the map preview is active.
fn build_sub_nav_preview<'a>(
    map: &dispel_core::references::save_file::MapSectionData,
) -> Row<'a, Message> {
    let mut nav = Row::new().spacing(4).padding(8);
    let back_btn = button(text("← Back to Tables").size(12))
        .on_press(Message::save_file_viewer(
            SaveFileViewerMessage::TogglePreview,
        ))
        .padding([4, 8]);
    nav = nav.push(back_btn);

    // Also show entity counts as read-only labels
    for (kind, base_label) in &ALL_KINDS {
        let count = kind_count(map, *kind);
        let label = format!("{} ({})", base_label, count);
        nav = nav.push(container(text(label).size(11)).padding([4, 8]));
    }
    nav
}

/// Number of records for a given entity kind from the parsed map data.
fn kind_count(
    map: &dispel_core::references::save_file::MapSectionData,
    kind: MapsTableKind,
) -> usize {
    use MapsTableKind::*;
    match kind {
        Monsters => map.monsters.len(),
        Npcs => map.npcs.len(),
        ExtraObjects => map.extra_objects.len(),
        Weapon => map.draw_items_weapon.len(),
        Heal => map.draw_items_heal.len(),
        Edit => map.draw_items_edit.len(),
        Misc => map.draw_items_misc.len(),
        Event => map.draw_items_event.len(),
    }
}

/// Immutable access to cached display rows + indices by table kind.
fn table_rows(
    caches: &crate::editors::save_file_viewer::state::MapsDisplayCaches,
    kind: MapsTableKind,
) -> (&[Vec<String>], &[usize]) {
    use MapsTableKind::*;
    match kind {
        Monsters => (&caches.monsters[..], &caches.monsters_indices[..]),
        Npcs => (&caches.npcs[..], &caches.npcs_indices[..]),
        ExtraObjects => (&caches.extra_objects[..], &caches.extra_objects_indices[..]),
        Weapon => (&caches.draw_items_weapon, &caches.draw_items_weapon_indices),
        Heal => (&caches.draw_items_heal, &caches.draw_items_heal_indices),
        Edit => (&caches.draw_items_edit, &caches.draw_items_edit_indices),
        Misc => (&caches.draw_items_misc, &caches.draw_items_misc_indices),
        Event => (&caches.draw_items_event, &caches.draw_items_event_indices),
    }
}

fn entity_table<'a>(
    map_idx: usize,
    kind: MapsTableKind,
    display_cache: &'a [Vec<String>],
    indices: &'a [usize],
    ts: &'a TableInteractionState,
    is_resizing: bool,
    paragraph_cache: gui_widgets::components::paragraph_cache::ParagraphCache,
) -> Element<'a, Message> {
    // Build columns from the kind's default layout, then apply the
    // per-table width overrides, active sort state, and column-filter badges.
    let mut columns = kind.default_columns();
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

    let selected = ts.selected_orig;
    let filter = &ts.filter;
    let is_highlight = filter.filter_mode == GlobalFilterMode::Highlight;
    let highlighted = &filter.highlighted_indices;
    let current_highlight = filter.current_highlight_orig_idx();
    let row_flags = move |visible_idx: usize| -> RowFlags {
        let orig = indices.get(visible_idx).copied();
        RowFlags {
            selected: orig == selected,
            highlighted: is_highlight && orig.map(|o| highlighted.contains(&o)).unwrap_or(false),
            current_highlight: is_highlight && orig == current_highlight,
        }
    };

    let key = TableKey::Map(map_idx, kind);
    let msg_fn = move |action: TableFilterAction| {
        Message::save_file_viewer(SaveFileViewerMessage::TableFilter { key, action })
    };
    let filter_msg_fn = move |action: ColumnFilterAction| msg_fn(action.into());

    let table = TableWidget::new(
        display_cache,
        indices,
        columns,
        0.0,
        row_flags,
        22.0,
        paragraph_cache.clone(),
    )
    .table_state(&ts.table_state)
    .on_select(move |visible_idx| {
        Message::save_file_viewer(SaveFileViewerMessage::MapsTableSelect {
            map: map_idx,
            kind,
            visible_idx,
        })
    })
    .on_sort(move |col| {
        Message::save_file_viewer(SaveFileViewerMessage::MapsTableSort {
            map: map_idx,
            kind,
            col,
        })
    })
    .on_start_resize(move |col| {
        Message::save_file_viewer(SaveFileViewerMessage::MapsTableStartResize {
            map: map_idx,
            kind,
            col,
        })
    })
    .on_reset_column_width(move |col| {
        Message::save_file_viewer(SaveFileViewerMessage::MapsTableResetColumnWidth {
            map: map_idx,
            kind,
            col,
        })
    })
    .on_scroll(move |x, y, vh| {
        Message::save_file_viewer(SaveFileViewerMessage::MapsTableScroll {
            map: map_idx,
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

    // Bound the table to a fixed height so its layout resolves to a finite
    // size inside the outer scrollable (TableWidget fills whatever the parent
    // offers, which is unbounded here).
    let table_elem: Element<'a, Message> = container(table).height(Length::Fixed(640.0)).into();

    // While resizing this table, capture cursor moves / release across the
    // whole table area so the drag isn't interrupted by the inner widget.
    let table_elem: Element<'a, Message> = if is_resizing {
        mouse_area(table_elem)
            .on_move(move |p| {
                Message::save_file_viewer(SaveFileViewerMessage::MapsTableResizeCursor(p.x))
            })
            .on_release(Message::save_file_viewer(
                SaveFileViewerMessage::MapsTableEndResize,
            ))
            .interaction(Interaction::ResizingHorizontally)
            .into()
    } else {
        table_elem
    };

    let filter_bar = filter::build_filter_bar(
        filter.filter_mode,
        &filter.filter_query,
        filter.is_active(),
        &filter.highlighted_indices,
        filter.current_highlight_pos,
        display_cache.len(),
        indices.len(),
        filter_msg_fn,
        FilterBarExtras {
            export_csv: Some(Message::save_file_viewer(SaveFileViewerMessage::ExportCsv(
                TableKey::Map(map_idx, kind),
            ))),
            ..FilterBarExtras::default()
        },
    );
    let wrapped = Column::<Message>::new()
        .push(filter_bar)
        .push(table_elem)
        .spacing(4);

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
}

fn empty_text(msg: &str) -> Element<'static, Message> {
    container(
        text(msg.to_string())
            .color(iced::Color::from_rgb(0.5, 0.5, 0.5))
            .size(12),
    )
    .padding(Padding {
        top: 0.0,
        right: 0.0,
        bottom: 8.0,
        left: 12.0,
    })
    .width(Fill)
    .into()
}
