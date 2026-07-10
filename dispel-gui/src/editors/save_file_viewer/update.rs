use iced::Task;

use crate::app::App;
use crate::editors::save_file_viewer::message::SaveFileViewerMessage;
use crate::message::{Message, MessageExt};

pub fn handle(msg: SaveFileViewerMessage, app: &mut App) -> Task<Message> {
    let tab_id = match app.state.workspace.active() {
        Some(t) => t.id,
        None => return Task::none(),
    };

    let state = match app.state.editors.save_file_viewers.get_mut(&tab_id) {
        Some(s) => s,
        None => return Task::none(),
    };

    match msg {
        SaveFileViewerMessage::SelectSection(section) => {
            state.active_section = section;
            Task::none()
        }
        SaveFileViewerMessage::SelectCategory(cat) => {
            state.inventory_category = Some(cat);
            Task::none()
        }
        SaveFileViewerMessage::HexViewer(index, msg) => {
            if let Some(viewer) = state.raw_hex_viewers.get_mut(index) {
                hexedit::update(&mut viewer.state, &hexedit::HexEditorConfig::default(), msg)
                    .map(Message::hex_editor)
            } else {
                Task::none()
            }
        }
        SaveFileViewerMessage::SelectJournalSection(section) => {
            state.journal_section = section;
            state.selected_journal_entry = None;
            Task::none()
        }
        SaveFileViewerMessage::SelectMap(index) => {
            state.selected_map = Some(index);
            Task::none()
        }
        SaveFileViewerMessage::Load(_) => {
            // Load is handled by app.rs::open_file_in_workspace via Task::perform
            state.loading = true;
            Task::none()
        }
        SaveFileViewerMessage::Loaded(result) => {
            state.loading = false;
            match result {
                Ok(loaded) => {
                    state.save_file = Some(loaded.save_file.clone());
                    // Build events display cache
                    let n = loaded.save_file.events.len();
                    let mut display_cache = Vec::with_capacity(n);
                    for (i, ev) in loaded.save_file.events.iter().enumerate() {
                        display_cache.push(vec![
                            format!("{}", i + 1),
                            ev.unknown_1.to_string(),
                            ev.unknown_2.to_string(),
                            ev.script_name.clone(),
                        ]);
                    }
                    state.events_display_cache = display_cache;
                    state.events_filtered_indices = (0..n).collect();
                    state.raw_hex_viewers = loaded
                        .hex_editors
                        .into_iter()
                        .map(|d| {
                            use crate::editors::save_file_viewer::state::RawHexViewer;
                            let editor = hexedit::HexEditorState::from_bytes(
                                d.label,
                                d.data.clone(),
                                None,
                                None,
                            );
                            RawHexViewer {
                                label: d.label,
                                state: editor,
                            }
                        })
                        .collect();
                    // Build inventory display caches
                    use crate::editors::save_file_viewer::state::InventoryCategory;
                    let inv = &loaded.save_file.inventory;
                    let mut inv_caches = std::collections::HashMap::new();
                    inv_caches.insert(
                        InventoryCategory::Weapon,
                        inv.weapon_items
                            .iter()
                            .map(|item| {
                                vec![
                                    item.name.clone(),
                                    item.base_price.to_string(),
                                    item.attack.to_string(),
                                    item.defense.to_string(),
                                    item.magical_strength.to_string(),
                                    item.durability.to_string(),
                                    item.req_strength.to_string(),
                                    item.req_agility.to_string(),
                                    item.req_wisdom.to_string(),
                                    item.health_points.to_string(),
                                    item.mana_points.to_string(),
                                ]
                            })
                            .collect(),
                    );
                    inv_caches.insert(
                        InventoryCategory::Heal,
                        inv.heal_items
                            .iter()
                            .map(|item| {
                                vec![
                                    item.name.clone(),
                                    item.base_price.to_string(),
                                    item.health_points.to_string(),
                                    item.mana_points.to_string(),
                                    bool_yesno(item.restore_full_health),
                                    bool_yesno(item.restore_full_mana),
                                    bool_yesno(item.poison_heal),
                                    bool_yesno(item.petrif_heal),
                                ]
                            })
                            .collect(),
                    );
                    inv_caches.insert(
                        InventoryCategory::Edit,
                        inv.edit_items
                            .iter()
                            .map(|item| {
                                vec![
                                    item.name.clone(),
                                    item.base_price.to_string(),
                                    item.health_points.to_string(),
                                    item.mana_points.to_string(),
                                    item.strength.to_string(),
                                    item.agility.to_string(),
                                    item.wisdom.to_string(),
                                    item.constitution.to_string(),
                                    item.offense.to_string(),
                                    item.defense.to_string(),
                                    item.magical_power.to_string(),
                                ]
                            })
                            .collect(),
                    );
                    inv_caches.insert(
                        InventoryCategory::Event,
                        inv.event_items
                            .iter()
                            .map(|item| {
                                vec![
                                    item.name.clone(),
                                    item.base_price.to_string(),
                                    item.event_item_id.to_string(),
                                ]
                            })
                            .collect(),
                    );
                    inv_caches.insert(
                        InventoryCategory::Misc,
                        inv.misc_items
                            .iter()
                            .map(|item| {
                                vec![
                                    item.name.clone(),
                                    item.base_price.to_string(),
                                ]
                            })
                            .collect(),
                    );
                    state.inventory_display_caches = inv_caches;
                    state.inventory_filtered_indices = state
                        .inventory_display_caches
                        .iter()
                        .map(|(cat, rows)| {
                            let indices: Vec<usize> = (0..rows.len()).collect();
                            (*cat, indices)
                        })
                        .collect();
                    // Build maps display caches
                    let maps_caches: Vec<crate::editors::save_file_viewer::state::MapsDisplayCaches> = loaded
                        .save_file
                        .maps
                        .iter()
                        .map(|map| {
                            let n_monsters = map.monsters.len();
                            let n_npcs = map.npcs.len();
                            let n_extras = map.extra_objects.len();
                            let n_dw = map.draw_items_weapon.len();
                            let n_dh = map.draw_items_heal.len();
                            let n_de = map.draw_items_edit.len();
                            let n_dm = map.draw_items_misc.len();
                            let n_dev = map.draw_items_event.len();
                            use crate::editors::save_file_viewer::state::MapsDisplayCaches;
                            MapsDisplayCaches {
                                monsters: map.monsters.iter().map(|m| {
                                    vec![
                                        m.name.clone(),
                                        format!("{}/{}", m.hp_current, m.hp_maximum),
                                        format!("{}/{}", m.mp_current, m.mp_maximum),
                                        m.offense_rate.to_string(),
                                        m.defense_rate.to_string(),
                                        format!("{}%", m.dodge_rate),
                                        format!("{}%", m.hit_rate),
                                        m.experience_on_kill.to_string(),
                                        m.gold_drop_on_kill.to_string(),
                                        m.sight_range.to_string(),
                                        m.attack_range.to_string(),
                                        ai_type_label(m.monster_ai_type),
                                        m.unknown_6_coordinate.to_string(),
                                        m.unknown_7_coordinate.to_string(),
                                    ]
                                }).collect(),
                                monsters_indices: (0..n_monsters).collect(),
                                npcs: map.npcs.iter().map(|n| {
                                    vec![
                                        n.name.clone(),
                                        n.role_description.clone(),
                                        n.npc_ref_dialog_id.to_string(),
                                        n.npc_ref_party_script_id.to_string(),
                                        n.npc_ref_show_on_event_id.to_string(),
                                        n.npc_ref_look_direction.to_string(),
                                        format_npc_waypoints(n),
                                    ]
                                }).collect(),
                                npcs_indices: (0..n_npcs).collect(),
                                extra_objects: map.extra_objects.iter().map(|e| {
                                    vec![
                                        e.name.clone(),
                                        e.unknown_7.to_string(),
                                        e.unknown_8.to_string(),
                                        format!("0x{:X}", e.unknown_6),
                                        e.unknown_11.to_string(),
                                        e.unknown_32.to_string(),
                                    ]
                                }).collect(),
                                extra_objects_indices: (0..n_extras).collect(),
                                draw_items_weapon: map.draw_items_weapon.iter().map(|d| {
                                    vec![
                                        d.name.clone(),
                                        d.base_price.to_string(),
                                        d.attack.to_string(),
                                        d.defense.to_string(),
                                        d.magical_strength.to_string(),
                                        fmt_coord(d.map_coordinate_x, d.map_coordinate_y),
                                    ]
                                }).collect(),
                                draw_items_weapon_indices: (0..n_dw).collect(),
                                draw_items_heal: map.draw_items_heal.iter().map(|d| {
                                    vec![
                                        d.name.clone(),
                                        d.base_price.to_string(),
                                        d.health_points.to_string(),
                                        d.mana_points.to_string(),
                                        fmt_coord(d.map_coordinate_x, d.map_coordinate_y),
                                    ]
                                }).collect(),
                                draw_items_heal_indices: (0..n_dh).collect(),
                                draw_items_edit: map.draw_items_edit.iter().map(|d| {
                                    vec![
                                        d.name.clone(),
                                        d.base_price.to_string(),
                                        d.health_points.to_string(),
                                        d.mana_points.to_string(),
                                        d.strength.to_string(),
                                        d.agility.to_string(),
                                        fmt_coord(d.map_coordinate_x, d.map_coordinate_y),
                                    ]
                                }).collect(),
                                draw_items_edit_indices: (0..n_de).collect(),
                                draw_items_misc: map.draw_items_misc.iter().map(|d| {
                                    vec![
                                        d.name.clone(),
                                        d.base_price.to_string(),
                                        fmt_coord(d.unknown_4, d.unknown_5),
                                    ]
                                }).collect(),
                                draw_items_misc_indices: (0..n_dm).collect(),
                                draw_items_event: map.draw_items_event.iter().map(|d| {
                                    vec![
                                        d.name.clone(),
                                        d.base_price.to_string(),
                                        d.event_item_id.to_string(),
                                        fmt_coord(d.map_coordinate_x, d.map_coordinate_y),
                                    ]
                                }).collect(),
                                draw_items_event_indices: (0..n_dev).collect(),
                            }
                        })
                        .collect();
                    state.maps_display_caches = maps_caches;
                    // Build journal display caches
                    use crate::editors::save_file_viewer::state::JournalSection;
                    let mut journal_caches =
                        std::collections::HashMap::<JournalSection, Vec<Vec<String>>>::new();
                    let mut journal_indices =
                        std::collections::HashMap::<JournalSection, Vec<usize>>::new();
                    for (section, entries) in [
                        (JournalSection::Main, &loaded.save_file.journal.main),
                        (JournalSection::Side, &loaded.save_file.journal.side),
                        (JournalSection::Trade, &loaded.save_file.journal.trade),
                    ] {
                        let cache: Vec<Vec<String>> = entries
                            .iter()
                            .map(|entry| {
                                let hex_rest: Vec<String> = entry.rest.iter().map(|b| format!("{:02X}", b)).collect();
                                vec![
                                    format!("{}", entry.index),
                                    entry.name.clone(),
                                    hex_rest.join(" "),
                                ]
                            })
                            .collect();
                        let indices: Vec<usize> = (0..cache.len()).collect();
                        journal_caches.insert(section, cache);
                        journal_indices.insert(section, indices);
                    }
                    state.journal_display_caches = journal_caches;
                    state.journal_filtered_indices = journal_indices;
                    state.error = None;
                }
                Err(e) => {
                    state.error = Some(e);
                }
            }
            Task::none()
        }
    }
}

fn bool_yesno(v: u8) -> String {
    if v != 0 { "Yes".into() } else { "No".into() }
}

fn ai_type_label(ai: u8) -> String {
    match ai {
        0 => "Passive".into(),
        1 => "Aggressive".into(),
        2 => "Defensive".into(),
        _ => format!("AI({})", ai),
    }
}

fn format_npc_waypoints(
    n: &dispel_core::references::save_file::NpcRecord,
) -> String {
    let mut parts = Vec::new();
    if n.npc_ref_waypoint1filled != 0 {
        parts.push(format!("WP1:({},{})", n.npc_ref_waypoint1x, n.npc_ref_waypoint1y));
    }
    if n.npc_ref_waypoint2filled != 0 {
        parts.push(format!("WP2:({},{})", n.npc_ref_waypoint2x, n.npc_ref_waypoint2y));
    }
    if n.npc_ref_waypoint3filled != 0 {
        parts.push(format!("WP3:({},{})", n.npc_ref_waypoint3x, n.npc_ref_waypoint3y));
    }
    if n.npc_ref_waypoint4filled != 0 {
        parts.push(format!("WP4:({},{})", n.npc_ref_waypoint4x, n.npc_ref_waypoint4y));
    }
    if parts.is_empty() {
        "(none)".into()
    } else {
        parts.join(" ")
    }
}

fn fmt_coord(x: impl std::fmt::Display, y: impl std::fmt::Display) -> String {
    format!("({},{})", x, y)
}
