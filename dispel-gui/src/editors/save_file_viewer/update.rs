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
        SaveFileViewerMessage::MapsTableSelect {
            map,
            kind,
            visible_idx,
        } => {
            let Some(cache) = state.maps_display_caches.get(map) else {
                return Task::none();
            };
            let Some(ts) = state
                .maps_table_states
                .get_mut(map)
                .and_then(|m| m.get_mut(&kind))
            else {
                return Task::none();
            };
            let orig = maps_table_indices(cache, kind).get(visible_idx).copied();
            ts.selected_orig = orig;
            Task::none()
        }
        SaveFileViewerMessage::MapsTableSort { map, kind, col } => {
            let Some(cache) = state.maps_display_caches.get_mut(map) else {
                return Task::none();
            };
            let Some(ts) = state
                .maps_table_states
                .get_mut(map)
                .and_then(|m| m.get_mut(&kind))
            else {
                return Task::none();
            };
            if ts.sort_column == Some(col) {
                ts.sort_ascending = !ts.sort_ascending;
            } else {
                ts.sort_column = Some(col);
                ts.sort_ascending = true;
            }
            let ascending = ts.sort_ascending;
            let (rows, indices) = maps_table_data(cache, kind);
            indices.sort_by(|&a, &b| compare_cells(rows, a, b, col, ascending));
            Task::none()
        }
        SaveFileViewerMessage::MapsTableStartResize { map, kind, col } => {
            let anchor_width = state
                .maps_table_states
                .get(map)
                .and_then(|m| m.get(&kind))
                .and_then(|ts| ts.column_widths.get(col).copied())
                .unwrap_or(80.0);
            state.maps_resizing = Some(
                crate::editors::save_file_viewer::state::MapsTableResizeDrag {
                    map,
                    kind,
                    col,
                    anchor_width,
                    anchor_cursor_x: None,
                },
            );
            Task::none()
        }
        SaveFileViewerMessage::MapsTableResetColumnWidth { map, kind, col } => {
            if let Some(ts) = state
                .maps_table_states
                .get_mut(map)
                .and_then(|m| m.get_mut(&kind))
            {
                let default_width = kind
                    .default_columns()
                    .into_iter()
                    .nth(col)
                    .map(|c| c.width_px)
                    .unwrap_or(80.0);
                if let Some(w) = ts.column_widths.get_mut(col) {
                    *w = default_width;
                }
            }
            Task::none()
        }
        SaveFileViewerMessage::MapsTableResizeCursor(x) => {
            if let Some(drag) = state.maps_resizing.as_mut() {
                let anchor_x = match drag.anchor_cursor_x {
                    Some(ax) => ax,
                    None => {
                        drag.anchor_cursor_x = Some(x);
                        return Task::none();
                    }
                };
                let new_width = (drag.anchor_width + (x - anchor_x))
                    .clamp(COL_WIDTH_MIN, COL_WIDTH_MAX);
                if let Some(ts) = state
                    .maps_table_states
                    .get_mut(drag.map)
                    .and_then(|m| m.get_mut(&drag.kind))
                {
                    if let Some(w) = ts.column_widths.get_mut(drag.col) {
                        *w = new_width;
                    }
                }
            }
            Task::none()
        }
        SaveFileViewerMessage::MapsTableEndResize => {
            state.maps_resizing = None;
            Task::none()
        }
        SaveFileViewerMessage::MapsTableScroll { map, kind, x, y, .. } => {
            if let Some(ts) = state
                .maps_table_states
                .get_mut(map)
                .and_then(|m| m.get_mut(&kind))
            {
                ts.scroll_offset = (x, y);
            }
            Task::none()
        }
        SaveFileViewerMessage::InventoryTableSelect {
            cat,
            visible_idx,
        } => {
            if let Some(indices) = state.inventory_filtered_indices.get(&cat) {
                let orig = indices.get(visible_idx).copied();
                if let Some(ts) = state.inventory_table_states.get_mut(&cat) {
                    ts.selected_orig = orig;
                }
            }
            Task::none()
        }
        SaveFileViewerMessage::InventoryTableSort { cat, col } => {
            let Some(ts) = state.inventory_table_states.get_mut(&cat) else {
                return Task::none();
            };
            if ts.sort_column == Some(col) {
                ts.sort_ascending = !ts.sort_ascending;
            } else {
                ts.sort_column = Some(col);
                ts.sort_ascending = true;
            }
            let ascending = ts.sort_ascending;
            let (rows, indices) = inventory_table_data(
                &mut state.inventory_display_caches,
                &mut state.inventory_filtered_indices,
                cat,
            );
            indices.sort_by(|&a, &b| compare_cells(rows, a, b, col, ascending));
            Task::none()
        }
        SaveFileViewerMessage::InventoryTableStartResize { cat, col } => {
            let anchor_width = state
                .inventory_table_states
                .get(&cat)
                .and_then(|ts| ts.column_widths.get(col).copied())
                .unwrap_or(80.0);
            state.inventory_resizing = Some(
                crate::editors::save_file_viewer::state::InventoryResizeDrag {
                    cat,
                    col,
                    anchor_width,
                    anchor_cursor_x: None,
                },
            );
            Task::none()
        }
        SaveFileViewerMessage::InventoryTableResetColumnWidth { cat, col } => {
            if let Some(ts) = state.inventory_table_states.get_mut(&cat) {
                let default_width = cat
                    .default_columns()
                    .into_iter()
                    .nth(col)
                    .map(|c| c.width_px)
                    .unwrap_or(80.0);
                if let Some(w) = ts.column_widths.get_mut(col) {
                    *w = default_width;
                }
            }
            Task::none()
        }
        SaveFileViewerMessage::InventoryTableResizeCursor(x) => {
            if let Some(drag) = state.inventory_resizing.as_mut() {
                let anchor_x = match drag.anchor_cursor_x {
                    Some(ax) => ax,
                    None => {
                        drag.anchor_cursor_x = Some(x);
                        return Task::none();
                    }
                };
                let new_width = (drag.anchor_width + (x - anchor_x))
                    .clamp(COL_WIDTH_MIN, COL_WIDTH_MAX);
                if let Some(ts) = state.inventory_table_states.get_mut(&drag.cat) {
                    if let Some(w) = ts.column_widths.get_mut(drag.col) {
                        *w = new_width;
                    }
                }
            }
            Task::none()
        }
        SaveFileViewerMessage::InventoryTableEndResize => {
            state.inventory_resizing = None;
            Task::none()
        }
        SaveFileViewerMessage::InventoryTableScroll {
            cat,
            x,
            y,
            ..
        } => {
            if let Some(ts) = state.inventory_table_states.get_mut(&cat) {
                ts.scroll_offset = (x, y);
            }
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
                    for ev in loaded.save_file.events.iter() {
                        display_cache.push(vec![
                            ev.event_id.to_string(),
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
                                    item.description.clone(),
                                    item.base_price.to_string(),
                                    item.weapon_item_id.to_string(),
                                    item.health_points.to_string(),
                                    item.mana_points.to_string(),
                                    item.strength.to_string(),
                                    item.agility.to_string(),
                                    item.wisdom.to_string(),
                                    item.constitution.to_string(),
                                    item.to_dodge.to_string(),
                                    item.to_hit.to_string(),
                                    item.attack.to_string(),
                                    item.defense.to_string(),
                                    item.magical_strength.to_string(),
                                    item.durability.to_string(),
                                    item.padding2.to_string(),
                                    item.padding3.to_string(),
                                    item.req_strength.to_string(),
                                    item.padding4.to_string(),
                                    item.req_agility.to_string(),
                                    item.padding5.to_string(),
                                    item.req_wisdom.to_string(),
                                    item.padding6.to_string(),
                                    item.padding7.to_string(),
                                    item.padding8.to_string(),
                                    item.unknown_1.to_string(),
                                    item.unknown_2.to_string(),
                                    item.unknown_3.to_string(),
                                    item.unknown_4.to_string(),
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
                                    item.description.clone(),
                                    item.base_price.to_string(),
                                    item.heal_item_id.to_string(),
                                    item.health_points.to_string(),
                                    item.mana_points.to_string(),
                                    item.restore_full_health.to_string(),
                                    item.restore_full_mana.to_string(),
                                    item.poison_heal.to_string(),
                                    item.petrif_heal.to_string(),
                                    item.polimorph_heal.to_string(),
                                    item.unknown_1.to_string(),
                                    item.unknown_2.to_string(),
                                    item.unknown_3.to_string(),
                                    item.unknown_4.to_string(),
                                    item.unknown_5.to_string(),
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
                                    item.description.clone(),
                                    item.base_price.to_string(),
                                    item.unknown_1.to_string(),
                                    item.unknown_2.to_string(),
                                    item.health_points.to_string(),
                                    item.mana_points.to_string(),
                                    item.strength.to_string(),
                                    item.agility.to_string(),
                                    item.wisdom.to_string(),
                                    item.constitution.to_string(),
                                    item.to_dodge.to_string(),
                                    item.to_hit.to_string(),
                                    item.offense.to_string(),
                                    item.defense.to_string(),
                                    item.magical_power.to_string(),
                                    item.item_destroying_power.to_string(),
                                    item.unknown_3.to_string(),
                                    item.modifies_item.to_string(),
                                    item.additional_effect.to_string(),
                                    item.unknown_4.to_string(),
                                    item.unknown_5.to_string(),
                                    item.unknown_6.to_string(),
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
                                    item.description.clone(),
                                    item.base_price.to_string(),
                                    item.event_item_id.to_string(),
                                    item.unknown_2.to_string(),
                                    item.unknown_3.to_string(),
                                    item.unknown_4.to_string(),
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
                                    item.description.clone(),
                                    item.base_price.to_string(),
                                    hex_bytes(&item.unknown_1),
                                    item.unknown_2.to_string(),
                                    item.unknown_3.to_string(),
                                    item.unknown_4.to_string(),
                                    item.unknown_5.to_string(),
                                    item.unknown_6.to_string(),
                                    item.unknown_7.to_string(),
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
                    // Build per-category inventory table interaction state.
                    // Column widths are initialised from each category's
                    // default column layout.
                    use crate::editors::save_file_viewer::state::{
                        InventoryTableState,
                    };
                    let mut inv_states: std::collections::HashMap<
                        InventoryCategory,
                        InventoryTableState,
                    > = std::collections::HashMap::new();
                    for cat in state.inventory_display_caches.keys() {
                        let widths: Vec<f32> = cat
                            .default_columns()
                            .iter()
                            .map(|c| c.width_px)
                            .collect();
                        inv_states.insert(
                            *cat,
                            InventoryTableState {
                                column_widths: widths,
                                ..Default::default()
                            },
                        );
                    }
                    state.inventory_table_states = inv_states;
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
                                        m.signature_a.to_string(),
                                        m.record_index.to_string(),
                                        m.signature_b.to_string(),
                                        m.name.clone(),
                                        m.monster_db_id.to_string(),
                                        m.hp_current.to_string(),
                                        m.hp_maximum.to_string(),
                                        m.mp_current.to_string(),
                                        m.mp_maximum.to_string(),
                                        m.walk_speed.to_string(),
                                        m.hit_rate.to_string(),
                                        m.dodge_rate.to_string(),
                                        m.offense_rate.to_string(),
                                        m.defense_rate.to_string(),
                                        m.magic_rate.to_string(),
                                        m.is_undead.to_string(),
                                        m.has_blood.to_string(),
                                        m.monster_ai_type.to_string(),
                                        m.experience_on_kill.to_string(),
                                        m.gold_drop_on_kill.to_string(),
                                        m.unknown_1.to_string(),
                                        m.sight_range.to_string(),
                                        m.attack_range.to_string(),
                                        m.spell_slot_1.to_string(),
                                        m.spell_slot_2.to_string(),
                                        m.spell_slot_3.to_string(),
                                        m.oversize.to_string(),
                                        m.magic_level.to_string(),
                                        m.unknown_2.to_string(),
                                        hex_bytes(&m.unknown_3),
                                        m.unknown_4.to_string(),
                                        m.unknown_5.to_string(),
                                        m.unknown_6_coordinate.to_string(),
                                        m.unknown_7_coordinate.to_string(),
                                        m.unknown_8_coordinate.to_string(),
                                        m.unknown_9_coordinate.to_string(),
                                        m.unknown_10_coordinate.to_string(),
                                        m.unknown_11_coordinate.to_string(),
                                        m.unknown_12.to_string(),
                                        m.unknown_13.to_string(),
                                        m.unknown_14.to_string(),
                                        m.unknown_15.to_string(),
                                        m.unknown_16.to_string(),
                                        m.unknown_17.to_string(),
                                        m.unknown_18.to_string(),
                                        hex_bytes(&m.unknown_19),
                                        m.unknown_20.to_string(),
                                        m.unknown_21.to_string(),
                                        m.unknown_22.to_string(),
                                        m.loot_item1.to_string(),
                                        m.loot_item2.to_string(),
                                        m.loot_item3.to_string(),
                                        m.mon_ref_padding_12.to_string(),
                                        m.mon_ref_padding_13.to_string(),
                                        m.unknown_23.to_string(),
                                        m.unknown_24.to_string(),
                                        m.unknown_25.to_string(),
                                        m.unknown_26.to_string(),
                                        m.special_attack_chance.to_string(),
                                        m.special_attack_duration.to_string(),
                                        hex_bytes(&m.unknown_27),
                                        m.boldness.to_string(),
                                        m.attack_speed.to_string(),
                                        hex_bytes(&m.unknown_28),
                                        m.unknown_29.to_string(),
                                        hex_bytes(&m.unknown_30),
                                    ]
                                }).collect(),
                                monsters_indices: (0..n_monsters).collect(),
                                npcs: map.npcs.iter().map(|n| {
                                    vec![
                                        n.name.clone(),
                                        n.role_description.clone(),
                                        n.unknown1.to_string(),
                                        n.unknown2.to_string(),
                                        n.unknown3.to_string(),
                                        n.unknown4.to_string(),
                                        n.unknown5.to_string(),
                                        n.unknown6.to_string(),
                                        n.unknown7.to_string(),
                                        n.unknown8.to_string(),
                                        n.unknown9.to_string(),
                                        n.unknown10.to_string(),
                                        n.unknown11.to_string(),
                                        hex_bytes(&n.unknown12),
                                        n.npc_ini_id.to_string(),
                                        hex_bytes(&n.unknown13),
                                        n.npc_ref_party_script_id.to_string(),
                                        n.npc_ref_show_on_event_id.to_string(),
                                        n.unknown14.to_string(),
                                        n.npc_ref_unknown_1.to_string(),
                                        n.npc_ref_waypoint1filled.to_string(),
                                        n.npc_ref_waypoint1x.to_string(),
                                        n.npc_ref_waypoint1y.to_string(),
                                        n.npc_ref_unknown_2.to_string(),
                                        n.npc_ref_look_direction.to_string(),
                                        n.npc_ref_unknown_9.to_string(),
                                        n.npc_ref_waypoint2filled.to_string(),
                                        n.npc_ref_waypoint2x.to_string(),
                                        n.npc_ref_waypoint2y.to_string(),
                                        n.npc_ref_unknown_3.to_string(),
                                        n.npc_ref_unknown_6.to_string(),
                                        n.npc_ref_unknown_10.to_string(),
                                        n.npc_ref_waypoint3filled.to_string(),
                                        n.npc_ref_waypoint3x.to_string(),
                                        n.npc_ref_waypoint3y.to_string(),
                                        n.npc_ref_unknown_4.to_string(),
                                        n.npc_ref_unknown_7.to_string(),
                                        n.npc_ref_unknown_11.to_string(),
                                        n.npc_ref_waypoint4filled.to_string(),
                                        n.npc_ref_waypoint4x.to_string(),
                                        n.npc_ref_waypoint4y.to_string(),
                                        n.npc_ref_unknown_5.to_string(),
                                        n.npc_ref_unknown_8.to_string(),
                                        n.npc_ref_unknown_12.to_string(),
                                        n.npc_ref_unknown_13.to_string(),
                                        n.npc_ref_unknown_14.to_string(),
                                        n.npc_ref_unknown_15.to_string(),
                                        n.npc_ref_unknown_16.to_string(),
                                        n.npc_ref_unknown_17.to_string(),
                                        n.unknown15.to_string(),
                                        n.npc_ref_dialog_id.to_string(),
                                        hex_bytes(&n.unknown16),
                                    ]
                                }).collect(),
                                npcs_indices: (0..n_npcs).collect(),
                                extra_objects: map.extra_objects.iter().map(|e| {
                                    vec![
                                        e.unknown_1.to_string(),
                                        e.unknown_2.to_string(),
                                        e.unknown_3.to_string(),
                                        e.unknown_4.to_string(),
                                        e.unknown_5.to_string(),
                                        e.name.clone(),
                                        e.unknown_6.to_string(),
                                        e.unknown_7.to_string(),
                                        e.unknown_8.to_string(),
                                        e.unknown_9.to_string(),
                                        hex_bytes(&e.unknown_10),
                                        e.unknown_11.to_string(),
                                        e.unknown_12.to_string(),
                                        e.unknown_13.to_string(),
                                        e.unknown_14.to_string(),
                                        e.unknown_15.to_string(),
                                        e.unknown_16.to_string(),
                                        e.unknown_17.to_string(),
                                        e.unknown_18.to_string(),
                                        e.unknown_19.to_string(),
                                        e.unknown_20.to_string(),
                                        e.unknown_21.to_string(),
                                        e.unknown_22.to_string(),
                                        hex_bytes(&e.unknown_23),
                                        e.unknown_24.to_string(),
                                        e.unknown_25.to_string(),
                                        e.unknown_26.to_string(),
                                        e.unknown_27.to_string(),
                                        e.unknown_28.to_string(),
                                        e.unknown_29.to_string(),
                                        hex_bytes(&e.unknown_30),
                                        hex_bytes(&e.unknown_31),
                                        e.unknown_32.to_string(),
                                        e.unknown_33.to_string(),
                                        e.unknown_34.to_string(),
                                        e.unknown_35.to_string(),
                                        e.unknown_36.to_string(),
                                        e.unknown_37.to_string(),
                                        e.unknown_38.to_string(),
                                    ]
                                }).collect(),
                                extra_objects_indices: (0..n_extras).collect(),
                                draw_items_weapon: map.draw_items_weapon.iter().map(|d| {
                                    vec![
                                        d.name.clone(),
                                        d.description.clone(),
                                        d.base_price.to_string(),
                                        d.weapon_item_id.to_string(),
                                        d.health_points.to_string(),
                                        d.mana_points.to_string(),
                                        d.strength.to_string(),
                                        d.agility.to_string(),
                                        d.wisdom.to_string(),
                                        d.constitution.to_string(),
                                        d.to_dodge.to_string(),
                                        d.to_hit.to_string(),
                                        d.attack.to_string(),
                                        d.defense.to_string(),
                                        d.magical_strength.to_string(),
                                        d.durability.to_string(),
                                        d.padding2.to_string(),
                                        d.padding3.to_string(),
                                        d.req_strength.to_string(),
                                        d.padding4.to_string(),
                                        d.req_agility.to_string(),
                                        d.padding5.to_string(),
                                        d.req_wisdom.to_string(),
                                        d.padding6.to_string(),
                                        d.padding7.to_string(),
                                        d.padding8.to_string(),
                                        d.map_coordinate_x.to_string(),
                                        d.map_coordinate_y.to_string(),
                                        d.unknown_1.to_string(),
                                    ]
                                }).collect(),
                                draw_items_weapon_indices: (0..n_dw).collect(),
                                draw_items_heal: map.draw_items_heal.iter().map(|d| {
                                    vec![
                                        d.name.clone(),
                                        d.description.clone(),
                                        d.base_price.to_string(),
                                        d.heal_item_id.to_string(),
                                        d.health_points.to_string(),
                                        d.mana_points.to_string(),
                                        d.restore_full_health.to_string(),
                                        d.restore_full_mana.to_string(),
                                        d.poison_heal.to_string(),
                                        d.petrif_heal.to_string(),
                                        d.polimorph_heal.to_string(),
                                        d.unknown_1.to_string(),
                                        d.unknown_2.to_string(),
                                        d.map_coordinate_x.to_string(),
                                        d.map_coordinate_y.to_string(),
                                        d.unknown_3.to_string(),
                                    ]
                                }).collect(),
                                draw_items_heal_indices: (0..n_dh).collect(),
                                draw_items_edit: map.draw_items_edit.iter().map(|d| {
                                    vec![
                                        d.name.clone(),
                                        d.description.clone(),
                                        d.base_price.to_string(),
                                        d.edit_item_id.to_string(),
                                        d.health_points.to_string(),
                                        d.mana_points.to_string(),
                                        d.strength.to_string(),
                                        d.agility.to_string(),
                                        d.wisdom.to_string(),
                                        d.constitution.to_string(),
                                        d.to_dodge.to_string(),
                                        d.to_hit.to_string(),
                                        d.offense.to_string(),
                                        d.defense.to_string(),
                                        d.magical_power.to_string(),
                                        d.item_destroying_power.to_string(),
                                        d.unknown_3.to_string(),
                                        d.modifies_item.to_string(),
                                        d.additional_effect.to_string(),
                                        d.map_coordinate_x.to_string(),
                                        d.map_coordinate_y.to_string(),
                                        d.unknown_4.to_string(),
                                    ]
                                }).collect(),
                                draw_items_edit_indices: (0..n_de).collect(),
                                draw_items_misc: map.draw_items_misc.iter().map(|d| {
                                    vec![
                                        d.name.clone(),
                                        d.description.clone(),
                                        d.base_price.to_string(),
                                        hex_bytes(&d.unknown_1),
                                        d.unknown_2.to_string(),
                                        d.unknown_3.to_string(),
                                        d.unknown_4.to_string(),
                                        d.unknown_5.to_string(),
                                        d.unknown_7.to_string(),
                                    ]
                                }).collect(),
                                draw_items_misc_indices: (0..n_dm).collect(),
                                draw_items_event: map.draw_items_event.iter().map(|d| {
                                    vec![
                                        d.name.clone(),
                                        d.description.clone(),
                                        d.base_price.to_string(),
                                        d.event_item_id.to_string(),
                                        d.map_coordinate_x.to_string(),
                                        d.map_coordinate_y.to_string(),
                                        d.unknown_1.to_string(),
                                    ]
                                }).collect(),
                                draw_items_event_indices: (0..n_dev).collect(),
                            }
                        })
                        .collect();
                    state.maps_display_caches = maps_caches;
                    // Build per-map, per-table interaction state. Column widths
                    // are initialised from each table kind's default layout.
                    use crate::editors::save_file_viewer::state::{
                        MapTableState, MapsTableKind,
                    };
                    let mut table_states: Vec<
                        std::collections::HashMap<MapsTableKind, MapTableState>,
                    > = Vec::with_capacity(state.maps_display_caches.len());
                    for _ in &state.maps_display_caches {
                        let mut per_map = std::collections::HashMap::new();
                        for kind in MapsTableKind::all() {
                            let widths: Vec<f32> = kind
                                .default_columns()
                                .iter()
                                .map(|c| c.width_px)
                                .collect();
                            per_map.insert(
                                *kind,
                                MapTableState {
                                    column_widths: widths,
                                    ..Default::default()
                                },
                            );
                        }
                        table_states.push(per_map);
                    }
                    state.maps_table_states = table_states;
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

/// Render raw bytes as uppercase, space-separated hex (e.g. "DE AD BE EF").
fn hex_bytes(v: &[u8]) -> String {
    v.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Clamp bounds for column resize widths.
const COL_WIDTH_MIN: f32 = 24.0;
const COL_WIDTH_MAX: f32 = 600.0;

/// Return the (immutable) indices slice for a given map table kind.
fn maps_table_indices<'a>(
    cache: &'a crate::editors::save_file_viewer::state::MapsDisplayCaches,
    kind: crate::editors::save_file_viewer::state::MapsTableKind,
) -> &'a [usize] {
    use crate::editors::save_file_viewer::state::MapsTableKind;
    match kind {
        MapsTableKind::Monsters => &cache.monsters_indices,
        MapsTableKind::Npcs => &cache.npcs_indices,
        MapsTableKind::ExtraObjects => &cache.extra_objects_indices,
        MapsTableKind::Weapon => &cache.draw_items_weapon_indices,
        MapsTableKind::Heal => &cache.draw_items_heal_indices,
        MapsTableKind::Edit => &cache.draw_items_edit_indices,
        MapsTableKind::Misc => &cache.draw_items_misc_indices,
        MapsTableKind::Event => &cache.draw_items_event_indices,
    }
}

/// Return the (immutable rows, mutable indices) pair for a given map table
/// kind. The two borrows are disjoint fields of `MapsDisplayCaches`.
fn maps_table_data<'a>(
    cache: &'a mut crate::editors::save_file_viewer::state::MapsDisplayCaches,
    kind: crate::editors::save_file_viewer::state::MapsTableKind,
) -> (&'a [Vec<String>], &'a mut Vec<usize>) {
    use crate::editors::save_file_viewer::state::MapsTableKind;
    match kind {
        MapsTableKind::Monsters => (&cache.monsters, &mut cache.monsters_indices),
        MapsTableKind::Npcs => (&cache.npcs, &mut cache.npcs_indices),
        MapsTableKind::ExtraObjects => (&cache.extra_objects, &mut cache.extra_objects_indices),
        MapsTableKind::Weapon => (&cache.draw_items_weapon, &mut cache.draw_items_weapon_indices),
        MapsTableKind::Heal => (&cache.draw_items_heal, &mut cache.draw_items_heal_indices),
        MapsTableKind::Edit => (&cache.draw_items_edit, &mut cache.draw_items_edit_indices),
        MapsTableKind::Misc => (&cache.draw_items_misc, &mut cache.draw_items_misc_indices),
        MapsTableKind::Event => (&cache.draw_items_event, &mut cache.draw_items_event_indices),
    }
}

/// Return the (immutable rows, mutable indices) pair for a given inventory
/// category. The two borrows are disjoint fields of the two HashMaps.
fn inventory_table_data<'a>(
    cache: &'a mut std::collections::HashMap<
        crate::editors::save_file_viewer::state::InventoryCategory,
        Vec<Vec<String>>,
    >,
    indices: &'a mut std::collections::HashMap<
        crate::editors::save_file_viewer::state::InventoryCategory,
        Vec<usize>,
    >,
    cat: crate::editors::save_file_viewer::state::InventoryCategory,
) -> (&'a [Vec<String>], &'a mut Vec<usize>) {
    let rows = cache.get(&cat).map(|v| &v[..]).unwrap_or(&[]);
    let idx = indices.get_mut(&cat).expect("inventory indices missing");
    (rows, idx)
}

/// Numeric-aware cell comparison for sorting. Falls back to lexicographic
/// string comparison when either value is not a parseable float.
fn compare_cells(
    rows: &[Vec<String>],
    a: usize,
    b: usize,
    col: usize,
    ascending: bool,
) -> std::cmp::Ordering {
    let av = rows.get(a).and_then(|r| r.get(col));
    let bv = rows.get(b).and_then(|r| r.get(col));
    let ord = match (av, bv) {
        (Some(a), Some(b)) => match (a.parse::<f64>(), b.parse::<f64>()) {
            (Ok(an), Ok(bn)) => an.partial_cmp(&bn).unwrap_or(std::cmp::Ordering::Equal),
            _ => a.cmp(b),
        },
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => std::cmp::Ordering::Equal,
    };
    if ascending {
        ord
    } else {
        ord.reverse()
    }
}
