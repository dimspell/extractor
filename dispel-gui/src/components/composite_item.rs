use std::rc::Rc;

use iced::widget::{column as col, pick_list, row, text, text_input};
use iced::{Element, Length};

use crate::components::select::lookup_select;
use crate::message::Message;
use crate::style;

const LABEL_W: f32 = 140.0;

/// The empty-slot option shared with the item catalog.
const NO_ITEM_KEY: &str = "255:15";

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

/// Build a searchable item picker for `CompositeItem` fields.
///
/// All item files share one select: every entry carries its type in the
/// display label (`[Weapon] Cap`, `[Healing] Herb`, …). A text input above the
/// select filters those labels and composite keys. Falls back to a plain
/// `text_input` when `entries` is `None` (lookups unavailable). `show_label`
/// controls whether the label column is drawn by this component or the caller.
pub fn composite_item_picker(
    label: &'static str,
    value: &str,
    entries: Option<&[(String, String)]>,
    show_label: bool,
    filter: &str,
    on_filter: impl Fn(String) -> Message + 'static,
    on_change: impl Fn(String) -> Message + 'static,
) -> Element<'static, Message> {
    let filter = filter.to_owned();
    let on_filter: Rc<dyn Fn(String) -> Message> = Rc::new(on_filter);
    let on_change: Rc<dyn Fn(String) -> Message> = Rc::new(on_change);

    let search_input: Element<'static, Message> = {
        let of = on_filter.clone();
        text_input("Search items...", filter.clone())
            .on_input(move |v| of(v))
            .padding(4)
            .size(11)
            .width(Length::Fill)
            .into()
    };

    let Some(entries) = entries else {
        let oc = on_change;
        let input: Element<'static, Message> = text_input("", value.to_owned())
            .on_input(move |v| oc(v))
            .padding(4)
            .size(11)
            .width(Length::Fill)
            .into();
        let body: Element<'static, Message> = col![search_input, input].spacing(4).into();
        return with_optional_label(label, body, show_label);
    };

    // One option per item file. Every key with the no-item byte denotes the
    // same empty slot, so only the canonical sentinel survives.
    let mut collapsed: Vec<(String, String)> = Vec::with_capacity(entries.len());
    for (key, display) in entries {
        if key.starts_with("255:") {
            if key == NO_ITEM_KEY && !collapsed.iter().any(|(k, _)| k == key) {
                collapsed.push((key.clone(), display.clone()));
            }
        } else {
            collapsed.push((key.clone(), display.clone()));
        }
    }
    if !collapsed.iter().any(|(key, _)| key == NO_ITEM_KEY) {
        collapsed.push((NO_ITEM_KEY.to_string(), "[-]".to_string()));
    }

    let is_no_item = value.starts_with("255:");
    let query = filter.to_lowercase();
    let mut filtered_entries: Vec<(String, String)> = collapsed
        .iter()
        .filter(|(key, display)| {
            display.to_lowercase().contains(&query) || key.to_lowercase().contains(&query)
        })
        .cloned()
        .collect();

    // The current value must never vanish from the select while the user
    // types. A no-item value also keeps the sentinel that stands in for it.
    let mut required: Vec<&str> = vec![value];
    if is_no_item {
        required.push(NO_ITEM_KEY);
    }
    for key in required {
        if !filtered_entries.iter().any(|(k, _)| k == key)
            && let Some(entry) = collapsed.iter().find(|(k, _)| k == key)
        {
            filtered_entries.push(entry.clone());
        }
    }

    // A non-canonical stored key (e.g. `255:255`) still renders as the shared
    // empty-slot option, but re-selecting that option must emit the exact key
    // so `InventoryItem::is_empty()` keeps working.
    let effective_selected_id = if is_no_item { NO_ITEM_KEY } else { value };
    let mut select = lookup_select(&filtered_entries, effective_selected_id);
    if is_no_item && value != NO_ITEM_KEY {
        for id in select.label_to_id.values_mut() {
            if id == NO_ITEM_KEY {
                *id = value.to_string();
            }
        }
    }

    let picker: Element<'static, Message> = {
        let ids = select.label_to_id;
        let fallback_key = value.to_owned();
        pick_list(select.selected, select.options, String::clone)
            .on_select(move |selected_name| {
                let key = ids
                    .get(&selected_name)
                    .cloned()
                    .unwrap_or_else(|| fallback_key.clone());
                on_change(key)
            })
            .width(Length::Fill)
            .into()
    };

    let body: Element<'static, Message> = col![search_input, picker].spacing(4).into();
    with_optional_label(label, body, show_label)
}

#[cfg(all(test, feature = "iced_test"))]
mod tests {
    use super::*;
    use crate::editors::weapon::WeaponEditorMessage;
    use crate::message::{EditorMessage, FileTreeMessage};
    use dispel_core::ItemTypeId;
    use iced_test::selector::{Candidate, Target};
    use iced_test::simulator;

    fn picker(
        value: &str,
        entries: Option<&[(String, String)]>,
        filter: &str,
    ) -> Element<'static, Message> {
        composite_item_picker(
            "Invented Item",
            value,
            entries,
            false,
            filter,
            |v| Message::FileTree(FileTreeMessage::Search(v)),
            |v| {
                Message::Editor(EditorMessage::Weapon(WeaponEditorMessage::FieldChanged(
                    0,
                    "item".to_string(),
                    v,
                )))
            },
        )
    }

    fn emitted_keys(messages: Vec<Message>) -> Vec<String> {
        messages
            .into_iter()
            .filter_map(|m| match m {
                Message::Editor(EditorMessage::Weapon(WeaponEditorMessage::FieldChanged(
                    _,
                    _,
                    value,
                ))) => Some(value),
                _ => None,
            })
            .collect()
    }

    /// Type into the search box and feed the emitted query back into a fresh
    /// render, the way the real caller stores it in state.
    fn typed_filter(element: Element<'static, Message>, query: &str) -> String {
        let mut ui = simulator(element);
        let mut search = |candidate: Candidate<'_>| match candidate {
            Candidate::TextInput { state, .. }
                if state.text().into_owned() == "Search items..." =>
            {
                Some(Target::from(candidate))
            }
            _ => None,
        };
        ui.click(&mut search)
            .expect("search input must be rendered");
        ui.typewrite(query);
        ui.into_messages()
            .filter_map(|m| match m {
                Message::FileTree(FileTreeMessage::Search(v)) => Some(v),
                _ => None,
            })
            .last()
            .expect("search input must emit its query")
    }

    fn select_option(
        value: &str,
        entries: &[(String, String)],
        filter: &str,
        offset: isize,
    ) -> Vec<String> {
        emitted_keys(crate::tests::common::activate_select_option(
            picker(value, Some(entries), filter),
            0,
            offset,
        ))
    }

    fn label_rendered(entries: Option<&[(String, String)]>, show_label: bool) -> bool {
        let element: Element<'_, Message> = composite_item_picker(
            "Invented Item",
            "1:0",
            entries,
            show_label,
            "",
            |_| Message::FileTree(FileTreeMessage::Search(String::new())),
            |_| Message::FileTree(FileTreeMessage::Search(String::new())),
        );
        let mut ui = simulator(element);
        ui.find("Invented Item").is_ok()
    }

    #[test]
    fn test_unmapped_composite_keys_render_a_label_and_keep_value() {
        let entries = [
            ("1:0".to_string(), "[Weapon] Invented Sword".to_string()),
            ("255:15".to_string(), "[-]".to_string()),
        ];

        for value in ["255:0", "255:255"] {
            let values = crate::tests::common::select_values(picker(value, Some(&entries), ""));
            assert!(
                values.iter().any(|v| v == "[-]"),
                "{value} must resolve to the no-item sentinel label, got {values:?}"
            );
        }

        let values = crate::tests::common::select_values(picker("1:999", Some(&entries), ""));
        assert!(
            values.iter().any(|v| v == "Unknown (1:999)"),
            "an unmapped composite key must render a visible fallback label, got {values:?}"
        );

        let keys = select_option("255:0", &entries, "", 0);
        assert_eq!(
            keys,
            vec!["255:0"],
            "choosing the sentinel must keep the exact stored key"
        );
    }

    #[test]
    fn test_one_select_offers_items_from_every_type() {
        let entries = [
            ("1:0".to_string(), "[Weapon] Invented Sword".to_string()),
            ("2:7".to_string(), "[Healing] Invented Herb".to_string()),
            ("3:2".to_string(), "[Edit] Invented Charm".to_string()),
            ("255:15".to_string(), "[-]".to_string()),
        ];
        let values = crate::tests::common::select_values(picker("1:0", Some(&entries), ""));
        assert_eq!(
            values.len(),
            1,
            "there must be exactly one select, got {values:?}"
        );

        assert_eq!(select_option("1:0", &entries, "", 1), vec!["2:7"]);
        assert_eq!(select_option("1:0", &entries, "", 2), vec!["3:2"]);
        assert_eq!(select_option("1:0", &entries, "", 3), vec!["255:15"]);
    }

    #[test]
    fn test_search_narrows_options_and_keeps_selection_visible() {
        let entries = [
            ("1:0".to_string(), "[Weapon] Invented Sword".to_string()),
            ("2:7".to_string(), "[Healing] Invented Herb".to_string()),
            ("3:2".to_string(), "[Edit] Invented Charm".to_string()),
            ("255:15".to_string(), "[-]".to_string()),
        ];
        let query = typed_filter(picker("1:0", Some(&entries), ""), "herb");
        assert_eq!(query, "herb");

        let values = crate::tests::common::select_values(picker("1:0", Some(&entries), &query));
        assert!(
            values.iter().any(|v| v == "[Weapon] Invented Sword"),
            "the current item must stay selected while filtering, got {values:?}"
        );

        assert_eq!(
            select_option("1:0", &entries, &query, -1),
            vec!["2:7"],
            "the only other option must be the matching item"
        );
        let reachable: std::collections::HashSet<String> = (-3..=3)
            .flat_map(|offset| select_option("1:0", &entries, "charm", offset))
            .collect();
        assert_eq!(
            reachable,
            std::collections::HashSet::from(["1:0".to_string(), "3:2".to_string()]),
            "non-matching items must be absent from the narrowed list"
        );

        // Filtering by the composite key works just like filtering by label.
        let query = typed_filter(picker("1:0", Some(&entries), ""), "3:2");
        assert_eq!(select_option("1:0", &entries, &query, -1), vec!["3:2"]);
    }

    #[test]
    fn test_selected_label_emits_its_exact_composite_key() {
        let entries = [
            ("1:0".to_string(), "[Weapon] Invented Sword".to_string()),
            ("2:7".to_string(), "[Healing] Invented Herb".to_string()),
            ("255:15".to_string(), "[-]".to_string()),
        ];
        assert_eq!(select_option("2:7", &entries, "", 0), vec!["2:7"]);
        assert_eq!(select_option("2:7", &entries, "", -1), vec!["1:0"]);
    }

    #[test]
    fn test_no_item_selection_preserves_exact_key() {
        use dispel_core::InventoryItem;

        let entries = [
            ("1:0".to_string(), "[Weapon] Invented Sword".to_string()),
            ("255:15".to_string(), "[-]".to_string()),
        ];

        for value in ["255:255", "255:5"] {
            assert_eq!(select_option(value, &entries, "", 0), vec![value]);
        }
        assert_eq!(select_option("255:15", &entries, "", 0), vec!["255:15"]);

        // Filtering out "[-]" must not hide the empty slot either.
        assert_eq!(
            select_option("255:255", &entries, "sword", 0),
            vec!["255:255"]
        );

        let item = InventoryItem::new(ItemTypeId::NoItem, 255);
        assert!(
            item.is_empty(),
            "255:255 must decode to an empty inventory item"
        );
    }

    #[test]
    fn test_duplicate_display_names_stay_distinguishable() {
        let entries = [
            ("1:4".to_string(), "Invented Blade".to_string()),
            ("2:4".to_string(), "Invented Blade".to_string()),
            ("255:15".to_string(), "[-]".to_string()),
        ];
        let values = crate::tests::common::select_values(picker("2:4", Some(&entries), ""));
        assert!(
            values.iter().any(|v| v == "Invented Blade (2:4)"),
            "duplicate labels must be qualified with their composite key, got {values:?}"
        );
        assert_eq!(select_option("2:4", &entries, "", -1), vec!["1:4"]);
        assert_eq!(select_option("2:4", &entries, "", 0), vec!["2:4"]);
    }

    #[test]
    fn test_composite_picker_label_follows_show_label() {
        let entries = [("1:0".to_string(), "[Weapon] Invented Sword".to_string())];
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
