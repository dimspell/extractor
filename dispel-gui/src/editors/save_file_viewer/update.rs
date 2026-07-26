use iced::widget::image::Handle;
use iced::Task;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::app::App;
use crate::components::map_render::EntitySpriteHandle;
use crate::editors::save_file_viewer::map_preview::state::EntityKind;
use crate::editors::save_file_viewer::message::SaveFileViewerMessage;
use crate::message::{Message, MessageExt};
use dispel_core::map::sprite_loader::{load_last_frame_of_sequence, load_sprite_frames};
use dispel_core::sprite;
use dispel_core::{Extra, Extractor, MonsterIni, NpcIni};

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
            use crate::editors::save_file_viewer::state::MapsTableKind;
            // Clear preview if switching to a different map while preview is open
            if state.selected_map != Some(index) {
                state.show_preview = false;
                state.map_preview = None;
            }
            state.selected_map = Some(index);
            state.selected_entity_kind = MapsTableKind::Monsters;
            Task::none()
        }
        SaveFileViewerMessage::SelectEntityKind(kind) => {
            state.selected_entity_kind = kind;
            state.maps_resizing = None;
            Task::none()
        }
        SaveFileViewerMessage::TogglePreview => {
            if state.show_preview {
                // Switching back to tables: drop preview state
                state.show_preview = false;
                state.map_preview = None;
                Task::none()
            } else {
                state.show_preview = true;
                let map_idx = match state.selected_map {
                    Some(i) => i,
                    None => return Task::none(),
                };
                let map = match state.save_file.as_ref() {
                    Some(sf) => sf.maps.get(map_idx),
                    None => return Task::none(),
                };
                let map = match map {
                    Some(m) => m,
                    None => return Task::none(),
                };
                let map_id = map.map_id;

                // Try to get game path from workspace
                let game_path = app.state.workspace.game_path.clone();
                let Some(ref gp) = game_path else {
                    // No game path set — can't load maps; leave preview in loading state
                    return Task::none();
                };

                // Kick off async map file loading
                let gp = gp.clone();
                let task = iced::Task::perform(
                    async move {
                        let stem =
                            crate::editors::map_editor::resolve_map_filename(map_id as i32, &gp);
                        let stem = match stem {
                            Some(s) => s,
                            None => return Err(format!("No map filename for map_id {}", map_id)),
                        };
                        let map_path = gp.join("Map").join(&stem).with_extension("map");
                        let file = std::fs::File::open(&map_path)
                            .map_err(|e| format!("Failed to open {:?}: {}", map_path, e))?;
                        let mut reader = std::io::BufReader::new(file);
                        let map_data = dispel_core::map::read_map_data(&mut reader)
                            .map_err(|e| format!("Failed to parse map: {}", e))?;
                        let diagonal =
                            map_data.model.tiled_map_width + map_data.model.tiled_map_height;

                        Ok(
                            crate::editors::save_file_viewer::message::MapPreviewLoaded {
                                map_data: std::sync::Arc::new(map_data),
                                diagonal,
                                map_stem: stem,
                            },
                        )
                    },
                    move |result| {
                        Message::save_file_viewer(
                            crate::editors::save_file_viewer::message::SaveFileViewerMessage::MapPreviewLoaded(map_idx, result),
                        )
                    },
                );
                task
            }
        }
        SaveFileViewerMessage::MapPreviewLoaded(map_idx, result) => {
            if state.selected_map != Some(map_idx) {
                // Map was switched while loading — discard
                return Task::none();
            }
            let loaded = match result {
                Ok(l) => l,
                Err(e) => {
                    state.show_preview = false;
                    state.map_preview = None;
                    return Task::done(Message::System(crate::message::SystemMessage::ShowError(
                        format!("Failed to load map preview: {}", e),
                    )));
                }
            };

            let game_path = app.state.workspace.game_path.clone();

            use crate::components::map_render::MapViewState;
            use crate::editors::map_editor::message::MapDataHandle;
            use crate::editors::save_file_viewer::map_preview::state::{
                MapPreviewLoading, MapPreviewState, PreviewEntity,
            };
            let mut preview_state = MapPreviewState {
                map_data: Some(MapDataHandle(loaded.map_data.clone())),
                diagonal: loaded.diagonal,
                game_path: game_path.clone(),
                view: MapViewState::default(),
                loading: MapPreviewLoading::Loaded,
                entity_markers: Vec::new(),
                gtl_handles: std::collections::HashMap::new(),
                btl_handles: std::collections::HashMap::new(),
                tiles_ready: false,
                map_stem: Some(loaded.map_stem.clone()),
                entity_sprites: Vec::new(),
                sprites_ready: false,
                internal_sprites: Vec::new(),
                selected_marker: None,
            };

            // Centre the map at 100% zoom, mirroring the map editor's load
            // behaviour. Uses the last known canvas size (defaults to 1200×800
            // until the cursor moves over the canvas).
            if let Some(map_handle) = &preview_state.map_data {
                let model = &map_handle.0.model;
                let diagonal = model.tiled_map_width + model.tiled_map_height;
                let (cx, cy) = dispel_core::map::types::convert_map_coords_to_image_coords(
                    model.tiled_map_width / 2,
                    model.tiled_map_height / 2,
                    diagonal,
                );
                let vp_w = preview_state.view.last_canvas_w;
                let vp_h = preview_state.view.last_canvas_h;
                preview_state.view.zoom = 1.0;
                preview_state.view.pan_x = vp_w / 2.0 - cx as f32;
                preview_state.view.pan_y = vp_h / 2.0 - cy as f32;
                preview_state.view.tile_layer_cache.clear();
            }

            state.map_preview = Some(preview_state);

            // Build entity markers from save file data (synchronous)
            if let Some(sf) = state.save_file.as_ref() {
                if let Some(map_data) = &sf.maps.get(map_idx) {
                    let mut entities = Vec::new();
                    /// Safe cast: u32 → i32, clamps to i32 range and warns on overflow.
                    fn to_tile(v: u32) -> i32 {
                        if v > i32::MAX as u32 {
                            eprintln!("WARN: tile coordinate {} exceeds i32 range", v);
                            0
                        } else {
                            v as i32
                        }
                    }

                    // Monsters
                    for m in &map_data.monsters {
                        let x = to_tile(m.current_position_x as u32);
                        let y = to_tile(m.current_position_y as u32);
                        if x != 0 || y != 0 {
                            entities.push(PreviewEntity {
                                kind: EntityKind::Monster,
                                label: m.name.clone(),
                                tile_x: x,
                                tile_y: y,
                                confirmed: false,
                                db_id: Some(m.monster_db_id as i32),
                                is_dead: m.hp_current == 0,
                                look_direction: 0,
                            });
                        }
                    }
                    // NPCs — first active waypoint (HIGH confidence)
                    // Mirrors npc_pos() in map_editor/canvas/hit_test.rs.
                    // The save file has no "current position" field, so if an NPC
                    // is mid-patrol the best we can do is its first filled waypoint.
                    for n in &map_data.npcs {
                        let waypoints = [
                            (
                                n.npc_ref_waypoint1filled,
                                n.npc_ref_waypoint1x,
                                n.npc_ref_waypoint1y,
                            ),
                            (
                                n.npc_ref_waypoint2filled,
                                n.npc_ref_waypoint2x,
                                n.npc_ref_waypoint2y,
                            ),
                            (
                                n.npc_ref_waypoint3filled,
                                n.npc_ref_waypoint3x,
                                n.npc_ref_waypoint3y,
                            ),
                            (
                                n.npc_ref_waypoint4filled,
                                n.npc_ref_waypoint4x,
                                n.npc_ref_waypoint4y,
                            ),
                        ];
                        let (nx, ny) = waypoints
                            .iter()
                            .find(|(filled, _, _)| *filled != 0)
                            .map(|&(_, x, y)| (to_tile(x), to_tile(y)))
                            .unwrap_or((
                                to_tile(n.npc_ref_waypoint1x),
                                to_tile(n.npc_ref_waypoint1y),
                            ));
                        if nx != 0 || ny != 0 {
                            entities.push(PreviewEntity {
                                kind: EntityKind::Npc,
                                label: n.name.clone(),
                                tile_x: nx,
                                tile_y: ny,
                                confirmed: true,
                                db_id: Some(n.npc_ini_id as i32),
                                is_dead: false,
                                look_direction: n.npc_ref_look_direction as u8,
                            });
                        }
                    }
                    // Extra objects — use unknown_7/8 which map structurally to
                    // ExtraRef.x_pos/y_pos (both appear right after name + type byte
                    // in their respective struct layouts).  Keep confirmed:false
                    // pending empirical verification against real save files.
                    for e in &map_data.extra_objects {
                        let x = to_tile(e.unknown_7);
                        let y = to_tile(e.unknown_8);
                        if x != 0 || y != 0 {
                            entities.push(PreviewEntity {
                                kind: EntityKind::Extra,
                                label: e.name.clone(),
                                tile_x: x,
                                tile_y: y,
                                confirmed: false,
                                db_id: Some(e.unknown_5 as i32),
                                is_dead: false,
                                look_direction: 0,
                            });
                        }
                    }
                    // Draw items — map_coordinate_x/y per type (HIGH confidence)
                    for d in &map_data.draw_items_weapon {
                        let x = to_tile(d.map_coordinate_x);
                        let y = to_tile(d.map_coordinate_y);
                        if x != 0 || y != 0 {
                            entities.push(PreviewEntity {
                                kind: EntityKind::DrawItem,
                                label: d.name.clone(),
                                tile_x: x,
                                tile_y: y,
                                confirmed: true,
                                db_id: None,
                                is_dead: false,
                                look_direction: 0,
                            });
                        }
                    }
                    for d in &map_data.draw_items_heal {
                        let x = to_tile(d.map_coordinate_x);
                        let y = to_tile(d.map_coordinate_y);
                        if x != 0 || y != 0 {
                            entities.push(PreviewEntity {
                                kind: EntityKind::DrawItem,
                                label: d.name.clone(),
                                tile_x: x,
                                tile_y: y,
                                confirmed: true,
                                db_id: None,
                                is_dead: false,
                                look_direction: 0,
                            });
                        }
                    }
                    for d in &map_data.draw_items_edit {
                        let x = to_tile(d.map_coordinate_x);
                        let y = to_tile(d.map_coordinate_y);
                        if x != 0 || y != 0 {
                            entities.push(PreviewEntity {
                                kind: EntityKind::DrawItem,
                                label: d.name.clone(),
                                tile_x: x,
                                tile_y: y,
                                confirmed: true,
                                db_id: None,
                                is_dead: false,
                                look_direction: 0,
                            });
                        }
                    }
                    for d in &map_data.draw_items_misc {
                        let x = to_tile(d.map_coordinate_x);
                        let y = to_tile(d.map_coordinate_y);
                        if x != 0 || y != 0 {
                            entities.push(PreviewEntity {
                                kind: EntityKind::DrawItem,
                                label: d.name.clone(),
                                tile_x: x,
                                tile_y: y,
                                confirmed: true,
                                db_id: None,
                                is_dead: false,
                                look_direction: 0,
                            });
                        }
                    }
                    for d in &map_data.draw_items_event {
                        let x = to_tile(d.map_coordinate_x);
                        let y = to_tile(d.map_coordinate_y);
                        if x != 0 || y != 0 {
                            entities.push(PreviewEntity {
                                kind: EntityKind::DrawItem,
                                label: d.name.clone(),
                                tile_x: x,
                                tile_y: y,
                                confirmed: true,
                                db_id: None,
                                is_dead: false,
                                look_direction: 0,
                            });
                        }
                    }
                    if let Some(preview) = state.map_preview.as_mut() {
                        preview.entity_markers = entities;
                    }
                }
            }

            // Derive gtl/btl paths and kick off async tileset decoding
            let map_path = match game_path {
                Some(ref gp) => gp.join("Map").join(&loaded.map_stem).with_extension("map"),
                None => return Task::none(),
            };
            let gtl_path = map_path.with_extension("gtl");
            let btl_path = map_path.with_extension("btl");

            use std::collections::HashMap;
            use std::collections::HashSet;

            // Include building tile IDs (from tiled_infos) in btl decode set.
            // Without this, buildings in the interlaced pass have no textures.
            let gtl_ids: HashSet<i32> = loaded.map_data.gtl_tiles.values().copied().collect();
            let btl_ids: HashSet<i32> = loaded
                .map_data
                .btl_tiles
                .values()
                .copied()
                .chain(
                    loaded
                        .map_data
                        .tiled_infos
                        .iter()
                        .flat_map(|t| t.ids.iter().map(|&id| id.unsigned_abs() as i32)),
                )
                .filter(|&id| id > 0)
                .collect();

            let tile_task = iced::Task::perform(
                async move {
                    use crate::components::map_render::decode_tileset_file;
                    use iced::widget::image::Handle;

                    let gtl_raw = decode_tileset_file(&gtl_path, &gtl_ids).unwrap_or_default();
                    let btl_raw = decode_tileset_file(&btl_path, &btl_ids).unwrap_or_default();

                    let gtl: HashMap<i32, Handle> = gtl_raw
                        .into_iter()
                        .map(|(id, px)| (id, Handle::from_rgba(62, 32, px)))
                        .collect();
                    let btl: HashMap<i32, Handle> = btl_raw
                        .into_iter()
                        .map(|(id, px)| (id, Handle::from_rgba(62, 32, px)))
                        .collect();

                    // Decode internal sprites from the .map file (thrones, decor, etc.)
                    let internal_sprites =
                        decode_internal_preview_sprites(&map_path, &loaded.map_data);

                    crate::editors::save_file_viewer::message::MapPreviewTiles {
                        gtl,
                        btl,
                        internal_sprites,
                    }
                },
                move |tiles| {
                    Message::save_file_viewer(
                        crate::editors::save_file_viewer::message::SaveFileViewerMessage::MapPreviewTilesReady(map_idx, Ok(tiles)),
                    )
                },
            );

            // Start async entity sprite loading (parallel to tile decoding)
            let gp_for_sprites = match &game_path {
                Some(gp) => gp.clone(),
                None => return tile_task,
            };
            let entity_markers = state
                .map_preview
                .as_ref()
                .map(|p| p.entity_markers.clone())
                .unwrap_or_default();
            let sprite_task = iced::Task::perform(
                async move { load_preview_sprites(gp_for_sprites, entity_markers).await },
                move |result| {
                    Message::save_file_viewer(
                        crate::editors::save_file_viewer::message::SaveFileViewerMessage::PreviewSpritesReady(map_idx, result),
                    )
                },
            );

            Task::batch([tile_task, sprite_task])
        }
        SaveFileViewerMessage::MapPreviewTilesReady(map_idx, result) => {
            if state.selected_map != Some(map_idx) {
                return Task::none();
            }
            let tiles = match result {
                Ok(t) => t,
                Err(_) => return Task::none(),
            };
            if let Some(preview) = state.map_preview.as_mut() {
                preview.gtl_handles = tiles.gtl;
                preview.btl_handles = tiles.btl;
                preview.internal_sprites = tiles.internal_sprites;
                preview.tiles_ready = true;
                // Force re-cache — tile cache was populated while tiles_ready was false
                preview.view.tile_layer_cache.clear();
            }
            Task::none()
        }
        SaveFileViewerMessage::PreviewSpritesReady(map_idx, result) => {
            if state.selected_map != Some(map_idx) {
                return Task::none();
            }
            let loaded = match result {
                Ok(l) => l,
                Err(_) => return Task::none(),
            };
            if let Some(preview) = state.map_preview.as_mut() {
                preview.entity_sprites = loaded.sprites;
                preview.sprites_ready = true;
                // Force tile layer to re-cache with sprites
                preview.view.tile_layer_cache.clear();
            }
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
                let new_width =
                    (drag.anchor_width + (x - anchor_x)).clamp(COL_WIDTH_MIN, COL_WIDTH_MAX);
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
        SaveFileViewerMessage::MapsTableScroll {
            map, kind, x, y, ..
        } => {
            if let Some(ts) = state
                .maps_table_states
                .get_mut(map)
                .and_then(|m| m.get_mut(&kind))
            {
                ts.table_state.scroll_offset = iced::Vector::new(x, y);
            }
            Task::none()
        }
        SaveFileViewerMessage::InventoryTableSelect { cat, visible_idx } => {
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
                let new_width =
                    (drag.anchor_width + (x - anchor_x)).clamp(COL_WIDTH_MIN, COL_WIDTH_MAX);
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
        SaveFileViewerMessage::InventoryTableScroll { cat, x, y, .. } => {
            if let Some(ts) = state.inventory_table_states.get_mut(&cat) {
                ts.table_state.scroll_offset = iced::Vector::new(x, y);
            }
            Task::none()
        }
        SaveFileViewerMessage::EventsTableSelect { visible_idx } => {
            let orig = state.events_filtered_indices.get(visible_idx).copied();
            state.events_table_state.selected_orig = orig;
            Task::none()
        }
        SaveFileViewerMessage::EventsTableSort { col } => {
            let ts = &mut state.events_table_state;
            if ts.sort_column == Some(col) {
                ts.sort_ascending = !ts.sort_ascending;
            } else {
                ts.sort_column = Some(col);
                ts.sort_ascending = true;
            }
            let ascending = ts.sort_ascending;
            let (rows, indices) = events_table_data(
                &mut state.events_display_cache,
                &mut state.events_filtered_indices,
            );
            indices.sort_by(|&a, &b| compare_cells(rows, a, b, col, ascending));
            Task::none()
        }
        SaveFileViewerMessage::EventsTableStartResize { col } => {
            let anchor_width = state
                .events_table_state
                .column_widths
                .get(col)
                .copied()
                .unwrap_or(80.0);
            state.events_resizing =
                Some(crate::editors::save_file_viewer::state::EventsResizeDrag {
                    col,
                    anchor_width,
                    anchor_cursor_x: None,
                });
            Task::none()
        }
        SaveFileViewerMessage::EventsTableResetColumnWidth { col } => {
            let default_width = crate::editors::save_file_viewer::state::events_default_columns()
                .into_iter()
                .nth(col)
                .map(|c| c.width_px)
                .unwrap_or(80.0);
            if let Some(w) = state.events_table_state.column_widths.get_mut(col) {
                *w = default_width;
            }
            Task::none()
        }
        SaveFileViewerMessage::EventsTableResizeCursor(x) => {
            if let Some(drag) = state.events_resizing.as_mut() {
                let anchor_x = match drag.anchor_cursor_x {
                    Some(ax) => ax,
                    None => {
                        drag.anchor_cursor_x = Some(x);
                        return Task::none();
                    }
                };
                let new_width =
                    (drag.anchor_width + (x - anchor_x)).clamp(COL_WIDTH_MIN, COL_WIDTH_MAX);
                if let Some(w) = state.events_table_state.column_widths.get_mut(drag.col) {
                    *w = new_width;
                }
            }
            Task::none()
        }
        SaveFileViewerMessage::EventsTableEndResize => {
            state.events_resizing = None;
            Task::none()
        }
        SaveFileViewerMessage::EventsTableScroll { x, y, .. } => {
            state.events_table_state.table_state.scroll_offset = iced::Vector::new(x, y);
            Task::none()
        }
        SaveFileViewerMessage::JournalTableSelect {
            section,
            visible_idx,
        } => {
            if let Some(indices) = state.journal_filtered_indices.get(&section) {
                let orig = indices.get(visible_idx).copied();
                if let Some(ts) = state.journal_table_states.get_mut(&section) {
                    ts.selected_orig = orig;
                }
            }
            Task::none()
        }
        SaveFileViewerMessage::JournalTableSort { section, col } => {
            let ts = match state.journal_table_states.get_mut(&section) {
                Some(ts) => ts,
                None => return Task::none(),
            };
            if ts.sort_column == Some(col) {
                ts.sort_ascending = !ts.sort_ascending;
            } else {
                ts.sort_column = Some(col);
                ts.sort_ascending = true;
            }
            let ascending = ts.sort_ascending;
            let (rows, indices) = journal_table_data(
                &mut state.journal_display_caches,
                &mut state.journal_filtered_indices,
                section,
            );
            indices.sort_by(|&a, &b| compare_cells(rows, a, b, col, ascending));
            Task::none()
        }
        SaveFileViewerMessage::JournalTableStartResize { section, col } => {
            let anchor_width = state
                .journal_table_states
                .get(&section)
                .and_then(|ts| ts.column_widths.get(col).copied())
                .unwrap_or(80.0);
            state.journal_resizing =
                Some(crate::editors::save_file_viewer::state::JournalResizeDrag {
                    section,
                    col,
                    anchor_width,
                    anchor_cursor_x: None,
                });
            Task::none()
        }
        SaveFileViewerMessage::JournalTableResetColumnWidth { section, col } => {
            if let Some(ts) = state.journal_table_states.get_mut(&section) {
                let default_width = section
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
        SaveFileViewerMessage::JournalTableResizeCursor(x) => {
            if let Some(drag) = state.journal_resizing.as_mut() {
                let anchor_x = match drag.anchor_cursor_x {
                    Some(ax) => ax,
                    None => {
                        drag.anchor_cursor_x = Some(x);
                        return Task::none();
                    }
                };
                let new_width =
                    (drag.anchor_width + (x - anchor_x)).clamp(COL_WIDTH_MIN, COL_WIDTH_MAX);
                if let Some(ts) = state.journal_table_states.get_mut(&drag.section) {
                    if let Some(w) = ts.column_widths.get_mut(drag.col) {
                        *w = new_width;
                    }
                }
            }
            Task::none()
        }
        SaveFileViewerMessage::JournalTableEndResize => {
            state.journal_resizing = None;
            Task::none()
        }
        SaveFileViewerMessage::JournalTableScroll { section, x, y, .. } => {
            if let Some(ts) = state.journal_table_states.get_mut(&section) {
                ts.table_state.scroll_offset = iced::Vector::new(x, y);
            }
            Task::none()
        }
        SaveFileViewerMessage::ExportCsv(key) => {
            let Some((headers, rows)) = resolve_csv_export_data(state, key) else {
                return Task::none();
            };
            let default_name = csv_default_filename(key);
            iced::Task::perform(
                async move {
                    let mut wtr = csv::Writer::from_writer(Vec::new());
                    wtr.write_record(&headers).map_err(|e| e.to_string())?;
                    for row in &rows {
                        wtr.write_record(row).map_err(|e| e.to_string())?;
                    }
                    let bytes = wtr.into_inner().map_err(|e| e.to_string())?;

                    let handle = rfd::AsyncFileDialog::new()
                        .set_file_name(&default_name)
                        .add_filter("CSV", &["csv"])
                        .save_file()
                        .await;
                    match handle {
                        Some(h) => {
                            let path = h.path().to_path_buf();
                            tokio::fs::write(&path, &bytes)
                                .await
                                .map(|_| path)
                                .map_err(|e| e.to_string())
                        }
                        None => Err("cancelled".to_string()),
                    }
                },
                move |result| {
                    Message::save_file_viewer(SaveFileViewerMessage::CsvExported(
                        result,
                    ))
                },
            )
        }
        SaveFileViewerMessage::Load(_) => {
            // Load is handled by app.rs::open_file_in_workspace via Task::perform
            state.loading = true;
            Task::none()
        }
        SaveFileViewerMessage::TableFilter { key, action } => {
            handle_table_filter(state, key, action)
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
                                    item.misc_item_id.to_string(),
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
                    use crate::editors::save_file_viewer::state::TableInteractionState;
                    let mut inv_states: std::collections::HashMap<
                        InventoryCategory,
                        TableInteractionState,
                    > = std::collections::HashMap::new();
                    for cat in state.inventory_display_caches.keys() {
                        let widths: Vec<f32> =
                            cat.default_columns().iter().map(|c| c.width_px).collect();
                        inv_states.insert(
                            *cat,
                            TableInteractionState {
                                column_widths: widths,
                                ..Default::default()
                            },
                        );
                    }
                    state.inventory_table_states = inv_states;

                    // Build events table interaction state (single table).
                    {
                        use crate::editors::save_file_viewer::state::events_default_columns;
                        let widths: Vec<f32> = events_default_columns()
                            .iter()
                            .map(|c| c.width_px)
                            .collect();
                        state.events_table_state = TableInteractionState {
                            column_widths: widths,
                            ..Default::default()
                        };
                    }

                    // Build journal table interaction state, keyed by section.
                    {
                        use crate::editors::save_file_viewer::state::JournalSection;
                        let mut journal_states: std::collections::HashMap<
                            JournalSection,
                            TableInteractionState,
                        > = std::collections::HashMap::new();
                        for section in JournalSection::all() {
                            let widths: Vec<f32> = section
                                .default_columns()
                                .iter()
                                .map(|c| c.width_px)
                                .collect();
                            journal_states.insert(
                                *section,
                                TableInteractionState {
                                    column_widths: widths,
                                    ..Default::default()
                                },
                            );
                        }
                        state.journal_table_states = journal_states;
                    }
                    // Build maps display caches
                    let maps_caches: Vec<
                        crate::editors::save_file_viewer::state::MapsDisplayCaches,
                    > = loaded
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
                                monsters: map
                                    .monsters
                                    .iter()
                                    .map(|m| {
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
                                            m.current_position_x.to_string(),
                                            m.current_position_y.to_string(),
                                            m.spawn_position_x.to_string(),
                                            m.spawn_position_y.to_string(),
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
                                            m.loot_item1.raw().to_string(),
                                            m.loot_item2.raw().to_string(),
                                            m.loot_item3.raw().to_string(),
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
                                    })
                                    .collect(),
                                monsters_indices: (0..n_monsters).collect(),
                                npcs: map
                                    .npcs
                                    .iter()
                                    .map(|n| {
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
                                    })
                                    .collect(),
                                npcs_indices: (0..n_npcs).collect(),
                                extra_objects: map
                                    .extra_objects
                                    .iter()
                                    .map(|e| {
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
                                    })
                                    .collect(),
                                extra_objects_indices: (0..n_extras).collect(),
                                draw_items_weapon: map
                                    .draw_items_weapon
                                    .iter()
                                    .map(|d| {
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
                                    })
                                    .collect(),
                                draw_items_weapon_indices: (0..n_dw).collect(),
                                draw_items_heal: map
                                    .draw_items_heal
                                    .iter()
                                    .map(|d| {
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
                                    })
                                    .collect(),
                                draw_items_heal_indices: (0..n_dh).collect(),
                                draw_items_edit: map
                                    .draw_items_edit
                                    .iter()
                                    .map(|d| {
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
                                    })
                                    .collect(),
                                draw_items_edit_indices: (0..n_de).collect(),
                                draw_items_misc: map
                                    .draw_items_misc
                                    .iter()
                                    .map(|d| {
                                        vec![
                                            d.name.clone(),
                                            d.description.clone(),
                                            d.base_price.to_string(),
                                            hex_bytes(&d.unknown_1),
                                            d.misc_item_id.to_string(),
                                            d.map_coordinate_x.to_string(),
                                            d.map_coordinate_y.to_string(),
                                            d.unknown_7.to_string(),
                                        ]
                                    })
                                    .collect(),
                                draw_items_misc_indices: (0..n_dm).collect(),
                                draw_items_event: map
                                    .draw_items_event
                                    .iter()
                                    .map(|d| {
                                        vec![
                                            d.name.clone(),
                                            d.description.clone(),
                                            d.base_price.to_string(),
                                            d.event_item_id.to_string(),
                                            d.map_coordinate_x.to_string(),
                                            d.map_coordinate_y.to_string(),
                                            d.unknown_1.to_string(),
                                        ]
                                    })
                                    .collect(),
                                draw_items_event_indices: (0..n_dev).collect(),
                            }
                        })
                        .collect();
                    state.maps_display_caches = maps_caches;
                    // Build per-map, per-table interaction state. Column widths
                    // are initialised from each table kind's default layout.
                    use crate::editors::save_file_viewer::state::{MapTableState, MapsTableKind};
                    let mut table_states: Vec<
                        std::collections::HashMap<MapsTableKind, MapTableState>,
                    > = Vec::with_capacity(state.maps_display_caches.len());
                    for _ in &state.maps_display_caches {
                        let mut per_map = std::collections::HashMap::new();
                        for kind in MapsTableKind::all() {
                            let widths: Vec<f32> =
                                kind.default_columns().iter().map(|c| c.width_px).collect();
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
                                let hex_rest: Vec<String> =
                                    entry.rest.iter().map(|b| format!("{:02X}", b)).collect();
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
        SaveFileViewerMessage::CsvExported(result) => {
            match result {
                Ok(path) => {
                    state.status_msg = Some(format!("Exported CSV to {}", path.display()));
                }
                Err(e) if e == "cancelled" => {}
                Err(e) => {
                    state.status_msg = Some(format!("CSV export failed: {}", e));
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
fn maps_table_indices(
    cache: &crate::editors::save_file_viewer::state::MapsDisplayCaches,
    kind: crate::editors::save_file_viewer::state::MapsTableKind,
) -> &[usize] {
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
fn maps_table_data(
    cache: &mut crate::editors::save_file_viewer::state::MapsDisplayCaches,
    kind: crate::editors::save_file_viewer::state::MapsTableKind,
) -> (&[Vec<String>], &mut Vec<usize>) {
    use crate::editors::save_file_viewer::state::MapsTableKind;
    match kind {
        MapsTableKind::Monsters => (&cache.monsters, &mut cache.monsters_indices),
        MapsTableKind::Npcs => (&cache.npcs, &mut cache.npcs_indices),
        MapsTableKind::ExtraObjects => (&cache.extra_objects, &mut cache.extra_objects_indices),
        MapsTableKind::Weapon => (
            &cache.draw_items_weapon,
            &mut cache.draw_items_weapon_indices,
        ),
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

/// Return the (immutable rows, mutable indices) pair for the events table.
fn events_table_data<'a>(
    cache: &'a mut [Vec<String>],
    indices: &'a mut Vec<usize>,
) -> (&'a [Vec<String>], &'a mut Vec<usize>) {
    (&cache[..], indices)
}

/// Return the (immutable rows, mutable indices) pair for a journal table.
fn journal_table_data<'a>(
    cache: &'a mut std::collections::HashMap<
        crate::editors::save_file_viewer::state::JournalSection,
        Vec<Vec<String>>,
    >,
    indices: &'a mut std::collections::HashMap<
        crate::editors::save_file_viewer::state::JournalSection,
        Vec<usize>,
    >,
    section: crate::editors::save_file_viewer::state::JournalSection,
) -> (&'a [Vec<String>], &'a mut Vec<usize>) {
    let rows = cache.get(&section).map(|v| &v[..]).unwrap_or(&[]);
    let idx = indices.get_mut(&section).expect("journal indices missing");
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

/// Row height used by every save-file-viewer table (kept in sync with the
/// `TableWidget::new` `row_height` argument in the view files). Used to scroll
/// a highlighted row into view during Highlight-mode navigation.
const FILTER_ROW_HEIGHT: f32 = 22.0;

/// Dispatches a unified column-filter action to the table identified by `key`.
fn handle_table_filter(
    state: &mut crate::editors::save_file_viewer::state::SaveFileViewerState,
    key: crate::editors::save_file_viewer::message::TableKey,
    action: crate::editors::save_file_viewer::message::TableFilterAction,
) -> Task<Message> {
    use crate::editors::save_file_viewer::message::TableFilterAction;

    match action {
        TableFilterAction::NextHighlight => return navigate_highlight(state, key, true),
        TableFilterAction::PrevHighlight => return navigate_highlight(state, key, false),
        _ => {}
    }

    let Some((filter, rows, indices)) = table_filter_access(state, key) else {
        return Task::none();
    };

    match action {
        TableFilterAction::OpenColumnFilter(col) => {
            filter.active_column_filter = Some(col);
            filter.column_filter_search.clear();
            filter.column_filter_options = unique_values(rows, col);
        }
        TableFilterAction::ToggleColumnFilterValue(col, value) => {
            let set = filter.column_filters.entry(col).or_default();
            if !set.insert(value.clone()) {
                set.remove(&value);
            }
            if set.is_empty() {
                filter.column_filters.remove(&col);
            }
            apply_table_filter(rows, filter, indices);
        }
        TableFilterAction::SelectAllColumnFilter(col) => {
            let search = filter.column_filter_search.to_lowercase();
            let values: std::collections::HashSet<String> = filter
                .column_filter_options
                .iter()
                .filter(|o| o.value.to_lowercase().contains(&search))
                .map(|o| o.value.clone())
                .collect();
            filter.column_filters.insert(col, values);
            apply_table_filter(rows, filter, indices);
        }
        TableFilterAction::ClearAllColumnFilter(col) => {
            let search = filter.column_filter_search.to_lowercase();
            let remove: std::collections::HashSet<String> = filter
                .column_filter_options
                .iter()
                .filter(|o| o.value.to_lowercase().contains(&search))
                .map(|o| o.value.clone())
                .collect();
            let current = filter.column_filters.entry(col).or_default();
            *current = current.difference(&remove).cloned().collect();
            if current.is_empty() {
                filter.column_filters.remove(&col);
            }
            apply_table_filter(rows, filter, indices);
        }
        TableFilterAction::ColumnFilterSearch(s) => {
            filter.column_filter_search = s;
        }
        TableFilterAction::CloseColumnFilterModal => {
            filter.active_column_filter = None;
        }
        TableFilterAction::ClearColumnFilter(col) => {
            filter.column_filters.remove(&col);
            filter.active_column_filter = None;
            apply_table_filter(rows, filter, indices);
        }
        TableFilterAction::QuickFilter(col, value) => {
            let mut set = std::collections::HashSet::new();
            set.insert(value);
            filter.column_filters.insert(col, set);
            filter.active_column_filter = None;
            apply_table_filter(rows, filter, indices);
        }
        TableFilterAction::QueryChanged(s) => {
            filter.filter_query = s;
            apply_table_filter(rows, filter, indices);
        }
        TableFilterAction::SetMode(mode) => {
            filter.filter_mode = mode;
            apply_table_filter(rows, filter, indices);
        }
        TableFilterAction::ClearAllFilters => {
            filter.column_filters.clear();
            filter.filter_query.clear();
            filter.active_column_filter = None;
            apply_table_filter(rows, filter, indices);
        }
        TableFilterAction::NextHighlight | TableFilterAction::PrevHighlight => {}
    }
    Task::none()
}

/// Borrow the filter state alongside the (immutable) display rows and the
/// (mutable) filtered indices for the table identified by `key`. The filter
/// state and the caches live in disjoint fields of `SaveFileViewerState`, so
/// both can be mutably borrowed at once.
#[allow(clippy::type_complexity)]
fn table_filter_access(
    state: &mut crate::editors::save_file_viewer::state::SaveFileViewerState,
    key: crate::editors::save_file_viewer::message::TableKey,
) -> Option<(
    &mut crate::editors::save_file_viewer::state::TableFilterState,
    &[Vec<String>],
    &mut Vec<usize>,
)> {
    use crate::editors::save_file_viewer::message::TableKey;
    match key {
        TableKey::Map(map, kind) => {
            let ts = state.maps_table_states.get_mut(map)?.get_mut(&kind)?;
            let filter = &mut ts.filter;
            let (rows, indices) = maps_table_data(&mut state.maps_display_caches[map], kind);
            Some((filter, rows, indices))
        }
        TableKey::Inventory(cat) => {
            let ts = state.inventory_table_states.get_mut(&cat)?;
            let filter = &mut ts.filter;
            let (rows, indices) = inventory_table_data(
                &mut state.inventory_display_caches,
                &mut state.inventory_filtered_indices,
                cat,
            );
            Some((filter, rows, indices))
        }
        TableKey::Events => {
            let filter = &mut state.events_table_state.filter;
            let (rows, indices) = events_table_data(
                &mut state.events_display_cache,
                &mut state.events_filtered_indices,
            );
            Some((filter, rows, indices))
        }
        TableKey::Journal(section) => {
            let ts = state.journal_table_states.get_mut(&section)?;
            let filter = &mut ts.filter;
            let (rows, indices) = journal_table_data(
                &mut state.journal_display_caches,
                &mut state.journal_filtered_indices,
                section,
            );
            Some((filter, rows, indices))
        }
    }
}

/// Rebuild `filtered_indices` / `highlighted_indices` from the current
/// `filter_query`, `filter_mode`, and `column_filters`. Mirrors the
/// spreadsheet editor's `apply_filter`.
fn apply_table_filter(
    rows: &[Vec<String>],
    filter: &mut crate::editors::save_file_viewer::state::TableFilterState,
    indices: &mut Vec<usize>,
) {
    use crate::components::filter::GlobalFilterMode;

    filter.highlighted_indices.clear();

    let has_query = !filter.filter_query.is_empty();
    let has_col = !filter.column_filters.is_empty();

    let col_matches = |row: &[String]| -> bool {
        for (&col, selected) in &filter.column_filters {
            if let Some(value) = row.get(col) {
                if !selected.is_empty() && !selected.contains(value) {
                    return false;
                }
            }
        }
        true
    };

    if !has_query && !has_col {
        *indices = (0..rows.len()).collect();
        return;
    }

    let query = filter.filter_query.to_lowercase();
    let matches_query =
        |row: &[String]| -> bool { row.iter().any(|cell| cell.to_lowercase().contains(&query)) };

    match filter.filter_mode {
        GlobalFilterMode::FilterOut => {
            indices.clear();
            for (idx, row) in rows.iter().enumerate() {
                let col_ok = !has_col || col_matches(row);
                let q_ok = !has_query || matches_query(row);
                if col_ok && q_ok {
                    indices.push(idx);
                }
            }
        }
        GlobalFilterMode::Highlight => {
            // Column filters hard-filter; global query only highlights.
            indices.clear();
            for (idx, row) in rows.iter().enumerate() {
                if !has_col || col_matches(row) {
                    indices.push(idx);
                    if has_query && matches_query(row) {
                        filter.highlighted_indices.push(idx);
                    }
                }
            }
            if !filter.highlighted_indices.is_empty() {
                filter.current_highlight_pos = Some(0);
            }
        }
    }
}

/// Distinct values (with counts) for a column, sorted by value.
fn unique_values(
    rows: &[Vec<String>],
    col: usize,
) -> Vec<crate::components::filter::ColumnFilterOption> {
    use crate::components::filter::ColumnFilterOption;
    use std::collections::HashMap;

    let mut counts: HashMap<&str, usize> = HashMap::new();
    for row in rows {
        if let Some(v) = row.get(col) {
            *counts.entry(v.as_str()).or_insert(0) += 1;
        }
    }
    let mut opts: Vec<ColumnFilterOption> = counts
        .into_iter()
        .map(|(v, count)| ColumnFilterOption {
            value: v.to_string(),
            count,
        })
        .collect();
    opts.sort_by(|a, b| a.value.cmp(&b.value));
    opts
}

/// Return the visible (filtered) indices for the table identified by `key`,
/// used to translate an original index to a visible position for scrolling.
fn filtered_indices_for(
    state: &crate::editors::save_file_viewer::state::SaveFileViewerState,
    key: crate::editors::save_file_viewer::message::TableKey,
) -> Option<&[usize]> {
    use crate::editors::save_file_viewer::message::TableKey;
    match key {
        TableKey::Map(map, kind) => state
            .maps_display_caches
            .get(map)
            .map(|c| maps_table_indices(c, kind)),
        TableKey::Inventory(cat) => state.inventory_filtered_indices.get(&cat).map(|v| &v[..]),
        TableKey::Events => Some(&state.events_filtered_indices),
        TableKey::Journal(section) => state.journal_filtered_indices.get(&section).map(|v| &v[..]),
    }
}

/// Step the Highlight-mode highlight cursor and bring the focused row into
/// view, mirroring the spreadsheet editor's `Navigate{Next,Prev}Highlight`.
fn navigate_highlight(
    state: &mut crate::editors::save_file_viewer::state::SaveFileViewerState,
    key: crate::editors::save_file_viewer::message::TableKey,
    next: bool,
) -> Task<Message> {
    use crate::editors::save_file_viewer::message::TableKey;

    // Advance the cursor on the table's filter state.
    match key {
        TableKey::Map(map, kind) => {
            let Some(ts) = state
                .maps_table_states
                .get_mut(map)
                .and_then(|m| m.get_mut(&kind))
            else {
                return Task::none();
            };
            if next {
                ts.filter.navigate_next_highlight();
            } else {
                ts.filter.navigate_prev_highlight();
            }
        }
        TableKey::Inventory(cat) => {
            let Some(ts) = state.inventory_table_states.get_mut(&cat) else {
                return Task::none();
            };
            if next {
                ts.filter.navigate_next_highlight();
            } else {
                ts.filter.navigate_prev_highlight();
            }
        }
        TableKey::Events => {
            if next {
                state.events_table_state.filter.navigate_next_highlight();
            } else {
                state.events_table_state.filter.navigate_prev_highlight();
            }
        }
        TableKey::Journal(section) => {
            let Some(ts) = state.journal_table_states.get_mut(&section) else {
                return Task::none();
            };
            if next {
                ts.filter.navigate_next_highlight();
            } else {
                ts.filter.navigate_prev_highlight();
            }
        }
    }

    // Resolve the focused original index, then the focused table state so we
    // can update selection + scroll. We re-fetch the table state here because
    // the filter state above lives inside it.
    let orig = match key {
        TableKey::Map(map, kind) => state
            .maps_table_states
            .get(map)
            .and_then(|m| m.get(&kind))
            .and_then(|ts| ts.filter.current_highlight_orig_idx()),
        TableKey::Inventory(cat) => state
            .inventory_table_states
            .get(&cat)
            .and_then(|ts| ts.filter.current_highlight_orig_idx()),
        TableKey::Events => state.events_table_state.filter.current_highlight_orig_idx(),
        TableKey::Journal(section) => state
            .journal_table_states
            .get(&section)
            .and_then(|ts| ts.filter.current_highlight_orig_idx()),
    };

    let Some(orig) = orig else {
        return Task::none();
    };

    let visible =
        filtered_indices_for(state, key).and_then(|idxs| idxs.iter().position(|&i| i == orig));

    match (key, visible) {
        (TableKey::Map(map, kind), Some(fidx)) => {
            if let Some(ts) = state
                .maps_table_states
                .get_mut(map)
                .and_then(|m| m.get_mut(&kind))
            {
                ts.selected_orig = Some(orig);
                ts.table_state.scroll_offset.y = fidx as f32 * FILTER_ROW_HEIGHT;
            }
        }
        (TableKey::Inventory(cat), Some(fidx)) => {
            if let Some(ts) = state.inventory_table_states.get_mut(&cat) {
                ts.selected_orig = Some(orig);
                ts.table_state.scroll_offset.y = fidx as f32 * FILTER_ROW_HEIGHT;
            }
        }
        (TableKey::Events, Some(fidx)) => {
            state.events_table_state.selected_orig = Some(orig);
            state.events_table_state.table_state.scroll_offset.y = fidx as f32 * FILTER_ROW_HEIGHT;
        }
        (TableKey::Journal(section), Some(fidx)) => {
            if let Some(ts) = state.journal_table_states.get_mut(&section) {
                ts.selected_orig = Some(orig);
                ts.table_state.scroll_offset.y = fidx as f32 * FILTER_ROW_HEIGHT;
            }
        }
        _ => {}
    }
    Task::none()
}

/// Async-load entity sprites for the map preview.
///
/// Reads `Monster.ini` / `Npc.ini` / `Extra.ini` to map entity DB IDs → sprite
/// filenames, then decodes frame[0] of each unique `.spr` file.  Returns a
/// `Vec` parallel to `entity_markers` (None for entities without a resolvable
/// sprite).
async fn load_preview_sprites(
    game_path: PathBuf,
    entity_markers: Vec<crate::editors::save_file_viewer::map_preview::state::PreviewEntity>,
) -> Result<crate::editors::save_file_viewer::message::PreviewSpritesLoaded, String> {
    // 1. Load Monster.ini → HashMap<id, sprite_filename>
    let monster_id_to_sprite: HashMap<i32, String> =
        MonsterIni::read_file(&game_path.join("Monster.ini"))
            .map_err(|e| format!("Failed to load Monster.ini: {}", e))?
            .into_iter()
            .filter_map(|m| m.sprite_filename.map(|s| (m.id, s)))
            .collect();

    // 2. Load Npc.ini → HashMap<id, sprite_filename>
    let npc_id_to_sprite: HashMap<i32, String> = NpcIni::read_file(&game_path.join("Npc.ini"))
        .map_err(|e| format!("Failed to load Npc.ini: {}", e))?
        .into_iter()
        .filter_map(|n| n.sprite_filename.map(|s| (n.id, s)))
        .collect();

    // 3. Load Extra.ini → HashMap<id, sprite_filename>
    let extra_id_to_sprite: HashMap<i32, String> = load_extra_ini_sprites(&game_path)
        .map_err(|e| format!("Failed to load Extra.ini: {}", e))?;

    // 4. Resolve sprites for each entity (parallel to entity_markers)
    // Cache key includes `is_dead` and `look_direction` because the same sprite
    // path can be shared by entities in different states (alive vs dead) or
    // facing different directions — without this the first loaded variant would
    // be reused for all others, showing the wrong frame or flip.
    let mut sprite_cache: HashMap<(PathBuf, bool, u8), Option<EntitySpriteHandle>> =
        HashMap::new();
    let sprites: Vec<Option<EntitySpriteHandle>> = entity_markers
        .iter()
        .map(|entity| {
            let db_id = entity.db_id?;
            let (sub_dir, id_to_sprite) = match entity.kind {
                EntityKind::Monster => ("MonsterInGame", &monster_id_to_sprite),
                EntityKind::Npc => ("NpcInGame", &npc_id_to_sprite),
                EntityKind::Extra => ("ExtraInGame", &extra_id_to_sprite),
                EntityKind::DrawItem => return None,
            };
            // The save file stores the Monster.db ID (0-based archetype index),
            // but Monster.ini / .ref files are keyed by the visual ID which is
            // offset by one (e.g. db 24 → ini 25). Translate before lookup.
            let lookup_id = if matches!(entity.kind, EntityKind::Monster) {
                db_id + 1
            } else {
                db_id
            };
            let sprite_name = id_to_sprite.get(&lookup_id)?;
            let path = resolve_sprite_path(&game_path, sub_dir, sprite_name)?;
            // Dead monsters render the LAST frame of the LAST sequence (the
            // death animation's final "corpse" pose).  Alive entities use the
            // NPC looking-direction formula (mirrors map_editor/update/map.rs)
            // to select a sprite sequence + flip.
            sprite_cache
                .entry((path.clone(), entity.is_dead, entity.look_direction))
                .or_insert_with(|| {
                    let frame = if entity.is_dead {
                        let seq_count = sprite::read_sprite_file(&path)
                            .ok()
                            .map(|sf| sf.sequences.len())
                            .unwrap_or(0);
                        if seq_count == 0 {
                            return None;
                        }
                        load_last_frame_of_sequence(&path, seq_count - 1)?
                    } else {
                        // Compute (sequence, flip) from looking direction,
                        // mirroring the map editor's formula in map.rs:473-479.
                        let dir = entity.look_direction;
                        let (seq, flip) = if dir > 4 {
                            ((8 - dir) as usize, true)
                        } else {
                            (dir as usize, false)
                        };
                        let frames = load_sprite_frames(&path)?;
                        let frame = frames.get(seq).or_else(|| frames.first())?;
                        let w = frame.image.width();
                        let h = frame.image.height();
                        return Some(EntitySpriteHandle {
                            handle: Handle::from_rgba(w, h, frame.image.as_raw().to_vec()),
                            width: w,
                            height: h,
                            origin_x: frame.origin_x,
                            origin_y: frame.origin_y,
                            flip,
                        });
                    };
                    let w = frame.image.width();
                    let h = frame.image.height();
                    Some(EntitySpriteHandle {
                        handle: Handle::from_rgba(w, h, frame.image.as_raw().to_vec()),
                        width: w,
                        height: h,
                        origin_x: frame.origin_x,
                        origin_y: frame.origin_y,
                        flip: false,
                    })
                })
                .clone()
        })
        .collect();

    Ok(crate::editors::save_file_viewer::message::PreviewSpritesLoaded { sprites })
}

/// Decode all internal sprites (thrones, decor, vases …) from the .map file.
///
/// Each sprite block references a sprite sequence + frame, and we decode
/// frame[0] of each placement.  This mirrors `decode_internal_map_sprites()`
/// in `map_editor/update/map.rs`.
fn decode_internal_preview_sprites(
    map_path: &Path,
    map_data: &dispel_core::map::MapData,
) -> Vec<crate::components::map_render::InternalSpriteHandle> {
    use iced::widget::image::Handle;
    use std::io::{Read, Seek, SeekFrom};

    let mut file = match std::fs::File::open(map_path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    let nox = map_data.model.map_non_occluded_start_x;
    let noy = map_data.model.map_non_occluded_start_y;

    let mut result = Vec::new();
    for block in &map_data.sprite_blocks {
        let Some(sequence) = map_data.internal_sprites.get(block.sprite_id) else {
            continue;
        };
        let Some(frame) = sequence.frame_infos.first() else {
            continue;
        };
        if frame.width <= 0 || frame.height <= 0 {
            continue;
        }
        if file
            .seek(SeekFrom::Start(frame.image_start_position))
            .is_err()
        {
            continue;
        }

        let w = frame.width as u32;
        let h = frame.height as u32;
        let pixel_count = (w * h) as usize;
        let mut raw = vec![0u8; pixel_count * 2];
        if file.read_exact(&mut raw).is_err() {
            continue;
        }

        let mut pixels = vec![0u8; pixel_count * 4];
        for i in 0..pixel_count {
            let lo = raw[i * 2] as u16;
            let hi = raw[i * 2 + 1] as u16;
            let pixel = lo | (hi << 8);
            if pixel > 0 {
                let r5 = ((pixel >> 11) & 0x1F) as u32;
                let g6 = ((pixel >> 5) & 0x3F) as u32;
                let b5 = (pixel & 0x1F) as u32;
                let idx = i * 4;
                pixels[idx] = (r5 * 255 / 31) as u8;
                pixels[idx + 1] = (g6 * 255 / 63) as u8;
                pixels[idx + 2] = (b5 * 255 / 31) as u8;
                pixels[idx + 3] = 255;
            }
        }

        result.push(crate::components::map_render::InternalSpriteHandle {
            handle: Handle::from_rgba(w, h, pixels),
            x: block.sprite_x + nox,
            y: block.sprite_y + noy,
            sort_y: block.sprite_bottom_right_y,
            width: w,
            height: h,
        });
    }
    result
}

/// Load Extra.ini → `HashMap<id, sprite_filename>`.
///
/// Tries `Extra::read_file()` (EUC-KR encoding per struct definition) first.
/// If the declared encoding rejects the file (Polish game version uses
/// WINDOWS-1250 for non-ASCII description fields), falls back to a raw-ASCII
/// CSV parse — the first two columns (id and sprite_filename) are always
/// pure ASCII and encoding-independent.
fn load_extra_ini_sprites(game_path: &Path) -> Result<HashMap<i32, String>, String> {
    let path = game_path.join("Extra.ini");
    // Try canonical Extractor read (EUC-KR encoding) first.
    if let Ok(extras) = Extra::read_file(&path) {
        return Ok(extras
            .into_iter()
            .filter_map(|e| e.sprite_filename.map(|s| (e.id, s)))
            .collect());
    }
    // Fallback: raw-bytes CSV parse (encoding-agnostic).
    let data = std::fs::read(&path).map_err(|e| format!("Cannot read Extra.ini: {}", e))?;
    let text = String::from_utf8_lossy(&data);
    let mut map = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        let mut cols = line.splitn(4, ',');
        let id: i32 = match cols.next().and_then(|s| s.trim().parse().ok()) {
            Some(id) => id,
            None => continue,
        };
        let sprite = cols
            .next()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s != "null");
        if let Some(s) = sprite {
            map.insert(id, s);
        }
    }
    Ok(map)
}

// ── CSV export helpers ─────────────────────────────────────────────────────

/// Build a default filename for a CSV export based on the table key.
fn csv_default_filename(key: crate::editors::save_file_viewer::message::TableKey) -> String {
    use crate::editors::save_file_viewer::message::TableKey;
    match key {
        TableKey::Inventory(cat) => format!("inventory-{}.csv", cat.label()),
        TableKey::Events => "events.csv".to_string(),
        TableKey::Journal(section) => {
            let label = match section {
                crate::editors::save_file_viewer::state::JournalSection::Main => "main",
                crate::editors::save_file_viewer::state::JournalSection::Side => "side",
                crate::editors::save_file_viewer::state::JournalSection::Trade => "trade",
            };
            format!("journal-{label}.csv")
        }
        TableKey::Map(_, kind) => {
            let label = match kind {
                crate::editors::save_file_viewer::state::MapsTableKind::Monsters => {
                    "monsters"
                }
                crate::editors::save_file_viewer::state::MapsTableKind::Npcs => "npcs",
                crate::editors::save_file_viewer::state::MapsTableKind::ExtraObjects => {
                    "extra-objects"
                }
                crate::editors::save_file_viewer::state::MapsTableKind::Weapon => {
                    "weapons"
                }
                crate::editors::save_file_viewer::state::MapsTableKind::Heal => "heals",
                crate::editors::save_file_viewer::state::MapsTableKind::Edit => "edits",
                crate::editors::save_file_viewer::state::MapsTableKind::Misc => "misc",
                crate::editors::save_file_viewer::state::MapsTableKind::Event => {
                    "events"
                }
            };
            format!("map-{label}.csv")
        }
    }
}

/// Resolve (column headers, filtered rows) for a table identified by `key`.
/// Returns `None` when the table has no data (empty cache or missing state).
fn resolve_csv_export_data(
    state: &crate::editors::save_file_viewer::state::SaveFileViewerState,
    key: crate::editors::save_file_viewer::message::TableKey,
) -> Option<(Vec<String>, Vec<Vec<String>>)> {
    use crate::editors::save_file_viewer::message::TableKey;
    use crate::editors::save_file_viewer::state::MapsTableKind;

    match key {
        TableKey::Inventory(cat) => {
            let headers: Vec<String> = cat
                .default_columns()
                .iter()
                .map(|c| c.label.clone())
                .collect();
            let cache = state.inventory_display_caches.get(&cat)?;
            let indices = state.inventory_filtered_indices.get(&cat)?;
            let rows: Vec<Vec<String>> = indices
                .iter()
                .filter_map(|&i| cache.get(i).cloned())
                .collect();
            Some((headers, rows))
        }
        TableKey::Events => {
            let headers: Vec<String> =
                crate::editors::save_file_viewer::state::events_default_columns()
                    .iter()
                    .map(|c| c.label.clone())
                    .collect();
            let rows: Vec<Vec<String>> = state
                .events_filtered_indices
                .iter()
                .filter_map(|&i| state.events_display_cache.get(i).cloned())
                .collect();
            Some((headers, rows))
        }
        TableKey::Journal(section) => {
            let headers: Vec<String> = section
                .default_columns()
                .iter()
                .map(|c| c.label.clone())
                .collect();
            let cache = state.journal_display_caches.get(&section)?;
            let indices = state.journal_filtered_indices.get(&section)?;
            let rows: Vec<Vec<String>> = indices
                .iter()
                .filter_map(|&i| cache.get(i).cloned())
                .collect();
            Some((headers, rows))
        }
        TableKey::Map(map_idx, kind) => {
            let headers: Vec<String> = kind
                .default_columns()
                .iter()
                .map(|c| c.label.clone())
                .collect();
            let cache = state.maps_display_caches.get(map_idx)?;
            let (rows_data, indices_slice): (&[Vec<String>], &[usize]) = match kind {
                MapsTableKind::Monsters => (&cache.monsters, &cache.monsters_indices),
                MapsTableKind::Npcs => (&cache.npcs, &cache.npcs_indices),
                MapsTableKind::ExtraObjects => {
                    (&cache.extra_objects, &cache.extra_objects_indices)
                }
                MapsTableKind::Weapon => {
                    (&cache.draw_items_weapon, &cache.draw_items_weapon_indices)
                }
                MapsTableKind::Heal => (&cache.draw_items_heal, &cache.draw_items_heal_indices),
                MapsTableKind::Edit => (&cache.draw_items_edit, &cache.draw_items_edit_indices),
                MapsTableKind::Misc => (&cache.draw_items_misc, &cache.draw_items_misc_indices),
                MapsTableKind::Event => {
                    (&cache.draw_items_event, &cache.draw_items_event_indices)
                }
            };
            let rows: Vec<Vec<String>> = indices_slice
                .iter()
                .filter_map(|&i| rows_data.get(i).cloned())
                .collect();
            Some((headers, rows))
        }
    }
}

/// Case-insensitive sprite file path resolution.
///
/// Tries original → uppercase → lowercase under `game_path/{sub_dir}/{filename}`.
fn resolve_sprite_path(game_path: &Path, sub_dir: &str, filename: &str) -> Option<PathBuf> {
    let base = game_path.join(sub_dir);
    for name in [
        filename.to_string(),
        filename.to_ascii_uppercase(),
        filename.to_ascii_lowercase(),
    ] {
        let p = base.join(&name);
        if p.exists() {
            return Some(p);
        }
    }
    None
}
