use std::rc::Rc;

use dispel_core::ItemTypeId;
use iced::widget::{column as col, pick_list, row, text, text_input};
use iced::{Element, Length};

use crate::message::Message;
use crate::style;

/// Build a cascading item-type + item-id picker for `CompositeItem` fields.
///
/// Renders two pick-lists (type then item) plus a read-only `id_field` label.
/// Falls back to a plain `text_input` when `entries` is `None` (lookups
/// unavailable).
pub fn composite_item_picker(
    label: &'static str,
    value: &str,
    id_field: &'static str,
    entries: Option<&[(String, String)]>,
    on_change: impl Fn(String) -> Message + 'static,
) -> Element<'static, Message> {
    const LABEL_W: f32 = 140.0;

    let current_type_byte: u8 = value
        .split(':')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(255);
    let current_type = ItemTypeId::from_u8(current_type_byte).unwrap_or(ItemTypeId::Other);
    let current_id = value.split(':').nth(1).unwrap_or("0");

    let on_change: Rc<dyn Fn(String) -> Message> = Rc::new(on_change);

    let Some(entries) = entries else {
        let oc = on_change;
        return row![
            text(label)
                .size(11)
                .width(LABEL_W)
                .style(style::subtle_text),
            text_input("", value)
                .on_input(move |v| oc(v))
                .padding(4)
                .size(11)
                .width(Length::Fill),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .into();
    };

    // Deduplicated type list, preserving insertion order
    let item_types: Vec<ItemTypeId> = {
        let mut seen = Vec::new();
        let mut result = Vec::new();
        for (key, _) in entries {
            let tb: u8 = key
                .split(':')
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(255);
            if !seen.contains(&tb) {
                seen.push(tb);
                if let Some(ty) = ItemTypeId::from_u8(tb) {
                    result.push(ty);
                }
            }
        }
        result
    };

    let type_labels: Vec<String> = item_types.iter().map(|t| t.to_string()).collect();
    let current_type_label = current_type.to_string();

    let filtered_items: Vec<(String, String)> = entries
        .iter()
        .filter(|(key, _)| {
            key.split(':').next().and_then(|s| s.parse::<u8>().ok()) == Some(current_type_byte)
        })
        .cloned()
        .collect();

    let selected_item = filtered_items
        .iter()
        .find(|(key, _)| *key == value)
        .map(|(_, name)| name.clone());

    let item_options: Vec<String> = filtered_items
        .iter()
        .map(|(_, name)| name.clone())
        .collect();

    // Clone type_labels before passing to pick_list (closure needs the data too)
    let type_labels_for_picker = type_labels.clone();

    let type_picker: Element<'static, Message> = {
        let oc = on_change.clone();
        pick_list(
            type_labels,
            Some(current_type_label),
            move |selected_label| {
                let type_byte = type_labels_for_picker
                    .iter()
                    .position(|l| l == &selected_label)
                    .and_then(|i| item_types.get(i))
                    .map(|t| u8::from(*t))
                    .unwrap_or(255);
                oc(type_byte.to_string())
            },
        )
        .width(Length::Fill)
        .into()
    };

    let item_picker: Element<'static, Message> = {
        pick_list(item_options, selected_item, move |selected_name| {
            let composite_key = filtered_items
                .iter()
                .find(|(_, name)| name == &selected_name)
                .map(|(key, _)| key.clone())
                .unwrap_or_default();
            on_change(composite_key)
        })
        .width(Length::Fill)
        .into()
    };

    col![
        type_picker,
        item_picker,
        text(format!("{}: {}", id_field, value))
            .size(10)
            .style(style::subtle_text)
    ]
    .spacing(4)
    .into()
}
