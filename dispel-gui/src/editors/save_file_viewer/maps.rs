use iced::widget::{button, container, text, Row};
use iced::{Element, Fill};

use crate::editors::save_file_viewer::state::SaveFileViewerState;
use crate::message::Message;
use crate::message::MessageExt;

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
        let mut col = iced::widget::Column::<Message>::new().spacing(2).padding(8).width(120);
        for (i, map) in sf.maps.iter().enumerate() {
            let is_active = state.selected_map == Some(i);
            let mut btn = button(text(format!("Map {}", map.map_id)).size(12));
            if is_active {
                btn = btn.style(iced::widget::button::primary);
            }
            col = col.push(
                btn.on_press(Message::save_file_viewer(
                    crate::editors::save_file_viewer::SaveFileViewerMessage::SelectMap(i),
                ))
                .padding([4, 8])
                .width(Fill),
            );
        }
        iced::widget::Scrollable::<Message>::new(col).height(Fill).into()
    };

    // Right panel: entity tables for selected map
    let main: Element<'a, Message> = if let Some(idx) = state.selected_map {
        let map = &sf.maps[idx];

        let mut content = iced::widget::Column::<Message>::new().spacing(12).padding(16);

        // Monster table
        content = content.push(text(format!("Monsters ({})", map.monsters.len())).size(14));
        if map.monsters.is_empty() {
            content = content.push(text("(none)").color(iced::Color::from_rgb(0.5, 0.5, 0.5)).size(12));
        } else {
            for m in &map.monsters {
                let row = Row::<Message>::new()
                    .spacing(8)
                    .push(text(m.name.clone()).size(12).width(150))
                    .push(text(format!("HP {}/{}", m.hp_current, m.hp_maximum)).size(12).width(100))
                    .push(text(format!("({},{})", m.tile_x, m.tile_y)).size(11).width(80))
                    .push(text(state_flags(&m.state)).size(11));
                content = content.push(container(row).padding([2, 8]).width(Fill));
            }
        }

        // NPC table
        content = content.push(text(format!("NPCs ({})", map.npcs.len())).size(14));
        if map.npcs.is_empty() {
            content = content.push(text("(none)").color(iced::Color::from_rgb(0.5, 0.5, 0.5)).size(12));
        } else {
            for npc in &map.npcs {
                let row = Row::<Message>::new()
                    .spacing(8)
                    .push(text(npc.name.clone()).size(12).width(150))
                    .push(text(npc.role_description.clone()).size(12));
                content = content.push(container(row).padding([2, 8]).width(Fill));
            }
        }

        // Extra objects
        content = content.push(text(format!("Extra Objects ({})", map.extra_objects.len())).size(14));
        if map.extra_objects.is_empty() {
            content = content.push(text("(none)").color(iced::Color::from_rgb(0.5, 0.5, 0.5)).size(12));
        } else {
            for obj in &map.extra_objects {
                let row = Row::<Message>::new()
                    .spacing(8)
                    .push(text(obj.name.clone()).size(12).width(150))
                    .push(text(format!("state={}", obj.state)).size(12));
                content = content.push(container(row).padding([2, 8]).width(Fill));
            }
        }

        // Draw items summary
        content = content.push(text("Ground Items").size(14));
        let draw_counts = [
            ("Weapon", map.draw_items_weapon.len(), 296),
            ("Heal", map.draw_items_heal.len(), 264),
            ("Edit", map.draw_items_edit.len(), 280),
            ("Misc", map.draw_items_misc.len(), 268),
            ("Event", map.draw_items_event.len(), 252),
        ];
        for (label, total_bytes, rec_size) in &draw_counts {
            let count = if *total_bytes > 0 { total_bytes / rec_size } else { 0 };
            content = content.push(
                text(format!("  {}: {} records ({} bytes)", label, count, total_bytes))
                    .size(12)
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            );
        }

        iced::widget::Scrollable::<Message>::new(content).height(Fill).into()
    } else {
        container(text("Select a map from the sidebar"))
            .width(Fill)
            .height(Fill)
            .padding(16)
            .into()
    };

    Row::<Message>::new()
        .push(sidebar)
        .push(main)
        .into()
}

fn state_flags(state: &dispel_core::references::save_file::MonsterState) -> String {
    let mut parts = Vec::new();
    if state.is_dead { parts.push("Dead"); }
    if state.is_poisoned { parts.push("Poisoned"); }
    if state.is_burning { parts.push("Burning"); }
    if state.is_frozen { parts.push("Frozen"); }
    if state.is_stunned { parts.push("Stunned"); }
    if state.is_invisible { parts.push("Invisible"); }
    if state.is_boss { parts.push("Boss"); }
    if parts.is_empty() {
        "Alive".into()
    } else {
        parts.join(", ")
    }
}
