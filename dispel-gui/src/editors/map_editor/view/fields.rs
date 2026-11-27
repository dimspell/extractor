use std::collections::HashMap;

use crate::components::composite_item::{ItemOption, composite_item_picker};
use crate::components::editable::{EditableRecord, FieldKind};
use crate::message::{Message, MessageExt};
use crate::style;
use iced::widget::{column, combo_box, pick_list, row, scrollable, text, text_input};
use iced::{Element, Fill};

use super::super::message::{MapEditorMessage, SelectedEntity};

/// Iterate all `FieldDescriptor`s for `R` and build a scrollable column of editor rows.
///
/// Text input values are cloned so the resulting widgets do not borrow temporary
/// field strings.
pub fn build_record_fields<'a, R: EditableRecord>(
    record: &R,
    tab_id: usize,
    sel: SelectedEntity,
    lookups: &'a HashMap<String, Vec<(String, String)>>,
    combo_state: &'a combo_box::State<ItemOption>,
) -> Element<'a, Message> {
    let mut col = column![].spacing(5);

    for desc in R::field_descriptors() {
        let value = record.get_field(desc.name);
        col = col.push(inspector_field_row(
            desc.label,
            desc.name,
            &desc.kind,
            &value,
            tab_id,
            sel,
            lookups,
            combo_state,
        ));
    }
    scrollable(col).spacing(6).into()
}

/// Render a single labeled field row for the map editor inspector.
///
/// `label` and `name` are `&'static str` (from `FieldDescriptor`); `value` is a
/// short-lived borrow of a locally-computed `String`; text inputs clone it before
/// constructing their returned widgets.
#[allow(clippy::too_many_arguments)]
pub fn inspector_field_row<'a>(
    label: &'static str,
    name: &'static str,
    kind: &FieldKind,
    value: &str,
    tab_id: usize,
    sel: SelectedEntity,
    lookups: &'a HashMap<String, Vec<(String, String)>>,
    combo_state: &'a combo_box::State<ItemOption>,
) -> Element<'a, Message> {
    const LABEL_W: f32 = 140.0;
    match kind {
        FieldKind::String | FieldKind::TextArea | FieldKind::Integer | FieldKind::Boolean => row![
            text(label)
                .size(11)
                .width(LABEL_W)
                .style(style::subtle_text),
            text_input("", value.to_owned())
                .on_input(move |v| {
                    Message::map_editor(MapEditorMessage::EntityFieldChanged(
                        tab_id,
                        sel,
                        name.to_string(),
                        v,
                    ))
                })
                .padding(4)
                .size(11),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .into(),

        FieldKind::Lookup(lookup_key) => {
            // Clone entries so the move closures own their data.
            let entries: Vec<(String, String)> =
                lookups.get(*lookup_key).cloned().unwrap_or_default();

            let field_widget: Element<'a, Message> = if !entries.is_empty() {
                let select = crate::components::select::lookup_select(&entries, value);
                let label_to_id = select.label_to_id;
                pick_list(select.selected, select.options, String::clone)
                    .on_select(move |v: String| {
                        let id = label_to_id.get(&v).cloned().unwrap_or_default();
                        Message::map_editor(MapEditorMessage::EntityFieldChanged(
                            tab_id,
                            sel,
                            name.to_string(),
                            id,
                        ))
                    })
                    .width(Fill)
                    .padding(4)
                    .text_size(11)
                    .into()
            } else {
                text_input("", value.to_owned())
                    .on_input(move |v| {
                        Message::map_editor(MapEditorMessage::EntityFieldChanged(
                            tab_id,
                            sel,
                            name.to_string(),
                            v,
                        ))
                    })
                    .padding(4)
                    .size(11)
                    .into()
            };

            row![
                text(label)
                    .size(11)
                    .width(LABEL_W)
                    .style(style::subtle_text),
                field_widget,
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center)
            .into()
        }

        FieldKind::Enum { variants } => {
            let mut options: Vec<String> = variants.iter().map(|v| (*v).to_string()).collect();
            let selected = if options.iter().any(|opt| opt == value) {
                Some(value.to_string())
            } else if !value.is_empty() {
                options.push(value.to_string());
                Some(value.to_string())
            } else {
                None
            };
            row![
                text(label)
                    .size(11)
                    .width(LABEL_W)
                    .style(style::subtle_text),
                pick_list(selected, options, String::clone)
                    .on_select(move |v: String| {
                        Message::map_editor(MapEditorMessage::EntityFieldChanged(
                            tab_id,
                            sel,
                            name.to_string(),
                            v,
                        ))
                    })
                    .width(Fill)
                    .padding(4)
                    .text_size(11),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center)
            .into()
        }

        FieldKind::CompositeItem { lookup_key: _ } => {
            composite_item_picker(label, value, combo_state, true, move |v| {
                Message::map_editor(MapEditorMessage::EntityFieldChanged(
                    tab_id,
                    sel,
                    name.to_string(),
                    v,
                ))
            })
        }
    }
}

#[cfg(all(test, feature = "iced_test"))]
mod tests {
    use super::*;
    use iced_test::selector::{Candidate, Target};
    use iced_test::simulator;

    static EMPTY_COMBO: std::sync::LazyLock<combo_box::State<ItemOption>> =
        std::sync::LazyLock::new(|| combo_box::State::new(Vec::new()));

    fn count_label_text(element: Element<'_, Message>, label: &str) -> usize {
        let mut ui = simulator(element);
        let mut count = 0usize;
        let mut selector = |candidate: Candidate<'_>| -> Option<Target> {
            if let Candidate::Text { content, .. } = candidate
                && content == label
            {
                count += 1;
            }
            None
        };
        let _ = ui.find(&mut selector);
        count
    }

    #[test]
    fn test_map_editor_composite_combo_emits_exact_key() {
        let kind = FieldKind::CompositeItem {
            lookup_key: "invented_items",
        };
        let lookups = HashMap::from([(
            "invented_items".to_string(),
            vec![
                ("1:0".to_string(), "[Weapon] Invented Sword".to_string()),
                ("2:7".to_string(), "[Healing] Invented Herb".to_string()),
            ],
        )]);
        let combo = combo_box::State::new(vec![
            ItemOption {
                key: "1:0".to_string(),
                label: "[Weapon] Invented Sword".to_string(),
            },
            ItemOption {
                key: "2:7".to_string(),
                label: "[Healing] Invented Herb".to_string(),
            },
        ]);

        let element = inspector_field_row(
            "Invented Item:",
            "item_type",
            &kind,
            "1:0",
            0,
            SelectedEntity::DrawItem(0),
            &lookups,
            &combo,
        );
        assert!(
            crate::tests::common::select_shows(element, "[Weapon] Invented Sword"),
            "the map editor combo must show the selected item label"
        );

        let element = inspector_field_row(
            "Invented Item:",
            "item_type",
            &kind,
            "1:0",
            0,
            SelectedEntity::DrawItem(0),
            &lookups,
            &combo,
        );
        let emitted: Vec<String> = crate::tests::common::activate_select_option(element, 0, 1)
            .into_iter()
            .filter_map(|message| match message {
                Message::Editor(crate::message::editor::EditorMessage::MapEditor(
                    MapEditorMessage::EntityFieldChanged(_, _, _, value),
                )) => Some(value),
                _ => None,
            })
            .collect();
        assert_eq!(
            emitted,
            vec!["2:7"],
            "the map editor must emit the selected composite key"
        );
    }

    #[test]
    fn test_npc_facing_directions_offer_left_in_map_editor() {
        use crate::components::editable::EditableRecord;
        use dispel_core::{EventNpcRef, NPC, references::enums::NpcLookingDirection};

        let _ = NpcLookingDirection::Left;

        fn check<R: EditableRecord>(field: &'static str) {
            let mut record = R::default();
            assert!(
                record.set_field(field, "Left".to_string()),
                "{field} must accept Left"
            );
            let value = record.get_field(field);
            assert_eq!(value, "Left", "{field} must display Left");
            let desc = R::field_descriptors()
                .iter()
                .find(|d| d.name == field)
                .expect("descriptor exists");

            let lookups = HashMap::new();
            let element = inspector_field_row(
                desc.label,
                desc.name,
                &desc.kind,
                &value,
                0,
                SelectedEntity::Npc(0),
                &lookups,
                &EMPTY_COMBO,
            );
            assert!(
                crate::tests::common::select_shows(element, "Left"),
                "map editor row for {field} must render the selected Left label"
            );
        }

        for field in [
            "waypoint1_facing_direction",
            "waypoint2_facing_direction",
            "waypoint3_facing_direction",
            "waypoint4_facing_direction",
        ] {
            check::<NPC>(field);
            check::<EventNpcRef>(field);
        }
    }

    #[test]
    fn test_map_editor_enum_shows_unknown_value() {
        let kind = FieldKind::Enum {
            variants: &["Alpha", "Beta"],
        };
        let lookups = HashMap::new();

        let unknown = inspector_field_row(
            "Invented:",
            "invented_field",
            &kind,
            "Bogus",
            0,
            SelectedEntity::DrawItem(0),
            &lookups,
            &EMPTY_COMBO,
        );
        assert!(
            crate::tests::common::select_shows(unknown, "Bogus"),
            "an unexpected enum value must render itself, not the first option"
        );

        let known = inspector_field_row(
            "Invented:",
            "invented_field",
            &kind,
            "Beta",
            0,
            SelectedEntity::DrawItem(0),
            &lookups,
            &EMPTY_COMBO,
        );
        assert!(
            crate::tests::common::select_shows(known, "Beta"),
            "a known enum value must render exactly"
        );
    }

    #[test]
    fn test_map_editor_lookup_shows_missing_and_duplicate_ids() {
        let kind = FieldKind::Lookup("monster_names");
        let lookups = HashMap::from([(
            "monster_names".to_string(),
            vec![
                ("7".to_string(), "Invented Creature".to_string()),
                ("9".to_string(), "Invented Creature".to_string()),
            ],
        )]);

        let missing = inspector_field_row(
            "Monster:",
            "monster_db_id",
            &kind,
            "99",
            0,
            SelectedEntity::Monster(0),
            &lookups,
            &EMPTY_COMBO,
        );
        assert!(
            crate::tests::common::select_shows(missing, "Unknown (99)"),
            "an id missing from the lookup must render a visible label"
        );

        let duplicate = inspector_field_row(
            "Monster:",
            "monster_db_id",
            &kind,
            "9",
            0,
            SelectedEntity::Monster(0),
            &lookups,
            &EMPTY_COMBO,
        );
        assert!(
            crate::tests::common::select_shows(duplicate, "Invented Creature (9)"),
            "duplicate display names must be qualified with their id"
        );

        let duplicate = inspector_field_row(
            "Monster:",
            "monster_db_id",
            &kind,
            "9",
            0,
            SelectedEntity::Monster(0),
            &lookups,
            &EMPTY_COMBO,
        );
        let messages = crate::tests::common::activate_select_option(duplicate, 0, -1);
        let emitted: Vec<String> = messages
            .into_iter()
            .filter_map(|m| match m {
                Message::Editor(crate::message::editor::EditorMessage::MapEditor(
                    MapEditorMessage::EntityFieldChanged(_, _, _, value),
                )) => Some(value),
                _ => None,
            })
            .collect();
        assert!(
            emitted.iter().any(|v| v == "7"),
            "selecting the other duplicate must emit its own id, got {emitted:?}"
        );
    }

    #[test]
    fn test_map_editor_composite_field_keeps_its_label() {
        let kind = FieldKind::CompositeItem {
            lookup_key: "invented_items",
        };
        let lookups = HashMap::new();
        let element = inspector_field_row(
            "Invented Item:",
            "item_type",
            &kind,
            "1:0",
            0,
            SelectedEntity::DrawItem(0),
            &lookups,
            &EMPTY_COMBO,
        );
        assert_eq!(
            count_label_text(element, "Invented Item:"),
            1,
            "map editor has no outer label, so the picker must render exactly one"
        );
    }
}
