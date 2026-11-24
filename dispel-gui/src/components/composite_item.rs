use std::rc::Rc;

use dispel_core::ItemTypeId;
use iced::widget::{column as col, pick_list, row, text, text_input};
use iced::{Element, Length};

use crate::message::Message;
use crate::style;

const LABEL_W: f32 = 140.0;

/// Render a picker body, optionally prefixed with a fixed-width label column.
///
/// Callers that already render a label above the field pass `show_label = false`
/// so the text is not drawn twice.
fn with_optional_label<'a>(
    label: &'static str,
    body: Element<'a, Message>,
    show_label: bool,
) -> Element<'a, Message> {
    if show_label {
        row![
            text(label)
                .size(11)
                .width(LABEL_W)
                .style(style::subtle_text),
            body,
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .into()
    } else {
        body
    }
}

/// Build a cascading item-type + item-id picker for `CompositeItem` fields.
///
/// Renders two pick-lists (type then item). Falls back to a plain `text_input`
/// when `entries` is `None` (lookups unavailable). `show_label` controls whether
/// the label column is drawn by this component or by the caller.
pub fn composite_item_picker(
    label: &'static str,
    value: &str,
    entries: Option<&[(String, String)]>,
    show_label: bool,
    on_change: impl Fn(String) -> Message + 'static,
) -> Element<'static, Message> {
    let current_type_byte: u8 = value
        .split(':')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(255);
    let current_type = ItemTypeId::from_u8(current_type_byte).unwrap_or(ItemTypeId::NoItem);
    let current_id = value.split(':').nth(1).unwrap_or("0").to_string();

    let on_change: Rc<dyn Fn(String) -> Message> = Rc::new(on_change);

    let Some(entries) = entries else {
        let oc = on_change;
        let input: Element<'static, Message> = text_input("", value.to_owned())
            .on_input(move |v| oc(v))
            .padding(4)
            .size(11)
            .width(Length::Fill)
            .into();
        return with_optional_label(label, input, show_label);
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
        let current_id = current_id.clone();
        pick_list(Some(current_type_label), type_labels, |v| v.to_string())
            .on_select(move |selected_label| {
                let type_byte = type_labels_for_picker
                    .iter()
                    .position(|l| l == &selected_label)
                    .and_then(|i| item_types.get(i))
                    .map(|t| u8::from(*t))
                    .unwrap_or(255);
                // Preserve the item ID when switching type so the user doesn't
                // lose their selection. The setter macro receives the full
                // composite key and updates both fields.
                oc(format!("{}:{}", type_byte, current_id))
            })
            .width(Length::Fill)
            .into()
    };

    let item_picker: Element<'static, Message> = {
        pick_list(selected_item, item_options, |v| v.to_string())
            .on_select(move |selected_name| {
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

    let pickers: Element<'static, Message> = col![type_picker, item_picker].spacing(4).into();
    with_optional_label(label, pickers, show_label)
}

#[cfg(all(test, feature = "iced_test"))]
mod tests {
    use super::*;
    use iced_test::simulator;

    fn label_rendered(entries: Option<&[(String, String)]>, show_label: bool) -> bool {
        let element: Element<'_, Message> =
            composite_item_picker("Invented Item", "1:0", entries, show_label, |_| {
                Message::FileTree(crate::message::FileTreeMessage::Search(String::new()))
            });
        let mut ui = simulator(element);
        ui.find("Invented Item").is_ok()
    }

    #[test]
    fn test_composite_picker_label_follows_show_label() {
        let entries = [("1:0".to_string(), "Invented Sword".to_string())];
        for show_label in [true, false] {
            assert_eq!(
                label_rendered(None, show_label),
                show_label,
                "fallback picker label must follow show_label"
            );
            assert_eq!(
                label_rendered(Some(&entries), show_label),
                show_label,
                "lookup picker label must follow show_label"
            );
        }
    }
}
