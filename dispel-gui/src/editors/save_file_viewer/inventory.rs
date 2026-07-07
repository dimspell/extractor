use iced::widget::{button, container, scrollable, text, Column, Row};
use iced::{Element, Fill};

use crate::editors::save_file_viewer::state::{InventoryCategory, SaveFileViewerState};
use crate::message::Message;
use crate::message::MessageExt;

/// Inventory section: category buttons + raw hex.
pub fn view<'a>(state: &'a SaveFileViewerState) -> Element<'a, Message> {
    let sf = match state.save_file.as_ref() {
        Some(sf) => sf,
        None => return container(text("No save file loaded")).into(),
    };

    let inv = &sf.inventory;
    let active = state.inventory_category;

    let categories = [
        ("Event Items", InventoryCategory::Event, inv.event_items.len(), 244),
        ("Misc Items", InventoryCategory::Misc, inv.misc_items.len(), 264),
        ("Edit Items", InventoryCategory::Edit, inv.edit_items.len(), 272),
        ("Weapon Items", InventoryCategory::Weapon, inv.weapon_items.len(), 292),
        ("Heal Items", InventoryCategory::Heal, inv.heal_items.len(), 256),
    ];

    // Category buttons row
    let buttons: Vec<Element<'a, Message>> = categories
        .iter()
        .map(|(label, cat, total_bytes, record_size)| {
            let count = if *total_bytes > 0 {
                total_bytes / record_size
            } else {
                0
            };
            let is_active = active == Some(*cat);
            let mut btn = button(text(format!(
                "{} ({} rec, {}B)",
                label, count, record_size
            )).size(12));
            if is_active {
                btn = btn.style(iced::widget::button::primary);
            }
            btn.on_press(Message::save_file_viewer(
                crate::editors::save_file_viewer::SaveFileViewerMessage::SelectCategory(*cat),
            ))
            .padding([4, 8])
            .into()
        })
        .collect();

    // Content
    let body: Element<'a, Message> = match active {
        Some(cat) => {
            let data = match cat {
                InventoryCategory::Event => &inv.event_items,
                InventoryCategory::Misc => &inv.misc_items,
                InventoryCategory::Edit => &inv.edit_items,
                InventoryCategory::Weapon => &inv.weapon_items,
                InventoryCategory::Heal => &inv.heal_items,
            };
            if data.is_empty() {
                container(text("(empty)").color(iced::Color::from_rgb(0.5, 0.5, 0.5)))
                    .width(Fill)
                    .height(Fill)
                    .padding(16)
                    .into()
            } else {
                // Show raw hex of first 256 bytes as a compact dump
                let preview = hex_preview(data);
                scrollable(
                    Column::new()
                        .push(text(format!(
                            "{} bytes ({} records)",
                            data.len(),
                            data.len() / category_record_size(cat)
                        )).size(13))
                        .push(text(preview).size(11))
                        .spacing(8)
                        .padding(16),
                )
                .into()
            }
        }
        None => container(text("Select a category above"))
            .width(Fill)
            .height(Fill)
            .padding(16)
            .into(),
    };

    Column::new()
        .push(Row::new().spacing(4).padding(8).extend(buttons))
        .push(body)
        .into()
}

fn category_record_size(cat: InventoryCategory) -> usize {
    match cat {
        InventoryCategory::Event => 244,
        InventoryCategory::Misc => 264,
        InventoryCategory::Edit => 272,
        InventoryCategory::Weapon => 292,
        InventoryCategory::Heal => 256,
    }
}

fn hex_preview(data: &[u8]) -> String {
    let max = 256.min(data.len());
    data[..max]
        .chunks(16)
        .map(|chunk| {
            let hex: String = chunk.iter().map(|b| format!("{:02x} ", b)).collect();
            let ascii: String = chunk
                .iter()
                .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' })
                .collect();
            format!("{:48}  {}", hex, ascii)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
