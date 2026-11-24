use std::collections::HashMap;

use crate::components::composite_item::composite_item_picker;
use crate::components::editable::{EditableRecord, FieldKind};
use crate::message::{Message, MessageExt};
use crate::style;
use iced::widget::{column, pick_list, row, scrollable, text, text_input};
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
) -> Element<'a, Message> {
    let mut col = column![].spacing(5);

    for desc in R::field_descriptors() {
        let value = record.get_field(desc.name);
        col = col.push(inspector_field_row(
            desc.label, desc.name, &desc.kind, &value, tab_id, sel, lookups,
        ));
    }
    scrollable(col).spacing(6).into()
}

/// Render a single labeled field row for the map editor inspector.
///
/// `label` and `name` are `&'static str` (from `FieldDescriptor`); `value` is a
/// short-lived borrow of a locally-computed `String`; text inputs clone it before
/// constructing their returned widgets.
pub fn inspector_field_row<'a>(
    label: &'static str,
    name: &'static str,
    kind: &FieldKind,
    value: &str,
    tab_id: usize,
    sel: SelectedEntity,
    lookups: &'a HashMap<String, Vec<(String, String)>>,
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
            let options: Vec<String> = entries.iter().map(|(_, d)| d.clone()).collect();
            let selected = entries
                .iter()
                .find(|(id, _)| id == value)
                .map(|(_, display)| display.clone());

            let field_widget: Element<'a, Message> = if !options.is_empty() {
                pick_list(selected, options, String::clone)
                    .on_select(move |v: String| {
                        let id = entries
                            .iter()
                            .find(|(_, d)| d == &v)
                            .map(|(id, _)| id.clone())
                            .unwrap_or_default();
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
            let options: Vec<&'static str> = variants.to_vec();
            let selected = options
                .iter()
                .find(|&&opt| opt == value)
                .copied()
                .or_else(|| options.first().copied());
            row![
                text(label)
                    .size(11)
                    .width(LABEL_W)
                    .style(style::subtle_text),
                pick_list(selected, options, |v| v.to_string())
                    .on_select(move |v: &'static str| {
                        Message::map_editor(MapEditorMessage::EntityFieldChanged(
                            tab_id,
                            sel,
                            name.to_string(),
                            v.to_string(),
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

        FieldKind::CompositeItem { lookup_key } => {
            let entries = lookups.get(*lookup_key).map(|v| v.as_slice());
            composite_item_picker(label, value, entries, true, move |v| {
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
        );
        assert_eq!(
            count_label_text(element, "Invented Item:"),
            1,
            "map editor has no outer label, so the picker must render exactly one"
        );
    }
}
