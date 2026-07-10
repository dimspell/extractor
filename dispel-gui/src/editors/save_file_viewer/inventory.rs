use iced::mouse::Interaction;
use iced::widget::{button, container, mouse_area, scrollable, text, Column};
use iced::{Element, Fill};

use crate::editors::save_file_viewer::state::{InventoryCategory, SaveFileViewerState};
use crate::editors::save_file_viewer::SaveFileViewerMessage;
use crate::message::Message;
use crate::message::MessageExt;
use gui_widgets::components::paragraph_cache::ParagraphCache;
use gui_widgets::{RowFlags, TableWidget};

/// Inventory section: category buttons + TableWidget per category.
pub fn view<'a>(state: &'a SaveFileViewerState) -> Element<'a, Message> {
    let categories = [
        InventoryCategory::Weapon,
        InventoryCategory::Heal,
        InventoryCategory::Edit,
        InventoryCategory::Event,
        InventoryCategory::Misc,
    ];

    let active = state.inventory_category;

    // Category buttons row
    let mut buttons = iced::widget::Row::<Message>::new().spacing(4).padding(8);
    for cat in &categories {
        let is_active = active == Some(*cat);
        let count = state
            .inventory_display_caches
            .get(cat)
            .map(|c| c.len())
            .unwrap_or(0);
        let label = format!("{} ({})", cat.label(), count);
        let mut btn = button(text(label).size(12));
        if is_active {
            btn = btn.style(iced::widget::button::primary);
        }
        buttons = buttons.push(
            btn.on_press(Message::save_file_viewer(
                SaveFileViewerMessage::SelectCategory(*cat),
            ))
            .padding([4, 8]),
        );
    }

    // Content: TableWidget for the selected category
    let body: Element<'a, Message> = match active {
        Some(cat) => inventory_table(state, cat),
        None => container(text("Select a category above"))
            .width(Fill)
            .height(Fill)
            .padding(16)
            .into(),
    };

    Column::<Message>::new().push(buttons).push(body).into()
}

fn inventory_table<'a>(
    state: &'a SaveFileViewerState,
    cat: InventoryCategory,
) -> Element<'a, Message> {
    let ts = state.inventory_table_states.get(&cat);
    let resizing = state.inventory_resizing.as_ref().map(|d| d.cat);

    // Build columns from the category's default layout, then apply the
    // per-table width overrides and active sort state.
    let mut columns = cat.default_columns();
    if let Some(ts) = ts {
        for (c, w) in columns.iter_mut().zip(&ts.column_widths) {
            c.width_px = *w;
        }
        if let Some(sc) = ts.sort_column {
            if let Some(c) = columns.get_mut(sc) {
                c.sort = Some(ts.sort_ascending);
            }
        }
    }

    let display_cache = match state.inventory_display_caches.get(&cat) {
        Some(c) if !c.is_empty() => c,
        _ => {
            return container(text("(empty)"))
                .width(Fill)
                .height(Fill)
                .padding(16)
                .into();
        }
    };

    let filtered_indices = match state.inventory_filtered_indices.get(&cat) {
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
    let scroll = ts.map(|t| t.scroll_offset).unwrap_or((0.0, 0.0));
    let row_flags = move |visible_idx: usize| -> RowFlags {
        let orig = filtered_indices.get(visible_idx).copied();
        RowFlags {
            selected: orig == selected,
            ..Default::default()
        }
    };

    let table = TableWidget::new(
        display_cache,
        filtered_indices,
        columns,
        0.0,
        row_flags,
        22.0,
        ParagraphCache::default(),
    )
    .external_offset(scroll.0, scroll.1)
    .on_select(move |visible_idx| {
        Message::save_file_viewer(SaveFileViewerMessage::InventoryTableSelect {
            cat,
            visible_idx,
        })
    })
    .on_sort(move |col| {
        Message::save_file_viewer(SaveFileViewerMessage::InventoryTableSort { cat, col })
    })
    .on_start_resize(move |col| {
        Message::save_file_viewer(SaveFileViewerMessage::InventoryTableStartResize { cat, col })
    })
    .on_reset_column_width(move |col| {
        Message::save_file_viewer(SaveFileViewerMessage::InventoryTableResetColumnWidth {
            cat,
            col,
        })
    })
    .on_scroll(move |x, y, vh| {
        Message::save_file_viewer(SaveFileViewerMessage::InventoryTableScroll {
            cat,
            x,
            y,
            viewport_height: vh,
        })
    });

    // While resizing this table, capture cursor moves / release across the
    // whole table area so the drag isn't interrupted by the inner widget.
    let table_elem: Element<'a, Message> = if resizing == Some(cat) {
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

    scrollable(table_elem).height(Fill).into()
}
