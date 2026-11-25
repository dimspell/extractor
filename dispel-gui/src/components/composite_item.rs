use std::rc::Rc;

use dispel_core::ItemTypeId;
use iced::widget::{column as col, pick_list, row, text, text_input};
use iced::{Element, Length};

use crate::message::Message;
use crate::style;

const LABEL_W: f32 = 140.0;

/// The empty-slot option shared with the item catalog.
const NO_ITEM_KEY: &str = "255:15";
const NO_ITEM_LABEL: &str = "[-]";

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

    // Parallel label/byte vectors, so a displayed type always maps back to the
    // byte the setter needs. Insert the current type when the lookup omits it.
    let mut type_labels: Vec<String> = Vec::new();
    let mut type_bytes: Vec<u8> = Vec::new();
    for (key, _) in entries {
        let tb: u8 = key
            .split(':')
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(255);
        if !type_bytes.contains(&tb)
            && let Some(ty) = ItemTypeId::from_u8(tb)
        {
            type_bytes.push(tb);
            type_labels.push(ty.to_string());
        }
    }
    let current_type_label = current_type.to_string();
    if !type_bytes.contains(&current_type_byte) {
        type_bytes.push(current_type_byte);
        type_labels.push(current_type_label.clone());
    }

    let mut filtered_items: Vec<(String, String)> = entries
        .iter()
        .filter(|(key, _)| {
            key.split(':').next().and_then(|s| s.parse::<u8>().ok()) == Some(current_type_byte)
        })
        .cloned()
        .collect();

    // Every key with the no-item byte denotes the same empty slot, regardless
    // of its id. Resolve all of them to the shared sentinel option so the
    // selection is never blank.
    let is_no_item = current_type_byte == u8::from(ItemTypeId::NoItem);
    if is_no_item && !filtered_items.iter().any(|(key, _)| key == NO_ITEM_KEY) {
        filtered_items.push((NO_ITEM_KEY.to_string(), NO_ITEM_LABEL.to_string()));
    }
    let lookup_key = if is_no_item { NO_ITEM_KEY } else { value };

    let mut selected_item = filtered_items
        .iter()
        .find(|(key, _)| *key == lookup_key)
        .map(|(_, name)| name.clone());

    let mut labelled_keys: Vec<(String, String)> = filtered_items
        .iter()
        .map(|(key, name)| (name.clone(), key.clone()))
        .collect();

    // A non-canonical no-item key still renders as the shared "[-]" option,
    // but re-selecting it must emit the exact stored key to keep the record
    // byte-for-byte intact.
    if is_no_item && value != NO_ITEM_KEY {
        for (label, key) in &mut labelled_keys {
            if label == NO_ITEM_LABEL {
                *key = value.to_string();
            }
        }
    }

    // An id that is gone from the lookup still gets a visible, selectable
    // label. The stored value is left untouched; choosing the fallback keeps
    // the original key.
    if selected_item.is_none() {
        let fallback_label = format!("Unknown ({})", current_id);
        labelled_keys.push((fallback_label.clone(), value.to_string()));
        selected_item = Some(fallback_label);
    }

    let item_options: Vec<String> = labelled_keys
        .iter()
        .map(|(label, _)| label.clone())
        .collect();

    let type_picker: Element<'static, Message> = {
        let oc = on_change.clone();
        let current_id = current_id.clone();
        let labels = type_labels.clone();
        let bytes = type_bytes.clone();
        let entries_owned: Vec<(String, String)> = entries.to_vec();
        pick_list(Some(current_type_label), type_labels, |v| v.to_string())
            .on_select(move |selected_label| {
                let new_byte = labels
                    .iter()
                    .position(|l| l == &selected_label)
                    .and_then(|i| bytes.get(i).copied())
                    .unwrap_or_else(|| u8::from(ItemTypeId::NoItem));
                let ids: Vec<String> = entries_owned
                    .iter()
                    .filter(|(key, _)| {
                        key.split(':').next().and_then(|s| s.parse::<u8>().ok()) == Some(new_byte)
                    })
                    .map(|(key, _)| key.split(':').nth(1).unwrap_or("0").to_string())
                    .collect();
                // Prefer the current id when the new type offers it, otherwise
                // use the first entry of that type, or the shared empty slot.
                let key = if ids.contains(&current_id) {
                    format!("{}:{}", new_byte, current_id)
                } else if let Some(id) = ids.first() {
                    format!("{}:{}", new_byte, id)
                } else {
                    NO_ITEM_KEY.to_string()
                };
                oc(key)
            })
            .width(Length::Fill)
            .into()
    };

    let item_picker: Element<'static, Message> = {
        let fallback_key = value.to_string();
        pick_list(selected_item, item_options, |v| v.to_string())
            .on_select(move |selected_name| {
                let composite_key = labelled_keys
                    .iter()
                    .find(|(label, _)| *label == selected_name)
                    .map(|(_, key)| key.clone())
                    .unwrap_or_else(|| fallback_key.clone());
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
    fn test_unmapped_composite_keys_render_a_label_and_keep_value() {
        let entries = [
            ("1:0".to_string(), "Invented Sword".to_string()),
            ("255:15".to_string(), "[-]".to_string()),
        ];
        let element = |value: &'static str| {
            composite_item_picker("Invented Item", value, Some(&entries), false, |v| {
                Message::FileTree(crate::message::FileTreeMessage::Search(v))
            })
        };

        for value in ["255:0", "255:255"] {
            let values = crate::tests::common::select_values(element(value));
            assert!(
                values.iter().any(|v| v == "[-]"),
                "{value} must resolve to the no-item sentinel label, got {values:?}"
            );
        }

        let values = crate::tests::common::select_values(element("1:999"));
        assert!(
            values.iter().any(|v| v.starts_with("Unknown (999)")),
            "an unmapped item id must render a visible fallback label, got {values:?}"
        );

        let messages = crate::tests::common::activate_select_option(element("255:0"), 1, 0);
        let emitted: Vec<String> = messages
            .into_iter()
            .filter_map(|m| match m {
                Message::FileTree(crate::message::FileTreeMessage::Search(v)) => Some(v),
                _ => None,
            })
            .collect();
        assert!(
            emitted.iter().any(|v| v == "255:0"),
            "choosing the sentinel must keep the exact stored key, got {emitted:?}"
        );
    }

    #[test]
    fn test_no_item_selection_preserves_exact_key() {
        use dispel_core::InventoryItem;

        let entries = [
            ("1:0".to_string(), "Invented Sword".to_string()),
            ("255:15".to_string(), "[-]".to_string()),
        ];
        let element = |value: &'static str| {
            composite_item_picker("Invented Item", value, Some(&entries), false, |v| {
                Message::FileTree(crate::message::FileTreeMessage::Search(v))
            })
        };
        let emitted_for = |value: &'static str| -> String {
            let messages = crate::tests::common::activate_select_option(element(value), 1, 0);
            messages
                .into_iter()
                .find_map(|m| match m {
                    Message::FileTree(crate::message::FileTreeMessage::Search(v)) => Some(v),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("no value emitted for {value}"))
        };

        for value in ["255:255", "255:5"] {
            let values = crate::tests::common::select_values(element(value));
            assert!(
                values.iter().any(|v| v == "[-]"),
                "{value} must render the shared no-item label, got {values:?}"
            );
        }

        assert_eq!(emitted_for("255:255"), "255:255");
        assert_eq!(emitted_for("255:5"), "255:5");
        assert_eq!(emitted_for("255:15"), "255:15");

        let emitted = emitted_for("255:255");
        let parts: Vec<&str> = emitted.split(':').collect();
        let item = InventoryItem::new(
            ItemTypeId::from_u8(parts[0].parse().unwrap()).unwrap(),
            parts[1].parse().unwrap(),
        );
        assert!(
            item.is_empty(),
            "255:255 must decode to an empty inventory item"
        );
    }

    #[test]
    fn test_composite_type_picker_keeps_current_type_and_selects_valid_id() {
        // The lookup knows only type 2; the record currently uses type 3.
        let entries = [("2:5".to_string(), "Invented Potion".to_string())];
        let element = || {
            composite_item_picker("Invented Item", "3:7", Some(&entries), false, |v| {
                Message::FileTree(crate::message::FileTreeMessage::Search(v))
            })
        };

        let values = crate::tests::common::select_values(element());
        assert!(
            values.first().is_some_and(|v| v == "Edit"),
            "the type picker must show the current type, got {values:?}"
        );

        // "Edit" is inserted after "Healing", so move the highlight up once.
        let messages = crate::tests::common::activate_select_option(element(), 0, -1);
        let emitted: Vec<String> = messages
            .into_iter()
            .filter_map(|m| match m {
                Message::FileTree(crate::message::FileTreeMessage::Search(v)) => Some(v),
                _ => None,
            })
            .collect();
        assert!(
            emitted.iter().any(|v| v == "2:5"),
            "switching type must choose an id present in the new type, got {emitted:?}"
        );
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
