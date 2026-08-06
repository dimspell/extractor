use std::collections::HashMap;

use crate::components::editable::EditableRecord;
use crate::components::utils::{horizontal_rule, horizontal_space};
use crate::message::{Message, MessageExt};
use crate::style;
use dispel_core::{DrawItem, ExtraRef, MonsterRef, NPC};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Fill, Font};

use super::super::message::{MapEditorMessage, SelectedEntity};
use super::super::state::MapEditorState;
use super::fields;

/// Build the inspector sidebar for a selected entity.
pub fn build_inspector<'a>(
    state: &'a MapEditorState,
    tab_id: usize,
    sel: SelectedEntity,
    lookups: &'a HashMap<String, Vec<(String, String)>>,
) -> Element<'a, Message> {
    let close_msg = Message::map_editor(MapEditorMessage::Deselect(tab_id));

    let (title, width, body): (&'static str, f32, Element<'a, Message>) = match sel {
        SelectedEntity::Monster(i) => {
            let body = if let Some(record) = state.data.monsters.get(i) {
                fields::build_record_fields::<MonsterRef>(record, tab_id, sel, lookups)
            } else {
                text("Monster not found").size(12).into()
            };
            (MonsterRef::detail_title(), MonsterRef::detail_width(), body)
        }
        SelectedEntity::Npc(i) => {
            let body: Element<'_, Message> = if let Some(record) = state.data.npcs.get(i) {
                // Collect field elements + button into a single column
                // (avoids iced's lifetime-invariance issues with push).
                let mut els: Vec<Element<'_, Message>> = Vec::new();
                for desc in NPC::field_descriptors() {
                    let value = record.get_field(desc.name);
                    els.push(fields::inspector_field_row(
                        desc.label, desc.name, &desc.kind, &value, tab_id, sel, lookups,
                    ));
                }
                els.push(horizontal_rule(1).into());
                let preview_btn = button(text("Preview Dialog").size(11))
                    .on_press(Message::map_editor(MapEditorMessage::ShowDialogPreview(
                        tab_id, i,
                    )))
                    .padding([4, 10])
                    .style(style::browse_button)
                    .into();
                els.push(preview_btn);
                column(els).spacing(6).into()
            } else {
                text("NPC not found").size(12).into()
            };
            (NPC::detail_title(), NPC::detail_width(), body)
        }
        SelectedEntity::Extra(i) => {
            let body = if let Some(record) = state.data.extra_refs.get(i) {
                fields::build_record_fields::<ExtraRef>(record, tab_id, sel, lookups)
            } else {
                text("Object not found").size(12).into()
            };
            (ExtraRef::detail_title(), ExtraRef::detail_width(), body)
        }
        SelectedEntity::DrawItem(i) => {
            let body = if let Some(record) = state.data.draw_items.get(i) {
                fields::build_record_fields::<DrawItem>(record, tab_id, sel, lookups)
            } else {
                text("Draw item not found").size(12).into()
            };
            (DrawItem::detail_title(), DrawItem::detail_width(), body)
        }
        SelectedEntity::CollisionTile(tx, ty) => {
            let body = column![
                text(format!("Collision at ({}, {})", tx, ty)).size(12),
                horizontal_rule(1),
                row![
                    text("Click the tile again to toggle collision.")
                        .size(11)
                        .style(style::subtle_text),
                ]
                .padding(4),
            ]
            .spacing(8)
            .into();
            ("Collision Tile", 220.0, body)
        }
        SelectedEntity::EventTile(tx, ty) => {
            let body = if let Some(map_handle) = state.map_data() {
                let map_data = &map_handle.0;
                if let Some(event) = map_data.events.get(&(tx, ty)) {
                    let x_str = event.x.to_string();
                    let y_str = event.y.to_string();
                    let event_id_val = event.event_id.to_string();

                    container(
                        column![
                            text(format!("Event at ({}, {})", tx, ty)).size(12),
                            horizontal_rule(1),
                            // event_id (editable)
                            row![
                                text("event_id")
                                    .size(11)
                                    .width(140.0)
                                    .style(style::subtle_text),
                                text_input("0", &event_id_val)
                                    .on_input(move |v| {
                                        Message::map_editor(MapEditorMessage::EntityFieldChanged(
                                            tab_id,
                                            SelectedEntity::EventTile(tx, ty),
                                            "event_id".to_string(),
                                            v,
                                        ))
                                    })
                                    .padding(4)
                                    .size(11),
                            ]
                            .spacing(6)
                            .align_y(iced::Alignment::Center),
                            // x (read-only)
                            row![
                                text("x").size(11).width(140.0).style(style::subtle_text),
                                text(x_str).size(11),
                            ]
                            .spacing(6)
                            .align_y(iced::Alignment::Center),
                            // y (read-only)
                            row![
                                text("y").size(11).width(140.0).style(style::subtle_text),
                                text(y_str).size(11),
                            ]
                            .spacing(6)
                            .align_y(iced::Alignment::Center),
                            // Remove event button
                            button(text("Remove Event").size(11))
                                .on_press(Message::map_editor(
                                    MapEditorMessage::EntityFieldChanged(
                                        tab_id,
                                        SelectedEntity::EventTile(tx, ty),
                                        "event_id".to_string(),
                                        "0".to_string(),
                                    )
                                ))
                                .padding([4, 10])
                                .style(style::browse_button),
                        ]
                        .spacing(8),
                    )
                    .into()
                } else {
                    // Tile has no event — offer to create one
                    container(
                        column![
                            text("No event on this tile")
                                .size(11)
                                .style(style::subtle_text),
                            button(text("Create Event").size(11))
                                .on_press(Message::map_editor(
                                    MapEditorMessage::EntityFieldChanged(
                                        tab_id,
                                        SelectedEntity::EventTile(tx, ty),
                                        "event_id".to_string(),
                                        "0".to_string(),
                                    )
                                ))
                                .padding([4, 10]),
                        ]
                        .spacing(8),
                    )
                    .into()
                }
            } else {
                text("Map data not loaded").size(11).into()
            };
            ("Event Inspector", 240.0, body)
        }
    };

    let header = row![
        text(title)
            .size(12)
            .font(Font::MONOSPACE)
            .style(style::subtle_text),
        horizontal_space(),
        button(text("×").size(14))
            .on_press(close_msg)
            .padding([3, 8])
            .style(style::browse_button),
    ]
    .align_y(iced::Alignment::Center)
    .padding([0, 4]);

    scrollable(
        container(
            column![header, horizontal_rule(1), body]
                .spacing(6)
                .padding(10),
        )
        .width(width)
        .height(Fill)
        .style(style::inspector_container),
    )
    .height(Fill)
    .into()
}
