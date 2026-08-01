use iced::Task;

use crate::app::App;
use crate::editors::save_file_viewer::map_preview::state::EntityKind;
use crate::editors::save_file_viewer::message::{SaveFileViewerMessage, TableKey};
use crate::editors::save_file_viewer::state::{ResizeDrag, SaveFileViewerState};
use crate::message::{Message, MessageExt};

pub(crate) mod csv_export;
pub(crate) mod filter;
pub(crate) mod preview;
pub(crate) mod table;

use self::csv_export::{csv_default_filename, resolve_csv_export_data};
use self::filter::{compare_cells, handle_table_filter};
use self::preview::load_preview_sprites;
use self::table::{
    apply_resize_cursor, auto_size_column, events_table_data, hex_bytes, inventory_table_data,
    journal_table_data, maps_table_data, maps_table_indices, maps_table_rows,
};

/// Handle a `*StartResize` press for any table. Returns `None` when the press
/// is recognised as a double-press (same table + column within 400 ms) — the
/// column is auto-sized and no drag should start. Returns `Some(drag)` for a
/// normal single press, which the caller should set as `state.resizing`.
///
/// We detect double-press here at the app level rather than relying on the
/// widget's built-in double-click detection because the first press makes
/// `state.resizing = Some(…)` which causes the view to wrap the table in a
/// `mouse_area`; that outer layer intercepts the second click before it can
/// reach the widget's internal handler.
pub fn try_begin_column_resize(
    state: &mut SaveFileViewerState,
    key: TableKey,
    col: usize,
) -> Option<ResizeDrag> {
    const DOUBLE_PRESS_MS: u128 = 400;
    let now = std::time::Instant::now();

    // Check for double-press
    if let Some((last_key, last_col, last_time)) = state.last_resize_press {
        if last_key == key
            && last_col == col
            && now.duration_since(last_time).as_millis() < DOUBLE_PRESS_MS
        {
            state.last_resize_press = None;
            // Auto-size the column
            auto_size_column_by_key(state, key, col);
            return None; // Don't start a drag
        }
    }

    state.last_resize_press = Some((key, col, now));

    let anchor_width = column_width_by_key(state, key, col);
    Some(ResizeDrag {
        key,
        col,
        anchor_width,
        anchor_cursor_x: None,
    })
}

/// Get the current width of a column for a given table key.
fn column_width_by_key(state: &SaveFileViewerState, key: TableKey, col: usize) -> f32 {
    match key {
        TableKey::Events => state
            .events_table_state
            .column_widths
            .get(col)
            .copied()
            .unwrap_or(80.0),
        TableKey::Map(map, kind) => state
            .maps_table_states
            .get(map)
            .and_then(|m| m.get(&kind))
            .and_then(|ts| ts.column_widths.get(col).copied())
            .unwrap_or(80.0),
        TableKey::Inventory(cat) => state
            .inventory_table_states
            .get(&cat)
            .and_then(|ts| ts.column_widths.get(col).copied())
            .unwrap_or(80.0),
        TableKey::Journal(section) => state
            .journal_table_states
            .get(&section)
            .and_then(|ts| ts.column_widths.get(col).copied())
            .unwrap_or(80.0),
    }
}

/// Compute and apply an auto-size width for a column identified by `key`.
fn auto_size_column_by_key(state: &mut SaveFileViewerState, key: TableKey, col: usize) {
    let header = column_label_by_key(key, col);

    let width = match key {
        TableKey::Events => auto_size_column(
            &state.events_display_cache,
            &state.events_filtered_indices,
            col,
            &header,
        ),
        TableKey::Inventory(cat) => {
            let Some(rows) = state.inventory_display_caches.get(&cat) else {
                return;
            };
            let Some(indices) = state.inventory_filtered_indices.get(&cat) else {
                return;
            };
            auto_size_column(rows, indices, col, &header)
        }
        TableKey::Journal(section) => {
            let Some(rows) = state.journal_display_caches.get(&section) else {
                return;
            };
            let Some(indices) = state.journal_filtered_indices.get(&section) else {
                return;
            };
            auto_size_column(rows, indices, col, &header)
        }
        TableKey::Map(map, kind) => {
            let Some(cache) = state.maps_display_caches.get(map) else {
                return;
            };
            let rows = maps_table_rows(cache, kind);
            let indices = maps_table_indices(cache, kind);
            auto_size_column(rows, indices, col, &header)
        }
    };

    apply_column_width(state, key, col, width);
}

/// Get the column header label for a table key.
fn column_label_by_key(key: TableKey, col: usize) -> String {
    match key {
        TableKey::Events => crate::editors::save_file_viewer::state::events_default_columns()
            .into_iter()
            .nth(col)
            .map(|c| c.label)
            .unwrap_or_default(),
        TableKey::Map(_, kind) => kind
            .default_columns()
            .into_iter()
            .nth(col)
            .map(|c| c.label)
            .unwrap_or_default(),
        TableKey::Inventory(cat) => cat
            .default_columns()
            .into_iter()
            .nth(col)
            .map(|c| c.label)
            .unwrap_or_default(),
        TableKey::Journal(section) => section
            .default_columns()
            .into_iter()
            .nth(col)
            .map(|c| c.label)
            .unwrap_or_default(),
    }
}

/// Set the width of a column for a given table key.
fn apply_column_width(state: &mut SaveFileViewerState, key: TableKey, col: usize, width: f32) {
    match key {
        TableKey::Events => {
            if let Some(w) = state.events_table_state.column_widths.get_mut(col) {
                *w = width;
            }
        }
        TableKey::Map(map, kind) => {
            if let Some(ts) = state
                .maps_table_states
                .get_mut(map)
                .and_then(|m| m.get_mut(&kind))
            {
                if let Some(w) = ts.column_widths.get_mut(col) {
                    *w = width;
                }
            }
        }
        TableKey::Inventory(cat) => {
            if let Some(ts) = state.inventory_table_states.get_mut(&cat) {
                if let Some(w) = ts.column_widths.get_mut(col) {
                    *w = width;
                }
            }
        }
        TableKey::Journal(section) => {
            if let Some(ts) = state.journal_table_states.get_mut(&section) {
                if let Some(w) = ts.column_widths.get_mut(col) {
                    *w = width;
                }
            }
        }
    }
}

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
            state.resizing = None;
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
                        let x = to_tile(e.x_pos);
                        let y = to_tile(e.y_pos);
                        if x != 0 || y != 0 {
                            entities.push(PreviewEntity {
                                kind: EntityKind::Extra,
                                label: e.name.clone(),
                                tile_x: x,
                                tile_y: y,
                                confirmed: false,
                                db_id: Some(e.extra_ini_id as i32),
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
                    let internal_sprites = match std::fs::File::open(&map_path) {
                        Ok(file) => {
                            let mut reader = std::io::BufReader::new(file);
                            crate::components::map_render::decode::decode_internal_sprites(
                                &mut reader,
                                &loaded.map_data,
                            )
                        }
                        Err(_) => Vec::new(),
                    };

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
            let drag = try_begin_column_resize(state, TableKey::Map(map, kind), col);
            state.resizing = drag;
            Task::none()
        }
        SaveFileViewerMessage::MapsTableResetColumnWidth { map, kind, col } => {
            let header = kind
                .default_columns()
                .into_iter()
                .nth(col)
                .map(|c| c.label)
                .unwrap_or_default();
            let width = if let Some(cache) = state.maps_display_caches.get(map) {
                let rows = maps_table_rows(cache, kind);
                let indices = maps_table_indices(cache, kind);
                auto_size_column(rows, indices, col, &header)
            } else {
                kind.default_columns()
                    .into_iter()
                    .nth(col)
                    .map(|c| c.width_px)
                    .unwrap_or(80.0)
            };
            if let Some(ts) = state
                .maps_table_states
                .get_mut(map)
                .and_then(|m| m.get_mut(&kind))
            {
                if let Some(w) = ts.column_widths.get_mut(col) {
                    *w = width;
                }
            }
            Task::none()
        }
        SaveFileViewerMessage::MapsTableResizeCursor(x) => {
            apply_resize_cursor(state, x);
            Task::none()
        }
        SaveFileViewerMessage::MapsTableEndResize => {
            state.resizing = None;
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
            let drag = try_begin_column_resize(state, TableKey::Inventory(cat), col);
            state.resizing = drag;
            Task::none()
        }
        SaveFileViewerMessage::InventoryTableResetColumnWidth { cat, col } => {
            let header = cat
                .default_columns()
                .into_iter()
                .nth(col)
                .map(|c| c.label)
                .unwrap_or_default();
            let width = if let (Some(cache), Some(indices)) = (
                state.inventory_display_caches.get(&cat),
                state.inventory_filtered_indices.get(&cat),
            ) {
                auto_size_column(cache, indices, col, &header)
            } else {
                cat.default_columns()
                    .into_iter()
                    .nth(col)
                    .map(|c| c.width_px)
                    .unwrap_or(80.0)
            };
            if let Some(ts) = state.inventory_table_states.get_mut(&cat) {
                if let Some(w) = ts.column_widths.get_mut(col) {
                    *w = width;
                }
            }
            Task::none()
        }
        SaveFileViewerMessage::InventoryTableResizeCursor(x) => {
            apply_resize_cursor(state, x);
            Task::none()
        }
        SaveFileViewerMessage::InventoryTableEndResize => {
            state.resizing = None;
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
            let drag = try_begin_column_resize(state, TableKey::Events, col);
            state.resizing = drag;
            Task::none()
        }
        SaveFileViewerMessage::EventsTableResetColumnWidth { col } => {
            let header = crate::editors::save_file_viewer::state::events_default_columns()
                .into_iter()
                .nth(col)
                .map(|c| c.label)
                .unwrap_or_default();
            let width = auto_size_column(
                &state.events_display_cache,
                &state.events_filtered_indices,
                col,
                &header,
            );
            if let Some(w) = state.events_table_state.column_widths.get_mut(col) {
                *w = width;
            }
            Task::none()
        }
        SaveFileViewerMessage::EventsTableResizeCursor(x) => {
            apply_resize_cursor(state, x);
            Task::none()
        }
        SaveFileViewerMessage::EventsTableEndResize => {
            state.resizing = None;
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
            let drag = try_begin_column_resize(state, TableKey::Journal(section), col);
            state.resizing = drag;
            Task::none()
        }
        SaveFileViewerMessage::JournalTableResetColumnWidth { section, col } => {
            let header = section
                .default_columns()
                .into_iter()
                .nth(col)
                .map(|c| c.label)
                .unwrap_or_default();
            let width = if let (Some(cache), Some(indices)) = (
                state.journal_display_caches.get(&section),
                state.journal_filtered_indices.get(&section),
            ) {
                auto_size_column(cache, indices, col, &header)
            } else {
                section
                    .default_columns()
                    .into_iter()
                    .nth(col)
                    .map(|c| c.width_px)
                    .unwrap_or(80.0)
            };
            if let Some(ts) = state.journal_table_states.get_mut(&section) {
                if let Some(w) = ts.column_widths.get_mut(col) {
                    *w = width;
                }
            }
            Task::none()
        }
        SaveFileViewerMessage::JournalTableResizeCursor(x) => {
            apply_resize_cursor(state, x);
            Task::none()
        }
        SaveFileViewerMessage::JournalTableEndResize => {
            state.resizing = None;
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
                move |result| Message::save_file_viewer(SaveFileViewerMessage::CsvExported(result)),
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
                    state.map_name_lookup = loaded.map_names;
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
                                    item.item_type_id.to_string(),
                                    item.position_index.to_string(),
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
                                    item.item_type_id.to_string(),
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
                                    item.item_type_id.to_string(),
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
                                    item.item_type_id.to_string(),
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
                                            m.monster_state.to_string(),
                                            m.record_index.to_string(),
                                            m.sprite_frame_id.to_string(),
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
                                            m.unknown_3a.to_string(),
                                            m.unknown_3b.to_string(),
                                            m.unknown_3c.to_string(),
                                            m.unknown_3d.to_string(),
                                            hex_bytes(&m.unknown_3e),
                                            m.unknown_3f.to_string(),
                                            m.event_id_on_kill.to_string(),
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
                                            e.extra_ref_record_id.to_string(),
                                            e.extra_ini_id.to_string(),
                                            e.name.clone(),
                                            e.object_type.to_string(),
                                            e.x_pos.to_string(),
                                            e.y_pos.to_string(),
                                            e.rotation.to_string(),
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
                                            e.event_ini_id.to_string(),
                                            e.message_scr_id.to_string(),
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
                    use crate::editors::save_file_viewer::state::MapsTableKind;
                    let mut table_states: Vec<
                        std::collections::HashMap<MapsTableKind, TableInteractionState>,
                    > = Vec::with_capacity(state.maps_display_caches.len());
                    for _ in &state.maps_display_caches {
                        let mut per_map = std::collections::HashMap::new();
                        for kind in MapsTableKind::all() {
                            let widths: Vec<f32> =
                                kind.default_columns().iter().map(|c| c.width_px).collect();
                            per_map.insert(
                                *kind,
                                TableInteractionState {
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
                                // let hex_rest: Vec<String> =
                                //     entry.rest.iter().map(|b| format!("{:02X}", b)).collect();
                                vec![
                                    format!("{}", entry.index),
                                    entry.name.clone(),
                                    entry.unknown_1.to_string(),
                                    entry.unknown_2a.to_string(),
                                    entry.unknown_2b.to_string(),
                                    entry.unknown_3a.to_string(),
                                    entry.unknown_3b.to_string(),
                                    entry.unknown_4a.to_string(),
                                    entry.unknown_4b.to_string(),
                                    entry.unknown_5a.to_string(),
                                    entry.quest_scr_id.to_string(),
                                    entry.quest_scr_id_progress1.to_string(),
                                    entry.quest_scr_id_progress2.to_string(),
                                    entry.is_completed.to_string(),
                                    // hex_rest.join(" "),
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
        SaveFileViewerMessage::MapPreview(msg) => {
            let Some(preview) = state.map_preview.as_mut() else {
                return Task::none();
            };
            crate::editors::save_file_viewer::map_preview::handle(msg, preview)
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
