use iced::widget::{button, container, scrollable, text, Column};
use iced::{Element, Fill};

use crate::editors::save_file_viewer::state::{InventoryCategory, SaveFileViewerState};
use crate::editors::save_file_viewer::SaveFileViewerMessage;
use crate::message::Message;
use crate::message::MessageExt;
use gui_widgets::components::paragraph_cache::ParagraphCache;
use gui_widgets::{TableColumn, TableWidget};

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
    let (columns, display_cache, filtered_indices): (
        Vec<TableColumn>,
        Option<&Vec<Vec<String>>>,
        Option<&Vec<usize>>,
    ) = match cat {
        InventoryCategory::Weapon => (
            vec![
                TableColumn { width_px: 160.0, label: "Name".into(), sort: None, has_filter: false },
                TableColumn { width_px: 55.0, label: "Price".into(), sort: None, has_filter: false },
                TableColumn { width_px: 42.0, label: "Atk".into(), sort: None, has_filter: false },
                TableColumn { width_px: 42.0, label: "Def".into(), sort: None, has_filter: false },
                TableColumn { width_px: 55.0, label: "MagStr".into(), sort: None, has_filter: false },
                TableColumn { width_px: 42.0, label: "Dur".into(), sort: None, has_filter: false },
                TableColumn { width_px: 50.0, label: "ReqStr".into(), sort: None, has_filter: false },
                TableColumn { width_px: 50.0, label: "ReqAgi".into(), sort: None, has_filter: false },
                TableColumn { width_px: 50.0, label: "ReqWis".into(), sort: None, has_filter: false },
                TableColumn { width_px: 42.0, label: "HP".into(), sort: None, has_filter: false },
                TableColumn { width_px: 42.0, label: "MP".into(), sort: None, has_filter: false },
            ],
            state.inventory_display_caches.get(&InventoryCategory::Weapon),
            state.inventory_filtered_indices.get(&InventoryCategory::Weapon),
        ),
        InventoryCategory::Heal => (
            vec![
                TableColumn { width_px: 160.0, label: "Name".into(), sort: None, has_filter: false },
                TableColumn { width_px: 55.0, label: "Price".into(), sort: None, has_filter: false },
                TableColumn { width_px: 42.0, label: "HP".into(), sort: None, has_filter: false },
                TableColumn { width_px: 42.0, label: "MP".into(), sort: None, has_filter: false },
                TableColumn { width_px: 52.0, label: "FullHP".into(), sort: None, has_filter: false },
                TableColumn { width_px: 52.0, label: "FullMP".into(), sort: None, has_filter: false },
                TableColumn { width_px: 62.0, label: "CurePois".into(), sort: None, has_filter: false },
                TableColumn { width_px: 62.0, label: "CurePetr".into(), sort: None, has_filter: false },
            ],
            state.inventory_display_caches.get(&InventoryCategory::Heal),
            state.inventory_filtered_indices.get(&InventoryCategory::Heal),
        ),
        InventoryCategory::Edit => (
            vec![
                TableColumn { width_px: 160.0, label: "Name".into(), sort: None, has_filter: false },
                TableColumn { width_px: 55.0, label: "Price".into(), sort: None, has_filter: false },
                TableColumn { width_px: 42.0, label: "HP".into(), sort: None, has_filter: false },
                TableColumn { width_px: 42.0, label: "MP".into(), sort: None, has_filter: false },
                TableColumn { width_px: 38.0, label: "Str".into(), sort: None, has_filter: false },
                TableColumn { width_px: 38.0, label: "Agi".into(), sort: None, has_filter: false },
                TableColumn { width_px: 38.0, label: "Wis".into(), sort: None, has_filter: false },
                TableColumn { width_px: 38.0, label: "Con".into(), sort: None, has_filter: false },
                TableColumn { width_px: 38.0, label: "Off".into(), sort: None, has_filter: false },
                TableColumn { width_px: 38.0, label: "Def".into(), sort: None, has_filter: false },
                TableColumn { width_px: 50.0, label: "MagPwr".into(), sort: None, has_filter: false },
            ],
            state.inventory_display_caches.get(&InventoryCategory::Edit),
            state.inventory_filtered_indices.get(&InventoryCategory::Edit),
        ),
        InventoryCategory::Event => (
            vec![
                TableColumn { width_px: 160.0, label: "Name".into(), sort: None, has_filter: false },
                TableColumn { width_px: 55.0, label: "Price".into(), sort: None, has_filter: false },
                TableColumn { width_px: 70.0, label: "EventID".into(), sort: None, has_filter: false },
            ],
            state.inventory_display_caches.get(&InventoryCategory::Event),
            state.inventory_filtered_indices.get(&InventoryCategory::Event),
        ),
        InventoryCategory::Misc => (
            vec![
                TableColumn { width_px: 160.0, label: "Name".into(), sort: None, has_filter: false },
                TableColumn { width_px: 55.0, label: "Price".into(), sort: None, has_filter: false },
            ],
            state.inventory_display_caches.get(&InventoryCategory::Misc),
            state.inventory_filtered_indices.get(&InventoryCategory::Misc),
        ),
    };

    let display_cache = match display_cache {
        Some(c) if !c.is_empty() => c,
        _ => {
            return container(text("(empty)"))
                .width(Fill)
                .height(Fill)
                .padding(16)
                .into();
        }
    };

    let filtered_indices = match filtered_indices {
        Some(i) => i,
        None => {
            return container(text("(empty)"))
                .width(Fill)
                .height(Fill)
                .padding(16)
                .into();
        }
    };

    scrollable(
        TableWidget::new(
            display_cache,
            filtered_indices,
            columns,
            0.0,
            |_| gui_widgets::RowFlags::default(),
            22.0,
            ParagraphCache::default(),
        ),
    )
    .height(Fill)
    .into()
}
