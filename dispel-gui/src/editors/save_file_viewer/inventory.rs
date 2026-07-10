use iced::widget::{button, container, scrollable, text, Column, Row};
use iced::{Element, Fill, Length};

use crate::editors::save_file_viewer::state::{InventoryCategory, SaveFileViewerState};
use crate::editors::save_file_viewer::SaveFileViewerMessage;
use crate::message::Message;
use crate::message::MessageExt;

/// Inventory section: category buttons + embedded hex viewer per category.
pub fn view<'a>(state: &'a SaveFileViewerState) -> Element<'a, Message> {
    let categories = [
        InventoryCategory::Event,
        InventoryCategory::Misc,
        InventoryCategory::Edit,
        InventoryCategory::Weapon,
        InventoryCategory::Heal,
    ];

    let active = state.inventory_category;

    // Category buttons row
    let mut buttons = iced::widget::Row::<Message>::new().spacing(4).padding(8);
    for cat in &categories {
        let is_active = active == Some(*cat);
        let count = state
            .inventory_hex_viewers
            .get(cat)
            .map(|e| e.provider.as_slice().len())
            .unwrap_or(0);
        let rec_count = if count > 0 {
            count / cat.record_size()
        } else {
            0
        };
        let label = format!("{} ({} rec, {}B)", cat.label(), rec_count, cat.record_size());
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

    // Content: embedded hex editor for the selected category
    let body: Element<'a, Message> = match active {
        Some(cat) => {
            if let Some(editor) = state.inventory_hex_viewers.get(&cat) {
                // TODO: Draw a table here (not the hex viewer) that list all the items in the inventory
                container(text("todo")).into()
            } else {
                container(text("Hex viewer not available"))
                    .width(Fill)
                    .height(Fill)
                    .padding(16)
                    .into()
            }
        }
        None => container(text("Select a category above"))
            .width(Fill)
            .height(Fill)
            .padding(16)
            .into(),
    };

    Column::<Message>::new()
        .push(buttons)
        .push(body)
        .into()
}
