use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::Hasher;

use iced::widget::combo_box::{self, ComboBox};
use iced::widget::{row, text, text_input};
use iced::{Element, Length};

use crate::message::Message;
use crate::style;

const LABEL_W: f32 = 140.0;

/// The empty-slot option shared with the item catalog.
const NO_ITEM_KEY: &str = "255:15";
const NO_ITEM_LABEL: &str = "[-]";

/// One item in the shared composite item catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemOption {
    pub key: String,
    pub label: String,
}

impl fmt::Display for ItemOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

/// A single app-level option list shared by every composite item field.
#[derive(Default)]
pub struct CompositeItemOptions {
    pub state: combo_box::State<ItemOption>,
    pub fingerprint: u64,
}

fn fingerprint_options(entries: Option<&[(String, String)]>) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write_u8(u8::from(entries.is_some()));
    if let Some(entries) = entries {
        for (key, label) in entries {
            hasher.write(key.as_bytes());
            hasher.write_u8(0);
            hasher.write(label.as_bytes());
            hasher.write_u8(1);
        }
    }
    hasher.finish()
}

/// Rebuild the shared option list whenever the `"items"` lookup changes.
pub fn refresh_composite_item_options(state: &mut crate::state::AppState) {
    let entries = state.lookups.get("items").map(Vec::as_slice);
    let fingerprint = fingerprint_options(entries);
    if fingerprint == state.composite_item_options.fingerprint {
        return;
    }

    let options = match entries {
        Some(entries) => {
            let mut options: Vec<ItemOption> = entries
                .iter()
                .map(|(key, label)| ItemOption {
                    key: key.clone(),
                    label: label.clone(),
                })
                .collect();
            if !options.iter().any(|option| option.key == NO_ITEM_KEY) {
                options.push(ItemOption {
                    key: NO_ITEM_KEY.to_string(),
                    label: NO_ITEM_LABEL.to_string(),
                });
            }
            options
        }
        None => Vec::new(),
    };

    state.composite_item_options.state = combo_box::State::new(options);
    state.composite_item_options.fingerprint = fingerprint;
}

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
/// Every item file shares the one app-level option list, so all fields get the
/// same built-in fuzzy search. Falls back to a plain `text_input` before the
/// item catalog has been loaded. `show_label` controls whether the label
/// column is drawn by this component or the caller.
pub fn composite_item_picker<'a>(
    label: &'static str,
    value: &str,
    combo_state: &'a combo_box::State<ItemOption>,
    show_label: bool,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    // Without lookups there is nothing to search: keep the raw key editable.
    if combo_state.options().is_empty() {
        let input: Element<'a, Message> = text_input("", value.to_owned())
            .on_input(on_change)
            .padding(4)
            .size(11)
            .width(Length::Fill)
            .into();
        return with_optional_label(label, input, show_label);
    }

    // ComboBox copies the selection into a String, so a locally-built option
    // is enough. Only the label must match; the closure owns the exact key.
    let selected = combo_state
        .options()
        .iter()
        .find(|option| option.key == value)
        .cloned()
        .unwrap_or_else(|| ItemOption {
            key: value.to_owned(),
            label: if value.starts_with("255:") {
                NO_ITEM_LABEL.to_string()
            } else {
                format!("Unknown ({value})")
            },
        });

    // A non-canonical stored key (e.g. `255:255`) renders as the shared empty
    // slot. Re-selecting that option must still emit the exact stored key so
    // `InventoryItem::is_empty()` keeps working.
    let stored_key = value.to_owned();
    let picker: Element<'a, Message> = ComboBox::new(
        combo_state,
        "Search items...",
        Some(&selected),
        move |option: ItemOption| {
            let key = if option.key == NO_ITEM_KEY && stored_key.starts_with("255:") {
                stored_key.clone()
            } else {
                option.key
            };
            on_change(key)
        },
    )
    .padding(4)
    .size(11)
    .width(Length::Fill)
    .menu_height(240.0)
    .into();

    with_optional_label(label, picker, show_label)
}

#[cfg(all(test, feature = "iced_test"))]
mod tests {
    use super::*;
    use crate::editors::weapon::WeaponEditorMessage;
    use crate::message::EditorMessage;
    use iced_test::core::accessibility::accesskit;
    use iced_test::instruction::Key;
    use iced_test::selector::{Candidate, Target};
    use iced_test::simulator;

    fn options(entries: &[(&str, &str)]) -> combo_box::State<ItemOption> {
        combo_box::State::new(
            entries
                .iter()
                .map(|(key, label)| ItemOption {
                    key: (*key).to_string(),
                    label: (*label).to_string(),
                })
                .collect(),
        )
    }

    fn picker<'a>(value: &str, state: &'a combo_box::State<ItemOption>) -> Element<'a, Message> {
        composite_item_picker("Invented Item", value, state, false, |v| {
            Message::Editor(EditorMessage::Weapon(WeaponEditorMessage::FieldChanged(
                0,
                "item".to_string(),
                v,
            )))
        })
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

    fn select_option(
        value: &str,
        state: &combo_box::State<ItemOption>,
        offset: isize,
    ) -> Vec<String> {
        emitted_keys(crate::tests::common::activate_select_option(
            picker(value, state),
            0,
            offset,
        ))
    }

    /// Labels of the options shown by the open combo box. The current
    /// selection is erased first so the widget filters by an empty query.
    fn visible_options(state: &combo_box::State<ItemOption>) -> Vec<String> {
        let mut ui = simulator(picker("1:0", state));
        let mut seen = 0;
        let mut text_len = 0;
        let mut select = |candidate: Candidate<'_>| match candidate {
            Candidate::TextInput { state, .. } if seen == 0 => {
                seen += 1;
                text_len = state.text().chars().count();
                Some(Target::from(candidate))
            }
            Candidate::TextInput { .. } => {
                seen += 1;
                None
            }
            _ => None,
        };
        ui.click(&mut select).expect("combo box must be clickable");
        for _ in 0..text_len {
            ui.tap_key(Key::Backspace);
        }
        ui.accessibility_tree()
            .nodes
            .into_iter()
            .filter(|(_, node)| node.role() == accesskit::Role::MenuListOption)
            .filter_map(|(_, node)| node.label().map(str::to_owned))
            .collect()
    }

    fn label_rendered(state: Option<&combo_box::State<ItemOption>>, show_label: bool) -> bool {
        let empty = options(&[]);
        let state = state.unwrap_or(&empty);
        let mut ui = simulator(composite_item_picker(
            "Invented Item",
            "1:0",
            state,
            show_label,
            |_| {
                Message::Editor(EditorMessage::Weapon(WeaponEditorMessage::FieldChanged(
                    0,
                    "item".to_string(),
                    String::new(),
                )))
            },
        ));
        ui.find("Invented Item").is_ok()
    }

    #[test]
    fn test_one_combo_box_lists_labels_from_every_item_type() {
        let state = options(&[
            ("1:0", "[Weapon] Invented Sword"),
            ("2:7", "[Healing] Invented Herb"),
            ("3:2", "[Edit] Invented Charm"),
            ("4:1", "[Event] Invented Scroll"),
            ("5:3", "[Misc] Invented Stone"),
            ("255:15", NO_ITEM_LABEL),
        ]);

        assert_eq!(
            crate::tests::common::select_values(picker("1:0", &state)),
            vec!["[Weapon] Invented Sword"],
            "one item field must render exactly one combo box"
        );
        let labels = visible_options(&state);
        for label in [
            "[Weapon] Invented Sword",
            "[Healing] Invented Herb",
            "[Edit] Invented Charm",
            "[Event] Invented Scroll",
            "[Misc] Invented Stone",
            NO_ITEM_LABEL,
        ] {
            assert!(
                labels.iter().any(|shown| shown == label),
                "the shared option list must offer {label:?}, got {labels:?}"
            );
        }
        assert_eq!(select_option("1:0", &state, 3), vec!["4:1"]);
        assert_eq!(select_option("1:0", &state, 5), vec!["255:15"]);
    }

    #[test]
    fn test_selected_option_emits_its_exact_composite_key() {
        let state = options(&[
            ("1:0", "[Weapon] Invented Sword"),
            ("2:7", "[Healing] Invented Herb"),
            ("255:15", NO_ITEM_LABEL),
        ]);
        assert_eq!(select_option("1:0", &state, 1), vec!["2:7"]);
        assert_eq!(select_option("1:0", &state, 2), vec!["255:15"]);
        assert_eq!(select_option("2:7", &state, 0), vec!["1:0"]);
    }

    #[test]
    fn test_non_canonical_no_item_key_is_preserved() {
        let state = options(&[
            ("1:0", "[Weapon] Invented Sword"),
            ("255:15", NO_ITEM_LABEL),
        ]);
        for value in ["255:255", "255:5"] {
            assert_eq!(
                crate::tests::common::select_values(picker(value, &state)),
                vec![NO_ITEM_LABEL],
                "{value} must display the shared empty-slot label"
            );
            assert_eq!(
                select_option(value, &state, -1),
                vec![value],
                "reselecting the empty slot must keep the exact stored key"
            );
        }
        assert_eq!(select_option("255:15", &state, -1), vec!["255:15"]);
    }

    #[test]
    fn test_duplicate_display_names_remain_distinct() {
        let state = options(&[
            ("1:4", "Invented Blade"),
            ("2:4", "Invented Blade"),
            ("255:15", NO_ITEM_LABEL),
        ]);
        let labels = visible_options(&state);
        assert_eq!(
            labels,
            vec!["Invented Blade", "Invented Blade", NO_ITEM_LABEL],
            "both duplicate rows must stay in the one option list"
        );
        assert_eq!(select_option("2:4", &state, 0), vec!["1:4"]);
        assert_eq!(select_option("2:4", &state, 1), vec!["2:4"]);
    }

    #[test]
    fn test_unmapped_key_shows_visible_fallback() {
        let state = options(&[
            ("1:0", "[Weapon] Invented Sword"),
            ("255:15", NO_ITEM_LABEL),
        ]);
        assert_eq!(
            crate::tests::common::select_values(picker("1:999", &state)),
            vec!["Unknown (1:999)"]
        );
        assert_eq!(
            crate::tests::common::select_values(picker("255:0", &state)),
            vec![NO_ITEM_LABEL]
        );
    }

    #[test]
    fn test_fallback_text_input_edits_the_raw_value() {
        let state = options(&[]);
        let no_combo = simulator(picker("255:255", &state))
            .accessibility_tree()
            .nodes
            .into_iter()
            .any(|(_, node)| node.role() == accesskit::Role::ComboBox);
        assert!(
            !no_combo,
            "without a loaded catalog there must be no combo box"
        );

        let mut ui = simulator(picker("255:255", &state));
        let mut input = |candidate: Candidate<'_>| match candidate {
            Candidate::TextInput { state, .. } if state.text().into_owned() == "255:255" => {
                Some(Target::from(candidate))
            }
            _ => None,
        };
        ui.click(&mut input)
            .expect("raw value input must be rendered");
        ui.typewrite("5");
        assert_eq!(emitted_keys(ui.into_messages().collect()), vec!["255:2555"]);
    }

    #[test]
    fn test_composite_picker_label_follows_show_label() {
        let state = options(&[("1:0", "[Weapon] Invented Sword")]);
        for show_label in [true, false] {
            assert_eq!(
                label_rendered(None, show_label),
                show_label,
                "fallback picker label must follow show_label"
            );
            assert_eq!(
                label_rendered(Some(&state), show_label),
                show_label,
                "combo box label must follow show_label"
            );
        }
    }
}

#[cfg(test)]
mod refresh_tests {
    use super::*;
    use crate::state::AppState;

    #[test]
    fn test_refresh_builds_one_option_list_and_adds_the_sentinel() {
        let mut app = AppState::default();
        app.lookups.insert(
            "items".to_string(),
            vec![
                ("1:5".to_string(), "[Weapon] Invented Sword".to_string()),
                ("2:7".to_string(), "[Healing] Invented Herb".to_string()),
            ],
        );

        refresh_composite_item_options(&mut app);
        let options = app.composite_item_options.state.options();
        assert_eq!(
            options
                .iter()
                .map(|option| option.key.as_str())
                .collect::<Vec<_>>(),
            vec!["1:5", "2:7", NO_ITEM_KEY]
        );
        assert_eq!(options.last().unwrap().label, NO_ITEM_LABEL);
        let fingerprint = app.composite_item_options.fingerprint;
        let pointer = options.as_ptr();

        refresh_composite_item_options(&mut app);
        assert_eq!(
            app.composite_item_options.fingerprint, fingerprint,
            "an unchanged lookup must not rebuild the options"
        );
        assert_eq!(
            app.composite_item_options.state.options().as_ptr(),
            pointer,
            "the same option list must be reused"
        );

        app.lookups
            .get_mut("items")
            .unwrap()
            .push(("3:2".to_string(), "[Edit] Invented Charm".to_string()));
        refresh_composite_item_options(&mut app);
        assert_ne!(app.composite_item_options.fingerprint, fingerprint);
        assert_eq!(app.composite_item_options.state.options().len(), 4);
    }

    #[test]
    fn test_refresh_without_lookups_keeps_the_fallback_empty() {
        let mut app = AppState::default();
        refresh_composite_item_options(&mut app);
        assert!(app.composite_item_options.state.options().is_empty());
    }
}
