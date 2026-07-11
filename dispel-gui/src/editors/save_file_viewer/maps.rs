use iced::mouse::Interaction;
use iced::widget::{button, container, mouse_area, scrollable, text, Column};
use iced::{Element, Fill, Length, Padding};

use crate::editors::save_file_viewer::filter_modal;
use crate::editors::save_file_viewer::message::{SaveFileViewerMessage, TableFilterAction, TableKey};
use gui_widgets::components::modal;
use crate::editors::save_file_viewer::state::{
    GlobalFilterMode, MapTableState, MapsTableKind, SaveFileViewerState,
};
use crate::message::Message;
use crate::message::MessageExt;
use gui_widgets::components::paragraph_cache::ParagraphCache;
use gui_widgets::{RowFlags, TableWidget};

/// Maps section: vertical sidebar + entity tables per map.
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

    // Left sidebar: map ID buttons
    let sidebar: Element<'a, Message> = {
        let mut col = iced::widget::Column::<Message>::new()
            .spacing(2)
            .padding(8)
            .width(120);
        for (i, map) in sf.maps.iter().enumerate() {
            let is_active = state.selected_map == Some(i);
            let mut btn = button(text(format!("Map {}", map.map_id)).size(12));
            if is_active {
                btn = btn.style(iced::widget::button::primary);
            }
            col = col.push(
                btn.on_press(Message::save_file_viewer(
                    SaveFileViewerMessage::SelectMap(i),
                ))
                .padding([4, 8])
                .width(Fill),
            );
        }
        iced::widget::Scrollable::<Message>::new(col)
            .height(Fill)
            .into()
    };

    // Right panel: entity tables for selected map
    let main: Element<'a, Message> = if let Some(idx) = state.selected_map {
        let caches = state.maps_display_caches.get(idx);
        let ts_map = state.maps_table_states.get(idx);
        let resizing = state
            .maps_resizing
            .as_ref()
            .map(|d| (d.map, d.kind));
        let map = &sf.maps[idx];

        let mut content = Column::<Message>::new().spacing(12).padding(16);

        // Monsters table
        content = content.push(section_header(&format!("Monsters ({})", map.monsters.len())));
        content = push_map_table(
            content,
            idx,
            MapsTableKind::Monsters,
            caches.map(|c| (&c.monsters[..], &c.monsters_indices[..])),
            ts_map.and_then(|m| m.get(&MapsTableKind::Monsters)),
            resizing,
        );

        // NPCs table
        content = content.push(section_header(&format!("NPCs ({})", map.npcs.len())));
        content = push_map_table(
            content,
            idx,
            MapsTableKind::Npcs,
            caches.map(|c| (&c.npcs[..], &c.npcs_indices[..])),
            ts_map.and_then(|m| m.get(&MapsTableKind::Npcs)),
            resizing,
        );

        // Extra objects table
        content = content.push(section_header(&format!(
            "Extra Objects ({})",
            map.extra_objects.len()
        )));
        content = push_map_table(
            content,
            idx,
            MapsTableKind::ExtraObjects,
            caches.map(|c| (&c.extra_objects[..], &c.extra_objects_indices[..])),
            ts_map.and_then(|m| m.get(&MapsTableKind::ExtraObjects)),
            resizing,
        );

        // Ground items
        content = content.push(section_header("Ground Items"));

        // Weapon items
        content = content.push(subsection_header(&format!(
            "Weapons ({})",
            map.draw_items_weapon.len()
        )));
        content = push_map_table(
            content,
            idx,
            MapsTableKind::Weapon,
            caches.map(|c| (&c.draw_items_weapon[..], &c.draw_items_weapon_indices[..])),
            ts_map.and_then(|m| m.get(&MapsTableKind::Weapon)),
            resizing,
        );

        // Heal items
        content = content.push(subsection_header(&format!(
            "Heals ({})",
            map.draw_items_heal.len()
        )));
        content = push_map_table(
            content,
            idx,
            MapsTableKind::Heal,
            caches.map(|c| (&c.draw_items_heal[..], &c.draw_items_heal_indices[..])),
            ts_map.and_then(|m| m.get(&MapsTableKind::Heal)),
            resizing,
        );

        // Edit items
        content = content.push(subsection_header(&format!(
            "Edits ({})",
            map.draw_items_edit.len()
        )));
        content = push_map_table(
            content,
            idx,
            MapsTableKind::Edit,
            caches.map(|c| (&c.draw_items_edit[..], &c.draw_items_edit_indices[..])),
            ts_map.and_then(|m| m.get(&MapsTableKind::Edit)),
            resizing,
        );

        // Misc items
        content = content.push(subsection_header(&format!(
            "Misc ({})",
            map.draw_items_misc.len()
        )));
        content = push_map_table(
            content,
            idx,
            MapsTableKind::Misc,
            caches.map(|c| (&c.draw_items_misc[..], &c.draw_items_misc_indices[..])),
            ts_map.and_then(|m| m.get(&MapsTableKind::Misc)),
            resizing,
        );

        // Event items
        content = content.push(subsection_header(&format!(
            "Events ({})",
            map.draw_items_event.len()
        )));
        content = push_map_table(
            content,
            idx,
            MapsTableKind::Event,
            caches.map(|c| (&c.draw_items_event[..], &c.draw_items_event_indices[..])),
            ts_map.and_then(|m| m.get(&MapsTableKind::Event)),
            resizing,
        );

        scrollable(content).height(Fill).into()
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

/// Push one entity table (or an empty/placeholder note) into the column.
fn push_map_table<'a>(
    content: Column<'a, Message>,
    map_idx: usize,
    kind: MapsTableKind,
    data: Option<(&'a [Vec<String>], &'a [usize])>,
    ts: Option<&'a MapTableState>,
    resizing: Option<(usize, MapsTableKind)>,
) -> Column<'a, Message> {
    match (data, ts) {
        (Some((rows, indices)), Some(ts)) if !rows.is_empty() => {
            content.push(entity_table(map_idx, kind, rows, indices, ts, resizing))
        }
        (Some(_), Some(_)) => content.push(empty_text("(none)")),
        _ => content.push(empty_text("(caches not ready)")),
    }
}

fn entity_table<'a>(
    map_idx: usize,
    kind: MapsTableKind,
    display_cache: &'a [Vec<String>],
    indices: &'a [usize],
    ts: &'a MapTableState,
    resizing: Option<(usize, MapsTableKind)>,
) -> Element<'a, Message> {
    // Build columns from the kind's default layout, then apply the
    // per-table width overrides, active sort state, and column-filter badges.
    let mut columns = kind.default_columns();
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

    let key = TableKey::Map(map_idx, kind);
    let msg_fn = move |action: TableFilterAction| {
        Message::save_file_viewer(SaveFileViewerMessage::TableFilter { key, action })
    };

    let table = TableWidget::new(
        display_cache,
        indices,
        columns,
        0.0,
        row_flags,
        22.0,
        ParagraphCache::default(),
    )
    .external_offset(ts.scroll_offset.0, ts.scroll_offset.1)
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
    let table_elem: Element<'a, Message> =
        container(table).height(Length::Fixed(640.0)).into();

    // While resizing this table, capture cursor moves / release across the
    // whole table area so the drag isn't interrupted by the inner widget.
    let table_elem: Element<'a, Message> = if resizing == Some((map_idx, kind)) {
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

    let filter_bar =
        filter_modal::build_filter_bar(filter, display_cache.len(), indices.len(), msg_fn);
    let wrapped = Column::<Message>::new()
        .push(filter_bar)
        .push(table_elem)
        .spacing(4);

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

fn section_header(label: &str) -> Element<'static, Message> {
    container(text(label.to_string()).size(16))
        .padding(Padding {
            top: 12.0,
            right: 0.0,
            bottom: 4.0,
            left: 0.0,
        })
        .width(Fill)
        .into()
}

fn subsection_header(label: &str) -> Element<'static, Message> {
    container(text(label.to_string()).size(13))
        .padding(Padding {
            top: 8.0,
            right: 0.0,
            bottom: 2.0,
            left: 8.0,
        })
        .width(Fill)
        .into()
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
