// ── MapCanvasTilesLayer — renders GTL/BTL tiles + interlaced objects ─────────

use super::draw_helpers::{diamond_path, draw_entity_sprite, draw_tile_layer};
use super::geometry::{is_visible, tile_center, tile_to_screen};
use super::input::MapCanvas;
use super::state::MapCanvasState;
use super::{TILE_H, TILE_W};
use crate::editors::map_editor::canvas::hit_test::npc_pos;
use crate::editors::map_editor::state::MapEditorState;
use crate::message::Message;
use iced::advanced::image::Image as CoreImage;
use iced::widget::canvas::{self, Action, Geometry};
use iced::{mouse, Color, Event, Point, Rectangle, Size};

/// Canvas Program for tile layers only (GTL, BTL, internal sprites).
/// Images always draw on top of primitives within a single canvas,
/// so we split into two canvases: tiles first, then overlays.
pub struct MapCanvasTilesLayer<'a> {
    pub state: &'a MapEditorState,
    pub tab_id: usize,
}

impl<'a> canvas::Program<Message> for MapCanvasTilesLayer<'a> {
    type State = MapCanvasState;

    fn update(
        &self,
        interaction: &mut MapCanvasState,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        // Delegate to MapCanvas's update logic.
        MapCanvas {
            state: self.state,
            tab_id: self.tab_id,
        }
        .update(interaction, event, bounds, cursor)
    }

    fn draw(
        &self,
        _interaction: &MapCanvasState,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        // Use the per-tab cache so cursor moves (which only touch the overlay canvas)
        // don't trigger an expensive tile-layer redraw.  The cache is cleared by the
        // update handler whenever pan, zoom, tiles, or entity sprites change.
        let geometry = self
            .state
            .view
            .tile_layer_cache
            .draw(renderer, bounds.size(), |frame| {
                // Fill background
                frame.fill_rectangle(
                    Point::ORIGIN,
                    bounds.size(),
                    Color::from_rgb(0.1, 0.1, 0.12),
                );

                let Some(map_handle) = self.state.map_data() else {
                    return; // cache closure returns ()
                };
                let map_data = &map_handle.0;
                let model = &map_data.model;
                let diagonal = model.tiled_map_width + model.tiled_map_height;

                let pan_x = self.state.view.pan_x;
                let pan_y = self.state.view.pan_y;
                let zoom = self.state.view.zoom;

                // Draw ground layer
                if self.state.view.show_ground && self.state.data.tiles_ready {
                    draw_tile_layer(
                        frame,
                        &map_data.gtl_tiles,
                        &self.state.data.gtl_handles,
                        diagonal,
                        pan_x,
                        pan_y,
                        zoom,
                        bounds,
                    );
                }

                // ── Interlaced object pass ────────────────────────────────────
                // All depth-relevant objects (buildings, internal sprites,
                // monsters, NPCs, extras) are collected into one list, sorted by
                // their Y-depth key, then rendered together — matching the
                // DispelTools IInterlacedOrderObject / IInterlacedOrderObjectComparer
                // approach. TypeOrder breaks ties: buildings < sprites < entities.
                if self.state.data.tiles_ready {
                    let nox = model.map_non_occluded_start_x;
                    let noy = model.map_non_occluded_start_y;
                    let nox_f = nox as f32;
                    let noy_f = noy as f32;

                    // Render item tags (no heap data, just indices).
                    enum Item {
                        TiledObject(usize),
                        Sprite(usize),
                        Monster(usize),
                        Npc(usize),
                        Extra(usize),
                    }

                    let mut items: Vec<(i32, i32, i32, Item)> = Vec::new();

                    if self.state.view.show_buildings {
                        for (i, info) in map_data.tiled_infos.iter().enumerate() {
                            let pos = info.y + info.ids.len() as i32 * TILE_H as i32;
                            items.push((pos, 0, i as i32, Item::TiledObject(i)));
                        }
                    }

                    if self.state.view.show_internal_sprites {
                        for (i, spr) in self.state.data.internal_sprite_handles.iter().enumerate() {
                            items.push((spr.sort_y, 1, 0, Item::Sprite(i)));
                        }
                    }

                    // External entity sort key: tile bottom-centre in occluded pixel space.
                    // Matches MapExternalObject.PositionOrder =
                    //   (-X+Y)*16 + mapPixelHeight/2 - mapNonOccludedStartY + 16
                    //   = convert_y(X,Y,diagonal) + 32 - noy
                    let entity_pos = |tx: i32, ty: i32| -> i32 {
                        let img_y = dispel_core::map::types::convert_map_coords_to_image_coords(
                            tx, ty, diagonal,
                        )
                        .1;
                        img_y + 32 - noy
                    };

                    if self.state.view.show_monsters {
                        for (i, m) in self.state.data.monsters.iter().enumerate() {
                            items.push((
                                entity_pos(m.pos_x, m.pos_y),
                                2,
                                m.pos_x,
                                Item::Monster(i),
                            ));
                        }
                    }

                    if self.state.view.show_npcs {
                        for (i, n) in self.state.data.npcs.iter().enumerate() {
                            let (nx, ny) = npc_pos(n);
                            items.push((entity_pos(nx, ny), 2, nx, Item::Npc(i)));
                        }
                    }

                    if self.state.view.show_objects {
                        for (i, e) in self.state.data.extra_refs.iter().enumerate() {
                            items.push((entity_pos(e.x_pos, e.y_pos), 2, e.x_pos, Item::Extra(i)));
                        }
                    }

                    items.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));

                    for (_, _, _, item) in &items {
                        match item {
                            Item::TiledObject(obj_i) => {
                                let info = &map_data.tiled_infos[*obj_i];
                                let base_x = (info.x as f32 + nox_f) * zoom + pan_x;
                                let base_y = (info.y as f32 + noy_f) * zoom + pan_y;
                                let w = TILE_W * zoom;
                                let h = TILE_H * zoom;
                                for (i, &btl_id) in info.ids.iter().enumerate() {
                                    if btl_id <= 0 {
                                        continue;
                                    }
                                    let handle_id = btl_id.unsigned_abs() as i32;
                                    let Some(handle) = self.state.data.btl_handles.get(&handle_id)
                                    else {
                                        continue;
                                    };
                                    let px = base_x;
                                    let py = base_y + i as f32 * h;
                                    if !is_visible(px, py, w, h, bounds) {
                                        continue;
                                    }
                                    frame.draw_image(
                                        Rectangle::new(Point::new(px, py), Size::new(w, h)),
                                        CoreImage::new(handle.clone()),
                                    );
                                }
                            }
                            Item::Sprite(i) => {
                                let spr = &self.state.data.internal_sprite_handles[*i];
                                let sx = spr.x as f32 * zoom + pan_x;
                                let sy = spr.y as f32 * zoom + pan_y;
                                let sw = spr.width as f32 * zoom;
                                let sh = spr.height as f32 * zoom;
                                if is_visible(sx, sy, sw, sh, bounds) {
                                    frame.draw_image(
                                        Rectangle::new(Point::new(sx, sy), Size::new(sw, sh)),
                                        CoreImage::new(spr.handle.clone()),
                                    );
                                }
                            }
                            Item::Monster(i) => {
                                let monster = &self.state.data.monsters[*i];
                                let (px, py) = tile_to_screen(
                                    monster.pos_x,
                                    monster.pos_y,
                                    diagonal,
                                    pan_x,
                                    pan_y,
                                    zoom,
                                );
                                if is_visible(px, py, TILE_W * zoom, TILE_H * zoom, bounds) {
                                    let (tile_cx, tile_cy) = tile_center(px, py, zoom);
                                    if let Some(Some(spr)) = self.state.data.monster_sprites.get(*i)
                                    {
                                        draw_entity_sprite(frame, spr, tile_cx, tile_cy, zoom);
                                    } else {
                                        let r = 4.0 * zoom;
                                        frame.fill(
                                            &diamond_path(tile_cx, tile_cy, r),
                                            Color::from_rgba(0.9, 0.15, 0.15, 0.85),
                                        );
                                    }
                                }
                            }
                            Item::Npc(i) => {
                                let npc = &self.state.data.npcs[*i];
                                let (nx, ny) = npc_pos(npc);
                                let (px, py) = tile_to_screen(nx, ny, diagonal, pan_x, pan_y, zoom);
                                if is_visible(px, py, TILE_W * zoom, TILE_H * zoom, bounds) {
                                    let (tile_cx, tile_cy) = tile_center(px, py, zoom);
                                    if let Some(Some(spr)) = self.state.data.npc_sprites.get(*i) {
                                        draw_entity_sprite(frame, spr, tile_cx, tile_cy, zoom);
                                    } else {
                                        let r = 3.5 * zoom;
                                        frame.fill(
                                            &canvas::Path::circle(Point::new(tile_cx, tile_cy), r),
                                            Color::from_rgba(0.15, 0.45, 0.9, 0.85),
                                        );
                                    }
                                }
                            }
                            Item::Extra(i) => {
                                let extra = &self.state.data.extra_refs[*i];
                                let (px, py) = tile_to_screen(
                                    extra.x_pos,
                                    extra.y_pos,
                                    diagonal,
                                    pan_x,
                                    pan_y,
                                    zoom,
                                );
                                if is_visible(px, py, TILE_W * zoom, TILE_H * zoom, bounds) {
                                    let (tile_cx, tile_cy) = tile_center(px, py, zoom);
                                    if let Some(Some(spr)) = self.state.data.extra_sprites.get(*i) {
                                        draw_entity_sprite(frame, spr, tile_cx, tile_cy, zoom);
                                    } else {
                                        let s = 5.0 * zoom;
                                        frame.fill_rectangle(
                                            Point::new(tile_cx - s * 0.5, tile_cy - s * 0.5),
                                            Size::new(s, s),
                                            Color::from_rgba(0.95, 0.85, 0.1, 0.85),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }

                // Draw flat BTL roof layer (top, after all depth-sorted objects)
                if self.state.view.show_roofs && self.state.data.tiles_ready {
                    draw_tile_layer(
                        frame,
                        &map_data.btl_tiles,
                        &self.state.data.btl_handles,
                        diagonal,
                        pan_x,
                        pan_y,
                        zoom,
                        bounds,
                    );
                }
            }); // end cache closure

        vec![geometry]
    }

    fn mouse_interaction(
        &self,
        _state: &MapCanvasState,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::Idle
        }
    }
}
