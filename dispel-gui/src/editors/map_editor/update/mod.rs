use crate::app::App;
use crate::editors::map_editor::canvas::{find_hovered_element, find_tile_at};
use crate::editors::map_editor::{MapEditorMessage, MapLayer, MapTool, SelectedEntity};
use crate::message::{Message, MessageExt};
use iced::Task;

mod conversation;
mod dialog;
mod entity;
mod map;
pub use map::resolve_map_filename;
mod persistence;
mod sprite_export;

/// Duration before a status message is automatically cleared.
const STATUS_DISMISS_SECS: u64 = 3;

pub fn handle(message: MapEditorMessage, app: &mut App) -> Task<Message> {
    match message {
        // ── Delegated to submodules ───────────────────────────────────────────
        MapEditorMessage::Open(tab_id, path) => map::open(app, tab_id, path),
        MapEditorMessage::MapLoaded(tab_id, result) => map::map_loaded(app, tab_id, result),
        MapEditorMessage::TilesDecoded(tab_id, result) => map::tiles_decoded(app, tab_id, result),
        MapEditorMessage::EntitiesLoaded(tab_id, bundle) => {
            map::entities_loaded(app, tab_id, bundle)
        }
        MapEditorMessage::EntityFieldChanged(tab_id, entity, field, value) => {
            entity::field_changed(app, tab_id, entity, field, value)
        }
        MapEditorMessage::Undo(tab_id) => entity::undo(app, tab_id),
        MapEditorMessage::Redo(tab_id) => entity::redo(app, tab_id),
        MapEditorMessage::SaveEntities(tab_id) => persistence::save_entities(app, tab_id),
        MapEditorMessage::SaveMap(tab_id) => persistence::save_map(app, tab_id),
        MapEditorMessage::SaveComplete(tab_id, result) => {
            persistence::save_complete(app, tab_id, result)
        }
        MapEditorMessage::MapSaved(tab_id, result) => persistence::map_saved(app, tab_id, result),
        MapEditorMessage::ExportImage(tab_id) => persistence::export_image(app, tab_id),
        MapEditorMessage::ExportComplete(tab_id, result) => {
            persistence::export_complete(app, tab_id, result)
        }

        MapEditorMessage::ExportTmx(tab_id) => {
            let state = app.state.editors.map_editors.get(&tab_id);
            let Some(editor) = state else {
                return Task::none();
            };

            // Get map data and paths from the editor state
            let map_data_opt = editor.map_data().map(|h| h.0.clone());
            let map_path = match &editor.data.map_path {
                Some(p) => p.clone(),
                None => {
                    if let Some(editor) = app.state.editors.map_editors.get_mut(&tab_id) {
                        editor.data.status_msg = Some("Map file path unknown".into());
                    }
                    return Task::none();
                }
            };

            let Some(map_data) = map_data_opt else {
                if let Some(editor) = app.state.editors.map_editors.get_mut(&tab_id) {
                    editor.data.status_msg = Some("Map not loaded".into());
                }
                return Task::none();
            };

            Task::perform(
                async move {
                    let handle = rfd::AsyncFileDialog::new()
                        .set_title("Choose TMX export directory")
                        .pick_folder()
                        .await;

                    let Some(folder) = handle else {
                        return Err("Export cancelled".into());
                    };
                    let out_dir = folder.path().to_path_buf();

                    // Determine tileset paths: same dir as map, same stem
                    let gtl_path = map_path.with_extension("gtl");
                    let btl_path = map_path.with_extension("btl");

                    // Create output dir
                    std::fs::create_dir_all(&out_dir)
                        .map_err(|e| format!("Failed to create output directory: {e}"))?;

                    // Export TMX
                    dispel_core::map::tmx::export_tmx(&map_data, &gtl_path, &btl_path, &out_dir)
                        .map_err(|e| format!("TMX export failed: {e}"))?;

                    Ok(out_dir.to_string_lossy().to_string())
                },
                move |r| {
                    crate::message::Message::map_editor(
                        crate::editors::map_editor::MapEditorMessage::TmxExportComplete(tab_id, r),
                    )
                },
            )
        }

        MapEditorMessage::TmxExportComplete(tab_id, result) => {
            if let Some(editor) = app.state.editors.map_editors.get_mut(&tab_id) {
                match result {
                    Ok(path) => {
                        editor.data.status_msg = Some(format!("TMX exported to: {}", path));
                    }
                    Err(e) => {
                        editor.data.status_msg = Some(format!("TMX export error: {}", e));
                    }
                }
            }
            Task::none()
        }

        MapEditorMessage::ShowDialogPreview(tab_id, npc_idx) => {
            dialog::show_preview(app, tab_id, npc_idx)
        }
        MapEditorMessage::DialogPreviewLoaded(tab_id, result) => {
            dialog::preview_loaded(app, tab_id, result)
        }
        MapEditorMessage::HideDialogPreview(tab_id) => dialog::hide_preview(app, tab_id),
        MapEditorMessage::StartConversation(tab_id, npc_idx) => {
            conversation::start(app, tab_id, npc_idx)
        }
        MapEditorMessage::ConversationLoaded(tab_id, result) => {
            conversation::loaded(app, tab_id, result)
        }
        MapEditorMessage::AdvanceConversation(tab_id) => conversation::advance(app, tab_id),
        MapEditorMessage::SelectChoice(tab_id, idx) => {
            conversation::select_choice(app, tab_id, idx)
        }
        MapEditorMessage::ResetConversation(tab_id) => conversation::reset(app, tab_id),
        MapEditorMessage::CloseConversation(tab_id) => conversation::close(app, tab_id),
        MapEditorMessage::ShowSpriteExportDialog(tab_id) => sprite_export::show_dialog(app, tab_id),
        MapEditorMessage::CloseSpriteExportDialog(tab_id) => {
            sprite_export::close_dialog(app, tab_id)
        }
        MapEditorMessage::ChooseSpriteExportDir(tab_id) => sprite_export::choose_dir(tab_id),
        MapEditorMessage::SpriteExportDirChosen(tab_id, path) => {
            sprite_export::dir_chosen(app, tab_id, path)
        }
        MapEditorMessage::ConfirmSpriteExport(tab_id) => sprite_export::confirm_export(app, tab_id),
        MapEditorMessage::SpriteExportDone(tab_id, result) => {
            sprite_export::export_done(app, tab_id, result)
        }

        // ── Inline handlers ───────────────────────────────────────────────────
        MapEditorMessage::PanChanged(tab_id, dx, dy) => {
            if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
                state.view.pan_x += dx;
                state.view.pan_y += dy;
                state.view.tile_layer_cache.clear();
                state.view.overlay_cache.clear();
            }
            Task::none()
        }

        MapEditorMessage::ZoomChanged(tab_id, factor, cx, cy) => {
            if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
                let old_zoom = state.view.zoom;
                let new_zoom = (old_zoom * factor).clamp(0.05, 8.0);
                let ratio = new_zoom / old_zoom;
                // cx/cy are NaN when triggered from a toolbar button (no cursor
                // position).
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
            if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
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
                    MapLayer::NpcWaypoints => {
                        state.view.show_npc_waypoints = !state.view.show_npc_waypoints
                    }
                    MapLayer::Objects => state.view.show_objects = !state.view.show_objects,
                    MapLayer::DrawItems => state.view.show_draw_items = !state.view.show_draw_items,
                    MapLayer::ObjectIds => state.view.show_object_ids = !state.view.show_object_ids,
                }
                // Hiding the layer that owns the active editing tool makes that
                // tool meaningless — reset to Pan.
                if let Some(owner) = state.view.active_tool.owning_layer()
                    && owner == layer
                    && !map::layer_visible(&state.view, layer)
                {
                    state.view.active_tool = MapTool::Pan;
                }
                // Tile canvas renders entities and tile layers; overlay renders
                // collisions and events — clear both caches.
                state.view.tile_layer_cache.clear();
                state.view.overlay_cache.clear();
            }
            Task::none()
        }

        MapEditorMessage::MouseMoved(tab_id, x, y, canvas_w, canvas_h) => {
            if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
                state.view.cursor_canvas_x = x;
                state.view.cursor_canvas_y = y;
                if canvas_w > 0.0 && canvas_h > 0.0 {
                    state.view.last_canvas_w = canvas_w;
                    state.view.last_canvas_h = canvas_h;
                }
                // Intentionally NOT clearing tile_layer_cache or overlay_cache
                // here: cursor moves are high-frequency and only affect the
                // cursor-dependent part of the overlay (tile highlight, hover
                // ring, coord label), which is drawn uncached on every frame
                // anyway.
            }
            Task::none()
        }

        MapEditorMessage::Deselect(tab_id) => {
            if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
                state.view.selected_entity = None;
                state.view.overlay_cache.clear();
            }
            Task::none()
        }

        MapEditorMessage::CanvasClicked(tab_id, cx, cy) => {
            if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
                // Tool routing comes FIRST: the active tool decides what a
                // click means. Layer visibility no longer gates editing.
                match state.view.active_tool {
                    MapTool::Pan => {
                        // Selection only — never mutates.
                        let clicked = find_hovered_element(state, cx, cy);
                        state.data.status_msg = Some(match &clicked {
                            Some(SelectedEntity::CollisionTile(tx, ty)) => {
                                format!("Collision tile ({},{}) detected!", tx, ty)
                            }
                            Some(SelectedEntity::EventTile(tx, ty)) => {
                                format!("Event tile ({},{}) detected!", tx, ty)
                            }
                            Some(SelectedEntity::ObjectIdTile(tx, ty)) => {
                                format!("Object ID tile ({},{}) detected!", tx, ty)
                            }
                            Some(SelectedEntity::Monster(i)) => format!("Monster {} detected", i),
                            Some(SelectedEntity::Npc(i)) => format!("NPC {} detected", i),
                            Some(SelectedEntity::Extra(i)) => format!("Extra {} detected", i),
                            Some(SelectedEntity::DrawItem(i)) => {
                                format!("Draw item {} detected", i)
                            }
                            None => {
                                format!("Clicked at ({:.0},{:.0}) — no tile detected", cx, cy)
                            }
                        });
                        state.view.selected_entity = clicked;
                        state.view.overlay_cache.clear();
                    }
                    MapTool::Collision => {
                        if let Some((tx, ty)) = find_tile_at(state, cx, cy)
                            && map::toggle_collision_at(state, tx, ty)
                        {
                            set_tab_modified(app, tab_id, true);
                        }
                    }
                    MapTool::ObjectId => {
                        if let Some((tx, ty)) = find_tile_at(state, cx, cy)
                            && map::apply_object_id_edit(state, tx, ty)
                        {
                            set_tab_modified(app, tab_id, true);
                        }
                    }
                    MapTool::EventInspect => {
                        if let Some((tx, ty)) = find_tile_at(state, cx, cy) {
                            state.data.status_msg =
                                Some(format!("Event tile ({},{}) — inspecting", tx, ty));
                            state.view.selected_entity = Some(SelectedEntity::EventTile(tx, ty));
                            state.view.overlay_cache.clear();
                        }
                    }
                }
            }
            Task::none()
        }

        MapEditorMessage::ClearStatus(tab_id) => {
            if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
                state.data.status_msg = None;
            }
            Task::none()
        }

        MapEditorMessage::SwitchViewMode(tab_id, mode) => {
            if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
                state.view.view_mode = mode;
                state.view.selected_sprite_sequence = None;
            }
            Task::none()
        }

        MapEditorMessage::SelectSpriteSequence(tab_id, idx) => {
            if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
                state.view.selected_sprite_sequence = idx;
            }
            Task::none()
        }

        MapEditorMessage::FitToWindow(tab_id) => {
            if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
                // Extract the map geometry before mutating view state to satisfy
                // the borrow checker (map_data() borrows state via
                // loading_state).
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
                    // Choose zoom so the full map width or height fits, capped at
                    // 1.0 (no zoom-in).
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

        MapEditorMessage::SelectTool(tab_id, tool) => map::select_tool(app, tab_id, tool),

        MapEditorMessage::SetObjectBrushMode(tab_id, mode) => {
            if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
                state.view.object_brush_mode = mode;
            }
            Task::none()
        }

        MapEditorMessage::SetObjectBrush(tab_id, value) => {
            if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
                state.data.object_brush = value.clamp(1, 511);
            }
            Task::none()
        }
    }
}

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
