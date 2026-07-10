use iced::widget::{button, container, scrollable, text, Column};
use iced::{Element, Fill};
use iced::Padding;

use crate::editors::save_file_viewer::state::SaveFileViewerState;
use crate::editors::save_file_viewer::SaveFileViewerMessage;
use crate::message::Message;
use crate::message::MessageExt;
use gui_widgets::components::paragraph_cache::ParagraphCache;
use gui_widgets::{TableColumn, TableWidget};

/// Maps section: vertical sidebar + entity tables per map.
pub fn view<'a>(state: &'a SaveFileViewerState) -> Element<'a, Message> {
    let sf = match state.save_file.as_ref() {
        Some(sf) => sf,
        None => return container(text("No save file loaded")).into(),
    };

    if sf.maps.is_empty() {
        return container(text("No maps"))
            .width(Fill)
            .height(Fill)
            .padding(16)
            .into();
    }

    // Left sidebar: map ID buttons
    let sidebar: Element<'a, Message> = {
        let mut col = iced::widget::Column::<Message>::new()
            .spacing(2)
            .padding(8)
            .width(120);
        for (i, map) in sf.maps.iter().enumerate() {
            let is_active = state.selected_map == Some(i);
            let mut btn = button(text(format!("Map {}", map.map_id)).size(12));
            if is_active {
                btn = btn.style(iced::widget::button::primary);
            }
            col = col.push(
                btn.on_press(Message::save_file_viewer(
                    SaveFileViewerMessage::SelectMap(i),
                ))
                .padding([4, 8])
                .width(Fill),
            );
        }
        iced::widget::Scrollable::<Message>::new(col)
            .height(Fill)
            .into()
    };

    // Right panel: entity tables for selected map
    let main: Element<'a, Message> = if let Some(idx) = state.selected_map {
        let caches = state.maps_display_caches.get(idx);
        let map = &sf.maps[idx];

        let mut content = Column::<Message>::new().spacing(12).padding(16);

        // Monsters table
        content = content.push(section_header(&format!("Monsters ({})", map.monsters.len())));
        if let Some(c) = caches {
            if !c.monsters.is_empty() {
                content = content.push(entity_table(
                    &c.monsters,
                    &c.monsters_indices,
                    vec![
                        TableColumn { width_px: 130.0, label: "Name".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 80.0, label: "HP".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 80.0, label: "MP".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 42.0, label: "Atk".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 42.0, label: "Def".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 48.0, label: "Dodge".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 42.0, label: "Hit".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 42.0, label: "XP".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 42.0, label: "Gold".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 42.0, label: "Sight".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 42.0, label: "Range".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 65.0, label: "AI".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 42.0, label: "X".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 42.0, label: "Y".into(), sort: None, has_filter: false },
                    ],
                ));
            } else {
                content = content.push(empty_text("(none)"));
            }
        } else {
            content = content.push(empty_text("(caches not ready)"));
        }

        // NPCs table
        content = content.push(section_header(&format!("NPCs ({})", map.npcs.len())));
        if let Some(c) = caches {
            if !c.npcs.is_empty() {
                content = content.push(entity_table(
                    &c.npcs,
                    &c.npcs_indices,
                    vec![
                        TableColumn { width_px: 130.0, label: "Name".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 130.0, label: "Role".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 55.0, label: "DialogID".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 65.0, label: "PartyScript".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 70.0, label: "ShowOnEvent".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 55.0, label: "LookDir".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 200.0, label: "Waypoints".into(), sort: None, has_filter: false },
                    ],
                ));
            } else {
                content = content.push(empty_text("(none)"));
            }
        } else {
            content = content.push(empty_text("(caches not ready)"));
        }

        // Extra objects table
        content = content.push(section_header(&format!("Extra Objects ({})", map.extra_objects.len())));
        if let Some(c) = caches {
            if !c.extra_objects.is_empty() {
                content = content.push(entity_table(
                    &c.extra_objects,
                    &c.extra_objects_indices,
                    vec![
                        TableColumn { width_px: 130.0, label: "Name".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 50.0, label: "X".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 50.0, label: "Y".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 60.0, label: "Unk6".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 60.0, label: "Unk11".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 60.0, label: "Unk32".into(), sort: None, has_filter: false },
                    ],
                ));
            } else {
                content = content.push(empty_text("(none)"));
            }
        } else {
            content = content.push(empty_text("(caches not ready)"));
        }

        // Ground items
        content = content.push(section_header("Ground Items"));

        // Weapon items
        content = content.push(subsection_header(&format!(
            "Weapons ({})",
            map.draw_items_weapon.len()
        )));
        if let Some(c) = caches {
            if !c.draw_items_weapon.is_empty() {
                content = content.push(entity_table(
                    &c.draw_items_weapon,
                    &c.draw_items_weapon_indices,
                    vec![
                        TableColumn { width_px: 130.0, label: "Name".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 50.0, label: "Price".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 38.0, label: "Atk".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 38.0, label: "Def".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 50.0, label: "MagStr".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 90.0, label: "Coords".into(), sort: None, has_filter: false },
                    ],
                ));
            } else {
                content = content.push(empty_text("(none)"));
            }
        }

        // Heal items
        content = content.push(subsection_header(&format!(
            "Heals ({})",
            map.draw_items_heal.len()
        )));
        if let Some(c) = caches {
            if !c.draw_items_heal.is_empty() {
                content = content.push(entity_table(
                    &c.draw_items_heal,
                    &c.draw_items_heal_indices,
                    vec![
                        TableColumn { width_px: 130.0, label: "Name".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 50.0, label: "Price".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 38.0, label: "HP".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 38.0, label: "MP".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 90.0, label: "Coords".into(), sort: None, has_filter: false },
                    ],
                ));
            } else {
                content = content.push(empty_text("(none)"));
            }
        }

        // Edit items
        content = content.push(subsection_header(&format!(
            "Edits ({})",
            map.draw_items_edit.len()
        )));
        if let Some(c) = caches {
            if !c.draw_items_edit.is_empty() {
                content = content.push(entity_table(
                    &c.draw_items_edit,
                    &c.draw_items_edit_indices,
                    vec![
                        TableColumn { width_px: 130.0, label: "Name".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 50.0, label: "Price".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 38.0, label: "HP".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 38.0, label: "MP".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 38.0, label: "Str".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 38.0, label: "Agi".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 90.0, label: "Coords".into(), sort: None, has_filter: false },
                    ],
                ));
            } else {
                content = content.push(empty_text("(none)"));
            }
        }

        // Misc items
        content = content.push(subsection_header(&format!(
            "Misc ({})",
            map.draw_items_misc.len()
        )));
        if let Some(c) = caches {
            if !c.draw_items_misc.is_empty() {
                content = content.push(entity_table(
                    &c.draw_items_misc,
                    &c.draw_items_misc_indices,
                    vec![
                        TableColumn { width_px: 130.0, label: "Name".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 50.0, label: "Price".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 90.0, label: "Coords".into(), sort: None, has_filter: false },
                    ],
                ));
            } else {
                content = content.push(empty_text("(none)"));
            }
        }

        // Event items
        content = content.push(subsection_header(&format!(
            "Events ({})",
            map.draw_items_event.len()
        )));
        if let Some(c) = caches {
            if !c.draw_items_event.is_empty() {
                content = content.push(entity_table(
                    &c.draw_items_event,
                    &c.draw_items_event_indices,
                    vec![
                        TableColumn { width_px: 130.0, label: "Name".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 50.0, label: "Price".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 60.0, label: "EventID".into(), sort: None, has_filter: false },
                        TableColumn { width_px: 90.0, label: "Coords".into(), sort: None, has_filter: false },
                    ],
                ));
            } else {
                content = content.push(empty_text("(none)"));
            }
        }

        scrollable(content).height(Fill).into()
    } else {
        container(text("Select a map from the sidebar"))
            .width(Fill)
            .height(Fill)
            .padding(16)
            .into()
    };

    iced::widget::Row::<Message>::new()
        .push(sidebar)
        .push(main)
        .into()
}

fn section_header(label: &str) -> Element<'static, Message> {
    container(text(label.to_string()).size(16))
        .padding(Padding {
            top: 12.0,
            right: 0.0,
            bottom: 4.0,
            left: 0.0,
        })
        .width(Fill)
        .into()
}

fn subsection_header(label: &str) -> Element<'static, Message> {
    container(text(label.to_string()).size(13))
        .padding(Padding {
            top: 8.0,
            right: 0.0,
            bottom: 2.0,
            left: 8.0,
        })
        .width(Fill)
        .into()
}

fn empty_text(msg: &str) -> Element<'static, Message> {
    container(text(msg.to_string()).color(iced::Color::from_rgb(0.5, 0.5, 0.5)).size(12))
        .padding(Padding {
            top: 0.0,
            right: 0.0,
            bottom: 8.0,
            left: 12.0,
        })
        .width(Fill)
        .into()
}

fn entity_table<'a>(
    display_cache: &'a [Vec<String>],
    indices: &'a [usize],
    columns: Vec<TableColumn>,
) -> Element<'a, Message> {
    let table = TableWidget::new(
        display_cache,
        indices,
        columns,
        0.0,
        |_| gui_widgets::RowFlags::default(),
        22.0,
        ParagraphCache::default(),
    );
    // TableWidget::layout() fills whatever vertical space the parent offers,
    // so inside the outer scrollable each Fill-height table resolves to an
    // unbounded/zero height and renders wrong. Bound it explicitly.
    container(table).height(iced::Length::Fixed(640.0)).into()
}
