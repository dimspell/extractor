use crate::app::App;
use crate::components::editor::editable::EditableRecord;
use crate::components::map_canvas::{decode_tileset_file, find_hovered_entity_impl};
use crate::loading_state::LoadingState;
use crate::message::editor::map_editor::{
    DecodedEntitySprite, DecodedMapSprite, EntityBundle, MapDataHandle, MapEditorMessage, MapLayer,
    MapViewMode, SelectedEntity, TilePixelData,
};
use crate::message::{Message, MessageExt};
use crate::state::map_editor::{
    EntitySpriteHandle, InternalSpriteHandle, MapEditAction, SpriteSequenceHandle,
};
use dispel_core::references::extractor::Extractor;
use iced::widget::image::Handle;
use iced::Task;
use std::collections::HashSet;
use std::sync::Arc;

/// Duration before a status message is automatically cleared.
const STATUS_DISMISS_SECS: u64 = 3;

pub fn handle(message: MapEditorMessage, app: &mut App) -> Task<Message> {
    match message {
        MapEditorMessage::Open(tab_id, path) => {
            let state = app.state.map_editors.entry(tab_id).or_default();
            state.data.map_path = Some(path.clone());
            state.data.loading_state = LoadingState::Loading;
            state.data.tiles_ready = false;
            state.data.internal_sprite_handles.clear();
            state.data.sprite_sequence_handles.clear();
            state.view.view_mode = MapViewMode::Map;
            state.view.selected_sprite_sequence = None;
            state.view.tile_layer_cache.clear();
            state.view.overlay_cache.clear();

            Task::perform(
                async move {
                    let file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
                    let mut reader = std::io::BufReader::new(file);
                    let map_data =
                        dispel_core::map::read_map_data(&mut reader).map_err(|e| e.to_string())?;

                    // While the file is still open, decode internal sprites (thrones, etc.)
                    let sprites = decode_internal_map_sprites(&mut reader, &map_data);

                    Ok((MapDataHandle(Arc::new(map_data)), sprites))
                },
                move |result| Message::map_editor(MapEditorMessage::MapLoaded(tab_id, result)),
            )
        }

        MapEditorMessage::MapLoaded(tab_id, result) => {
            let state = match app.state.map_editors.get_mut(&tab_id) {
                Some(s) => s,
                None => return Task::none(),
            };

            match result {
                Ok((handle, decoded_sprites)) => {
                    let arc_data = handle.0.clone();
                    let nox = arc_data.model.map_non_occluded_start_x;
                    let noy = arc_data.model.map_non_occluded_start_y;

                    // Build per-sequence thumbnail data for the Sprites browser.
                    // decoded_sprites is parallel to arc_data.sprite_blocks.
                    let mut seq_first: std::collections::HashMap<usize, (u32, u32, Vec<u8>)> =
                        std::collections::HashMap::new();
                    let mut seq_placements: std::collections::HashMap<usize, Vec<(i32, i32)>> =
                        std::collections::HashMap::new();
                    for (i, sprite) in decoded_sprites.iter().enumerate() {
                        if let Some(block) = arc_data.sprite_blocks.get(i) {
                            let sid = block.sprite_id;
                            seq_first.entry(sid).or_insert_with(|| {
                                (sprite.width, sprite.height, sprite.pixels.clone())
                            });
                            seq_placements
                                .entry(sid)
                                .or_default()
                                .push((block.sprite_x, block.sprite_y));
                        }
                    }
                    let mut seq_handles: Vec<SpriteSequenceHandle> = seq_first
                        .into_iter()
                        .map(|(sid, (w, h, pixels))| SpriteSequenceHandle {
                            sequence_idx: sid,
                            handle: Handle::from_rgba(w, h, pixels),
                            width: w,
                            height: h,
                            placement_count: seq_placements.get(&sid).map(|v| v.len()).unwrap_or(0),
                            placements: seq_placements.remove(&sid).unwrap_or_default(),
                        })
                        .collect();
                    seq_handles.sort_by_key(|s| s.sequence_idx);
                    state.data.sprite_sequence_handles = seq_handles;

                    // Convert decoded internal sprites → Iced Handles.
                    state.data.internal_sprite_handles = decoded_sprites
                        .into_iter()
                        .map(|s| InternalSpriteHandle {
                            x: s.x + nox,
                            y: s.y + noy,
                            sort_y: s.bottom_right_y,
                            handle: Handle::from_rgba(s.width, s.height, s.pixels),
                            width: s.width,
                            height: s.height,
                        })
                        .collect();

                    state.data.loading_state = LoadingState::Loaded(handle);
                    state.view.tile_layer_cache.clear();
                    state.view.overlay_cache.clear();

                    let map_path = match &state.data.map_path {
                        Some(p) => p.clone(),
                        None => return Task::none(),
                    };
                    let gtl_path = map_path.with_extension("gtl");
                    let btl_path = map_path.with_extension("btl");
                    state.data.gtl_path = Some(gtl_path.clone());
                    state.data.btl_path = Some(btl_path.clone());

                    Task::perform(
                        async move {
                            let gtl_ids: HashSet<i32> =
                                arc_data.gtl_tiles.values().copied().collect();
                            let btl_ids: HashSet<i32> =
                                arc_data
                                    .btl_tiles
                                    .values()
                                    .copied()
                                    .chain(arc_data.tiled_infos.iter().flat_map(|t| {
                                        t.ids.iter().map(|&id| id.unsigned_abs() as i32)
                                    }))
                                    .filter(|&id| id > 0)
                                    .collect();

                            let gtl = decode_tileset_file(&gtl_path, &gtl_ids).unwrap_or_default();
                            let btl = decode_tileset_file(&btl_path, &btl_ids).unwrap_or_default();

                            Ok(TilePixelData { gtl, btl })
                        },
                        move |result| {
                            Message::map_editor(MapEditorMessage::TilesDecoded(tab_id, result))
                        },
                    )
                }
                Err(e) => {
                    state.data.loading_state = LoadingState::Failed(e.clone());
                    Task::done(Message::System(crate::message::SystemMessage::ShowError(
                        format!("Failed to load map: {}", e),
                    )))
                }
            }
        }

        MapEditorMessage::TilesDecoded(tab_id, result) => {
            let state = match app.state.map_editors.get_mut(&tab_id) {
                Some(s) => s,
                None => return Task::none(),
            };

            match result {
                Ok(pixel_data) => {
                    state.data.gtl_handles = pixel_data
                        .gtl
                        .into_iter()
                        .map(|(id, px)| (id, Handle::from_rgba(62, 32, px)))
                        .collect();
                    state.data.btl_handles = pixel_data
                        .btl
                        .into_iter()
                        .map(|(id, px)| (id, Handle::from_rgba(62, 32, px)))
                        .collect();
                    state.data.tiles_ready = true;
                    state.view.tile_layer_cache.clear();
                    state.view.overlay_cache.clear();
                }
                Err(e) => {
                    eprintln!("Tile decode failed for tab {}: {}", tab_id, e);
                }
            }

            // Centre the map at 100% zoom using the last known canvas size (defaults
            // to 1200×800 until the user moves the mouse over the canvas).
            let center = state.map_data().map(|h| {
                let model = &h.0.model;
                let diagonal = model.tiled_map_width + model.tiled_map_height;
                let (cx, cy) = dispel_core::map::types::convert_map_coords_to_image_coords(
                    model.tiled_map_width / 2,
                    model.tiled_map_height / 2,
                    diagonal,
                );
                (cx as f32, cy as f32)
            });
            if let Some((center_px, center_py)) = center {
                let vp_w = state.view.last_canvas_w;
                let vp_h = state.view.last_canvas_h;
                state.view.zoom = 1.0;
                state.view.pan_x = vp_w / 2.0 - center_px;
                state.view.pan_y = vp_h / 2.0 - center_py;
                state.view.tile_layer_cache.clear();
                state.view.overlay_cache.clear();
            }

            let map_path = match &state.data.map_path {
                Some(p) => p.clone(),
                None => return Task::none(),
            };
            let game_path = app.state.workspace.game_path.clone();
            if game_path.is_none() {
                state.data.status_msg =
                    Some("No game path set — entity files not loaded".to_string());
            }
            Task::perform(
                async move { load_entities(&map_path, game_path) },
                move |bundle| Message::map_editor(MapEditorMessage::EntitiesLoaded(tab_id, bundle)),
            )
        }

        MapEditorMessage::EntitiesLoaded(tab_id, bundle) => {
            if let Some(state) = app.state.map_editors.get_mut(&tab_id) {
                state.data.monsters = bundle.monsters;
                state.data.npcs = bundle.npcs;
                state.data.extra_refs = bundle.extra_refs;
                state.data.monster_ref_path = bundle.monster_ref_path;
                state.data.npc_ref_path = bundle.npc_ref_path;
                state.data.extra_ref_path = bundle.extra_ref_path;

                state.data.monster_sprites = bundle
                    .monster_sprites
                    .into_iter()
                    .map(|opt| {
                        opt.map(|s| EntitySpriteHandle {
                            handle: Handle::from_rgba(s.width, s.height, s.pixels),
                            width: s.width,
                            height: s.height,
                            origin_x: s.origin_x,
                            origin_y: s.origin_y,
                            flip: s.flip,
                        })
                    })
                    .collect();
                state.data.npc_sprites = bundle
                    .npc_sprites
                    .into_iter()
                    .map(|opt| {
                        opt.map(|s| EntitySpriteHandle {
                            handle: Handle::from_rgba(s.width, s.height, s.pixels),
                            width: s.width,
                            height: s.height,
                            origin_x: s.origin_x,
                            origin_y: s.origin_y,
                            flip: s.flip,
                        })
                    })
                    .collect();
                state.data.extra_sprites = bundle
                    .extra_sprites
                    .into_iter()
                    .map(|opt| {
                        opt.map(|s| EntitySpriteHandle {
                            handle: Handle::from_rgba(s.width, s.height, s.pixels),
                            width: s.width,
                            height: s.height,
                            origin_x: s.origin_x,
                            origin_y: s.origin_y,
                            flip: s.flip,
                        })
                    })
                    .collect();

                state.view.tile_layer_cache.clear();
                state.view.overlay_cache.clear();
            }
            Task::none()
        }

        MapEditorMessage::PanChanged(tab_id, dx, dy) => {
            if let Some(state) = app.state.map_editors.get_mut(&tab_id) {
                state.view.pan_x += dx;
                state.view.pan_y += dy;
                state.view.tile_layer_cache.clear();
                state.view.overlay_cache.clear();
            }
            Task::none()
        }

        MapEditorMessage::ZoomChanged(tab_id, factor, cx, cy) => {
            if let Some(state) = app.state.map_editors.get_mut(&tab_id) {
                let old_zoom = state.view.zoom;
                let new_zoom = (old_zoom * factor).clamp(0.05, 8.0);
                let ratio = new_zoom / old_zoom;
                // cx/cy are NaN when triggered from a toolbar button (no cursor position).
                if cx.is_finite() && cy.is_finite() {
                    state.view.pan_x = cx - (cx - state.view.pan_x) * ratio;
                    state.view.pan_y = cy - (cy - state.view.pan_y) * ratio;
                }
                state.view.zoom = new_zoom;
                state.view.tile_layer_cache.clear();
                state.view.overlay_cache.clear();
            }
            Task::none()
        }

        MapEditorMessage::LayerToggled(tab_id, layer) => {
            if let Some(state) = app.state.map_editors.get_mut(&tab_id) {
                match layer {
                    MapLayer::Ground => state.view.show_ground = !state.view.show_ground,
                    MapLayer::Buildings => state.view.show_buildings = !state.view.show_buildings,
                    MapLayer::Roofs => state.view.show_roofs = !state.view.show_roofs,
                    MapLayer::InternalSprites => {
                        state.view.show_internal_sprites = !state.view.show_internal_sprites
                    }
                    MapLayer::Collisions => {
                        state.view.show_collisions = !state.view.show_collisions
                    }
                    MapLayer::Events => state.view.show_events = !state.view.show_events,
                    MapLayer::Monsters => state.view.show_monsters = !state.view.show_monsters,
                    MapLayer::Npcs => state.view.show_npcs = !state.view.show_npcs,
                    MapLayer::Objects => state.view.show_objects = !state.view.show_objects,
                }
                // Tile canvas renders entities and tile layers; overlay renders
                // collisions and events — clear both caches.
                state.view.tile_layer_cache.clear();
                state.view.overlay_cache.clear();
            }
            Task::none()
        }

        MapEditorMessage::MouseMoved(tab_id, x, y, canvas_w, canvas_h) => {
            if let Some(state) = app.state.map_editors.get_mut(&tab_id) {
                state.view.cursor_canvas_x = x;
                state.view.cursor_canvas_y = y;
                if canvas_w > 0.0 && canvas_h > 0.0 {
                    state.view.last_canvas_w = canvas_w;
                    state.view.last_canvas_h = canvas_h;
                }
                // Intentionally NOT clearing tile_layer_cache or overlay_cache here:
                // cursor moves are high-frequency and only affect the cursor-dependent
                // part of the overlay (tile highlight, hover ring, coord label), which
                // is drawn uncached on every frame anyway.
            }
            Task::none()
        }

        MapEditorMessage::Deselect(tab_id) => {
            if let Some(state) = app.state.map_editors.get_mut(&tab_id) {
                state.view.selected_entity = None;
                state.view.overlay_cache.clear();
            }
            Task::none()
        }

        MapEditorMessage::CanvasClicked(tab_id, cx, cy) => {
            if let Some(state) = app.state.map_editors.get_mut(&tab_id) {
                // Reuse the same hit-test as the hover highlight — single source of truth.
                state.view.selected_entity = find_hovered_entity_impl(state, cx, cy);
                state.view.overlay_cache.clear();
            }
            Task::none()
        }

        MapEditorMessage::EntityFieldChanged(tab_id, entity, field, value) => {
            let state = match app.state.map_editors.get_mut(&tab_id) {
                Some(s) => s,
                None => return Task::none(),
            };
            // Capture old value before mutating (for undo).
            let old_value = match entity {
                SelectedEntity::Monster(i) => state
                    .data
                    .monsters
                    .get(i)
                    .map(|m| m.get_field(&field))
                    .unwrap_or_default(),
                SelectedEntity::Npc(i) => state
                    .data
                    .npcs
                    .get(i)
                    .map(|n| n.get_field(&field))
                    .unwrap_or_default(),
                SelectedEntity::Extra(i) => state
                    .data
                    .extra_refs
                    .get(i)
                    .map(|e| e.get_field(&field))
                    .unwrap_or_default(),
            };
            // Apply the change.
            match entity {
                SelectedEntity::Monster(i) => {
                    if let Some(m) = state.data.monsters.get_mut(i) {
                        m.set_field(&field, value.clone());
                    }
                }
                SelectedEntity::Npc(i) => {
                    if let Some(n) = state.data.npcs.get_mut(i) {
                        n.set_field(&field, value.clone());
                    }
                }
                SelectedEntity::Extra(i) => {
                    if let Some(e) = state.data.extra_refs.get_mut(i) {
                        e.set_field(&field, value.clone());
                    }
                }
            }
            if old_value != value {
                state.push_undo(MapEditAction {
                    entity,
                    field,
                    old_value,
                    new_value: value,
                });
                // Entity positions live on the tile canvas; selection ring on the overlay.
                state.view.tile_layer_cache.clear();
                state.view.overlay_cache.clear();
                set_tab_modified(app, tab_id, true);
            }
            Task::none()
        }

        MapEditorMessage::Undo(tab_id) => {
            let state = match app.state.map_editors.get_mut(&tab_id) {
                Some(s) => s,
                None => return Task::none(),
            };
            if let Some(action) = state.pop_undo() {
                match action.entity {
                    SelectedEntity::Monster(i) => {
                        if let Some(m) = state.data.monsters.get_mut(i) {
                            m.set_field(&action.field, action.old_value);
                        }
                    }
                    SelectedEntity::Npc(i) => {
                        if let Some(n) = state.data.npcs.get_mut(i) {
                            n.set_field(&action.field, action.old_value);
                        }
                    }
                    SelectedEntity::Extra(i) => {
                        if let Some(e) = state.data.extra_refs.get_mut(i) {
                            e.set_field(&action.field, action.old_value);
                        }
                    }
                }
                state.view.tile_layer_cache.clear();
                state.view.overlay_cache.clear();
                let still_dirty = !state.data.undo_stack.is_empty();
                set_tab_modified(app, tab_id, still_dirty);
            }
            Task::none()
        }

        MapEditorMessage::Redo(tab_id) => {
            let state = match app.state.map_editors.get_mut(&tab_id) {
                Some(s) => s,
                None => return Task::none(),
            };
            if let Some(action) = state.pop_redo() {
                match action.entity {
                    SelectedEntity::Monster(i) => {
                        if let Some(m) = state.data.monsters.get_mut(i) {
                            m.set_field(&action.field, action.new_value);
                        }
                    }
                    SelectedEntity::Npc(i) => {
                        if let Some(n) = state.data.npcs.get_mut(i) {
                            n.set_field(&action.field, action.new_value);
                        }
                    }
                    SelectedEntity::Extra(i) => {
                        if let Some(e) = state.data.extra_refs.get_mut(i) {
                            e.set_field(&action.field, action.new_value);
                        }
                    }
                }
                state.view.tile_layer_cache.clear();
                state.view.overlay_cache.clear();
                set_tab_modified(app, tab_id, true);
            }
            Task::none()
        }

        MapEditorMessage::SaveEntities(tab_id) => {
            let state = match app.state.map_editors.get_mut(&tab_id) {
                Some(s) => s,
                None => return Task::none(),
            };
            if state.data.is_saving {
                return Task::none();
            }
            state.data.is_saving = true;
            let monsters = state.data.monsters.clone();
            let npcs = state.data.npcs.clone();
            let extra_refs = state.data.extra_refs.clone();
            let monster_path = state.data.monster_ref_path.clone();
            let npc_path = state.data.npc_ref_path.clone();
            let extra_path = state.data.extra_ref_path.clone();

            Task::perform(
                async move {
                    let mut saved: Vec<String> = Vec::new();
                    let mut errors: Vec<String> = Vec::new();

                    macro_rules! save_type {
                        ($T:ty, $records:expr, $path:expr) => {
                            if let Some(p) = $path {
                                match <$T>::save_file($records, &p) {
                                    Ok(()) => saved.push(
                                        p.file_name()
                                            .map(|n| n.to_string_lossy().to_string())
                                            .unwrap_or_else(|| p.display().to_string()),
                                    ),
                                    Err(e) => errors.push(format!(
                                        "{}: {}",
                                        p.file_name()
                                            .map(|n| n.to_string_lossy().to_string())
                                            .unwrap_or_default(),
                                        e
                                    )),
                                }
                            }
                        };
                    }
                    save_type!(dispel_core::MonsterRef, &monsters, monster_path);
                    save_type!(dispel_core::NPC, &npcs, npc_path);
                    save_type!(dispel_core::ExtraRef, &extra_refs, extra_path);

                    if !errors.is_empty() {
                        Err(errors.join("; "))
                    } else if saved.is_empty() {
                        Err("No entity files found to save".to_string())
                    } else {
                        Ok(format!("Saved: {}", saved.join(", ")))
                    }
                },
                move |result| Message::map_editor(MapEditorMessage::SaveComplete(tab_id, result)),
            )
        }

        MapEditorMessage::SaveComplete(tab_id, result) => {
            let success = result.is_ok();
            if let Some(state) = app.state.map_editors.get_mut(&tab_id) {
                state.data.is_saving = false;
                match result {
                    Ok(msg) => {
                        state.data.dirty = false;
                        state.data.status_msg = Some(msg);
                    }
                    Err(e) => {
                        state.data.status_msg = Some(format!("Save failed: {e}"));
                    }
                }
            }
            if success {
                set_tab_modified(app, tab_id, false);
                dismiss_status_after(tab_id)
            } else {
                Task::none()
            }
        }

        MapEditorMessage::ExportImage(tab_id) => {
            let state = match app.state.map_editors.get_mut(&tab_id) {
                Some(s) => s,
                None => return Task::none(),
            };
            if state.data.is_exporting {
                return Task::none();
            }
            state.data.is_exporting = true;
            let state = &*state;
            let map_path = match &state.data.map_path {
                Some(p) => p.clone(),
                None => return Task::none(),
            };
            let gtl_path = match &state.data.gtl_path {
                Some(p) => p.clone(),
                None => map_path.with_extension("gtl"),
            };
            let btl_path = match &state.data.btl_path {
                Some(p) => p.clone(),
                None => map_path.with_extension("btl"),
            };
            let Some(map_handle) = state.map_data() else {
                return Task::none();
            };
            let map_data = map_handle.0.clone();
            let game_path = app.state.workspace.game_path.clone();

            Task::perform(
                async move {
                    let file_handle = rfd::AsyncFileDialog::new()
                        .set_title("Export map as PNG")
                        .add_filter("PNG Image", &["png"])
                        .set_file_name(
                            map_path
                                .file_stem()
                                .map(|s| format!("{}.png", s.to_string_lossy()))
                                .unwrap_or_else(|| "map.png".to_string())
                                .as_str(),
                        )
                        .save_file()
                        .await;

                    let Some(file_handle) = file_handle else {
                        return Ok("Export cancelled".to_string());
                    };
                    let output_path = file_handle.path().to_path_buf();

                    let gtl_tiles = dispel_core::map::tileset::extract(&gtl_path)
                        .map_err(|e| format!("GTL read failed: {e}"))?;
                    let btl_tiles = dispel_core::map::tileset::extract(&btl_path)
                        .map_err(|e| format!("BTL read failed: {e}"))?;

                    let file = std::fs::File::open(&map_path)
                        .map_err(|e| format!("Map open failed: {e}"))?;
                    let mut reader = std::io::BufReader::new(file);

                    let map_id = map_path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();

                    dispel_core::map::render::render_map(
                        dispel_core::map::render::MapRenderConfig {
                            reader: &mut reader,
                            output_path: &output_path,
                            data: &map_data,
                            occlusion: false,
                            gtl_tileset: &gtl_tiles,
                            btl_tileset: &btl_tiles,
                            map_id: &map_id,
                            game_path: game_path.as_deref(),
                        },
                    )
                    .map_err(|e| format!("Render failed: {e}"))?;

                    Ok(format!(
                        "Exported to {}",
                        output_path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| output_path.display().to_string())
                    ))
                },
                move |result| Message::map_editor(MapEditorMessage::ExportComplete(tab_id, result)),
            )
        }

        MapEditorMessage::ExportComplete(tab_id, result) => {
            let success = result.is_ok();
            if let Some(state) = app.state.map_editors.get_mut(&tab_id) {
                state.data.is_exporting = false;
                state.data.status_msg = Some(match result {
                    Ok(msg) => msg,
                    Err(e) => format!("Export failed: {e}"),
                });
            }
            if success {
                dismiss_status_after(tab_id)
            } else {
                Task::none()
            }
        }

        MapEditorMessage::ClearStatus(tab_id) => {
            if let Some(state) = app.state.map_editors.get_mut(&tab_id) {
                state.data.status_msg = None;
            }
            Task::none()
        }

        MapEditorMessage::SwitchViewMode(tab_id, mode) => {
            if let Some(state) = app.state.map_editors.get_mut(&tab_id) {
                state.view.view_mode = mode;
                state.view.selected_sprite_sequence = None;
            }
            Task::none()
        }

        MapEditorMessage::SelectSpriteSequence(tab_id, idx) => {
            if let Some(state) = app.state.map_editors.get_mut(&tab_id) {
                state.view.selected_sprite_sequence = idx;
            }
            Task::none()
        }

        MapEditorMessage::ShowSpriteExportDialog(tab_id) => {
            if let Some(state) = app.state.map_editors.get_mut(&tab_id) {
                state.data.sprite_export_dialog =
                    Some(crate::state::map_editor::SpriteExportDialogState::default());
            }
            Task::none()
        }

        MapEditorMessage::CloseSpriteExportDialog(tab_id) => {
            if let Some(state) = app.state.map_editors.get_mut(&tab_id) {
                state.data.sprite_export_dialog = None;
            }
            Task::none()
        }

        MapEditorMessage::ChooseSpriteExportDir(tab_id) => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .pick_folder()
                    .await
                    .map(|h| h.path().to_path_buf())
            },
            move |path| Message::map_editor(MapEditorMessage::SpriteExportDirChosen(tab_id, path)),
        ),

        MapEditorMessage::SpriteExportDirChosen(tab_id, path) => {
            if let Some(state) = app.state.map_editors.get_mut(&tab_id) {
                if let Some(ref mut dlg) = state.data.sprite_export_dialog {
                    dlg.export_dir = path;
                    dlg.status = crate::state::map_editor::SpriteExportStatus::Idle;
                }
            }
            Task::none()
        }

        MapEditorMessage::ConfirmSpriteExport(tab_id) => {
            let Some(state) = app.state.map_editors.get_mut(&tab_id) else {
                return Task::none();
            };
            let Some(ref dlg) = state.data.sprite_export_dialog else {
                return Task::none();
            };
            let Some(ref export_dir) = dlg.export_dir else {
                return Task::none();
            };
            let Some(ref map_path) = state.data.map_path else {
                return Task::none();
            };
            let map_path = map_path.clone();
            let export_dir = export_dir.clone();

            if let Some(ref mut dlg) = state.data.sprite_export_dialog {
                dlg.status = crate::state::map_editor::SpriteExportStatus::Exporting;
            }

            Task::perform(
                async move {
                    dispel_core::map::extract_sprites(&map_path, &export_dir)
                        .map(|()| format!("Sprites exported → {}", export_dir.display()))
                        .map_err(|e| e.to_string())
                },
                move |result| {
                    Message::map_editor(MapEditorMessage::SpriteExportDone(tab_id, result))
                },
            )
        }

        MapEditorMessage::SpriteExportDone(tab_id, result) => {
            if let Some(state) = app.state.map_editors.get_mut(&tab_id) {
                if let Some(ref mut dlg) = state.data.sprite_export_dialog {
                    dlg.status = match result {
                        Ok(msg) => crate::state::map_editor::SpriteExportStatus::Done(msg),
                        Err(e) => crate::state::map_editor::SpriteExportStatus::Error(e),
                    };
                }
            }
            Task::none()
        }

        MapEditorMessage::FitToWindow(tab_id) => {
            if let Some(state) = app.state.map_editors.get_mut(&tab_id) {
                // Extract the map geometry before mutating view state to satisfy
                // the borrow checker (map_data() borrows state via loading_state).
                let fit = state.map_data().map(|h| {
                    let model = &h.0.model;
                    let diagonal = model.tiled_map_width + model.tiled_map_height;
                    let map_px_w = diagonal as f32 * 32.0;
                    let map_px_h = diagonal as f32 * 16.0;
                    let (cx, cy) = dispel_core::map::types::convert_map_coords_to_image_coords(
                        model.tiled_map_width / 2,
                        model.tiled_map_height / 2,
                        diagonal,
                    );
                    (map_px_w, map_px_h, cx as f32, cy as f32)
                });
                if let Some((map_px_w, map_px_h, center_px, center_py)) = fit {
                    let vp_w = state.view.last_canvas_w;
                    let vp_h = state.view.last_canvas_h;
                    // Choose zoom so the whole map fits, capped at 1:1.
                    let zoom = (vp_w / map_px_w).min(vp_h / map_px_h).clamp(0.05, 1.0);
                    state.view.zoom = zoom;
                    state.view.pan_x = vp_w / 2.0 - center_px * zoom;
                    state.view.pan_y = vp_h / 2.0 - center_py * zoom;
                } else {
                    state.view.zoom = 1.0;
                    state.view.pan_x = 0.0;
                    state.view.pan_y = 0.0;
                }
                state.view.tile_layer_cache.clear();
                state.view.overlay_cache.clear();
            }
            Task::none()
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Mark the workspace tab as modified/clean.
fn set_tab_modified(app: &mut App, tab_id: usize, modified: bool) {
    if let Some(tab) = app.state.workspace.tabs.iter_mut().find(|t| t.id == tab_id) {
        tab.modified = modified;
    }
}

/// Emit a delayed `ClearStatus` message to auto-dismiss the toolbar status text.
fn dismiss_status_after(tab_id: usize) -> Task<Message> {
    Task::perform(
        async move {
            tokio::time::sleep(std::time::Duration::from_secs(STATUS_DISMISS_SECS)).await;
        },
        move |()| Message::map_editor(MapEditorMessage::ClearStatus(tab_id)),
    )
}

// ── Internal sprite decoding ──────────────────────────────────────────────────

fn decode_internal_map_sprites(
    reader: &mut std::io::BufReader<std::fs::File>,
    map_data: &dispel_core::map::MapData,
) -> Vec<DecodedMapSprite> {
    use std::io::{Read, Seek, SeekFrom};

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
        if reader
            .seek(SeekFrom::Start(frame.image_start_position))
            .is_err()
        {
            continue;
        }

        let w = frame.width as u32;
        let h = frame.height as u32;
        let pixel_count = (w * h) as usize;
        let mut raw = vec![0u8; pixel_count * 2];
        if reader.read_exact(&mut raw).is_err() {
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

        result.push(DecodedMapSprite {
            x: block.sprite_x,
            y: block.sprite_y,
            bottom_right_y: block.sprite_bottom_right_y,
            pixels,
            width: w,
            height: h,
        });
    }

    result
}

// ── Entity loading ────────────────────────────────────────────────────────────

/// Load entity .ref files for the given map using `Ref/Map.ini` for discovery.
///
/// Matches the map_ini entry by checking whether each entry's entity filenames
/// *contain* the map stem — e.g. "npccat1.ref" contains "cat1".  This mirrors
/// the strategy used by `render.rs::plot_external_entities`.
pub fn load_entities(
    map_path: &std::path::Path,
    game_path: Option<std::path::PathBuf>,
) -> EntityBundle {
    use dispel_core::references::extra_ini::Extra;
    use dispel_core::references::monster_ini::MonsterIni;
    use dispel_core::references::npc_ini::NpcIni;
    use dispel_core::{ExtraRef, MonsterRef, NPC};
    use std::collections::HashMap;
    use std::path::PathBuf;

    let stem = map_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let game_path = match game_path {
        Some(p) => p,
        None => return EntityBundle::default(),
    };

    let map_ini_path = game_path.join("Ref").join("Map.ini");
    if !map_ini_path.exists() {
        return EntityBundle::default();
    }

    let map_inis = match dispel_core::references::map_ini::read_map_ini(&map_ini_path) {
        Ok(v) => v,
        Err(_) => return EntityBundle::default(),
    };

    let map_ini = map_inis.into_iter().find(|ini| {
        ini.monsters_filename
            .as_ref()
            .is_some_and(|m| m.to_lowercase().contains(stem.as_str()))
            || ini
                .npc_filename
                .as_ref()
                .is_some_and(|n| n.to_lowercase().contains(stem.as_str()))
            || ini
                .extra_filename
                .as_ref()
                .is_some_and(|e| e.to_lowercase().contains(stem.as_str()))
    });

    let Some(map_ini) = map_ini else {
        return EntityBundle::default();
    };

    // Case-insensitive file resolution: try original → uppercase → lowercase.
    let resolve = |sub_dir: &str, filename: &str| -> Option<PathBuf> {
        for name in &[
            filename.to_string(),
            filename.to_ascii_uppercase(),
            filename.to_ascii_lowercase(),
        ] {
            let p = game_path.join(sub_dir).join(name);
            if p.exists() {
                return Some(p);
            }
        }
        None
    };

    // Build id→sprite_filename lookups from .ini files.
    let monster_id_to_sprite: HashMap<i32, String> =
        MonsterIni::read_file(&game_path.join("Monster.ini"))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|m| m.sprite_filename.map(|s| (m.id, s)))
            .collect();
    let npc_id_to_sprite: HashMap<i32, String> = NpcIni::read_file(&game_path.join("Npc.ini"))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|n| n.sprite_filename.map(|s| (n.id, s)))
        .collect();
    let extra_id_to_sprite: HashMap<i32, String> = Extra::read_file(&game_path.join("Extra.ini"))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|e| e.sprite_filename.map(|s| (e.id, s)))
        .collect();

    // Frame cache: avoid re-reading the same sprite file.
    type FrameCache =
        HashMap<PathBuf, Option<Vec<dispel_core::map::sprite_loader::LoadedSpriteFrame>>>;
    let mut sprite_cache: FrameCache = HashMap::new();

    // ── Monsters ──────────────────────────────────────────────────────────────
    let (monsters, monster_sprite_handles, monster_ref_path) = load_ref_file(
        map_ini.monsters_filename,
        "MonsterInGame",
        &resolve,
        |m: &MonsterRef| (m.mon_id, 3, false),
        &monster_id_to_sprite,
        &mut sprite_cache,
    );

    // ── NPCs ──────────────────────────────────────────────────────────────────
    let (npcs, npc_sprite_handles, npc_ref_path) = load_ref_file(
        map_ini.npc_filename,
        "NpcInGame",
        &resolve,
        |n: &NPC| {
            let dir = i32::from(n.looking_direction);
            let (seq, flip) = if dir > 4 {
                ((8 - dir) as usize, true)
            } else {
                (dir as usize, false)
            };
            (n.npc_id, seq, flip)
        },
        &npc_id_to_sprite,
        &mut sprite_cache,
    );

    // ── Extra refs ────────────────────────────────────────────────────────────
    let (extra_refs, extra_sprite_handles, extra_ref_path) = load_ref_file(
        map_ini.extra_filename,
        "ExtraInGame",
        &resolve,
        |e: &ExtraRef| {
            let rotation = e.rotation as usize;
            let obj_type = u8::from(e.object_type) as usize;
            let seq = if obj_type == 0 {
                2 * e.closed as usize + rotation
            } else {
                rotation
            };
            (e.ext_id as i32, seq, false)
        },
        &extra_id_to_sprite,
        &mut sprite_cache,
    );

    EntityBundle {
        monsters,
        npcs,
        extra_refs,
        monster_sprites: monster_sprite_handles,
        npc_sprites: npc_sprite_handles,
        extra_sprites: extra_sprite_handles,
        monster_ref_path,
        npc_ref_path,
        extra_ref_path,
    }
}

/// Shared loader for one entity type: reads the .ref file, resolves per-entity
/// sprites, and returns (entities, sprite_handles, ref_path).
///
/// `get_id_seq_flip` derives `(sprite_lookup_id, frame_seq, flip)` for each entity.
fn load_ref_file<T: Extractor>(
    filename: Option<String>,
    subdir: &str,
    resolve: &impl Fn(&str, &str) -> Option<std::path::PathBuf>,
    get_id_seq_flip: impl Fn(&T) -> (i32, usize, bool),
    id_to_sprite: &std::collections::HashMap<i32, String>,
    sprite_cache: &mut std::collections::HashMap<
        std::path::PathBuf,
        Option<Vec<dispel_core::map::sprite_loader::LoadedSpriteFrame>>,
    >,
) -> (
    Vec<T>,
    Vec<Option<DecodedEntitySprite>>,
    Option<std::path::PathBuf>,
) {
    use dispel_core::map::sprite_loader::load_sprite_frames;

    let Some(f) = filename else {
        return (Vec::new(), Vec::new(), None);
    };
    let Some(p) = resolve(subdir, &f) else {
        return (Vec::new(), Vec::new(), None);
    };
    let Ok(data) = T::read_file(&p) else {
        return (Vec::new(), Vec::new(), None);
    };

    let ref_path = Some(p.clone());
    let sprites: Vec<Option<DecodedEntitySprite>> = data
        .iter()
        .map(|entity| {
            let (id, seq, flip) = get_id_seq_flip(entity);
            id_to_sprite
                .get(&id)
                .and_then(|spr_name| resolve(subdir, spr_name))
                .and_then(|spr_path| {
                    let frames = sprite_cache
                        .entry(spr_path.clone())
                        .or_insert_with(|| load_sprite_frames(&spr_path));
                    frames
                        .as_ref()
                        .and_then(|fs| fs.get(seq).or_else(|| fs.first()))
                        .map(|frame| decoded_from_frame(frame, flip))
                })
        })
        .collect();

    (data, sprites, ref_path)
}

/// Convert a `LoadedSpriteFrame` to `DecodedEntitySprite`.
fn decoded_from_frame(
    frame: &dispel_core::map::sprite_loader::LoadedSpriteFrame,
    flip: bool,
) -> DecodedEntitySprite {
    let w = frame.image.width();
    let h = frame.image.height();
    DecodedEntitySprite {
        pixels: frame.image.as_raw().to_vec(),
        width: w,
        height: h,
        origin_x: frame.origin_x,
        origin_y: frame.origin_y,
        flip,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_load_entities() {
        let game_path = Path::new("../fixtures/Dispel");
        if !game_path.exists() {
            println!("Skipping test_load_entities: fixtures/Dispel path does not exist");
            return;
        }

        // cat1 = Palace of Aesh: NPCs only (24), no monsters, no extras
        let cat1_entities = load_entities(
            &game_path.join("Map/cat1.map"),
            Some(game_path.to_path_buf()),
        );
        assert_eq!(cat1_entities.monsters.len(), 0, "cat1 has no monsters");
        assert_eq!(cat1_entities.npcs.len(), 24, "cat1 has 24 NPCs");
        assert_eq!(cat1_entities.extra_refs.len(), 0, "cat1 has no extra refs");
        assert_eq!(
            cat1_entities.npc_sprites.len(),
            24,
            "each NPC has a sprite slot"
        );

        // map1 = Aesh overworld: monsters + NPCs + extras
        let map1_entities = load_entities(
            &game_path.join("Map/map1.map"),
            Some(game_path.to_path_buf()),
        );
        assert!(
            map1_entities.monsters.len() > 0,
            "map1 should have monsters"
        );
        assert!(map1_entities.npcs.len() > 0, "map1 should have NPCs");
        assert!(
            map1_entities.extra_refs.len() > 0,
            "map1 should have extra refs"
        );
        assert_eq!(
            map1_entities.monster_sprites.len(),
            map1_entities.monsters.len(),
            "sprite vec parallel to monsters"
        );
    }
}
