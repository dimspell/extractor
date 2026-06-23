use std::collections::{HashMap, HashSet};

use crate::components::composite_item::composite_item_picker;
use crate::components::editable::{EditableRecord, FieldKind};
use crate::message::{Message, MessageExt};
use crate::style;
use iced::widget::{column, pick_list, row, scrollable, text, text_input};
use iced::{Element, Fill};

use super::super::message::{MapEditorMessage, SelectedEntity};

/// Iterate all `FieldDescriptor`s for `R` and build a scrollable column of editor rows.
///
/// `text_input` copies its value string internally, so the `String` returned by
/// `get_field` can be a temporary — the resulting `Element` has no lifetime tie to it.
pub fn build_record_fields<'a, R: EditableRecord>(
    record: &R,
    tab_id: usize,
    sel: SelectedEntity,
    lookups: &'a HashMap<String, Vec<(String, String)>>,
) -> Element<'a, Message> {
    let mut col = column![].spacing(5);
    let composite_id_fields: HashSet<&'static str> = R::field_descriptors()
        .iter()
        .filter_map(|d| match &d.kind {
            FieldKind::CompositeItem { id_field, .. } => Some(*id_field),
            _ => None,
        })
        .collect();

    for desc in R::field_descriptors() {
        if composite_id_fields.contains(&desc.name) {
            continue;
        }
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
/// short-lived borrow of a locally-computed `String` — safe because `text_input`
/// and `pick_list` copy their value arguments before returning the widget.
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
            text_input("", value)
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
                text_input("", value)
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

        FieldKind::CompositeItem {
            lookup_key,
            id_field,
        } => {
            let entries = lookups.get(*lookup_key).map(|v| v.as_slice());
            composite_item_picker(label, value, id_field, entries, move |v| {
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
