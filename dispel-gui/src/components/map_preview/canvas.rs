//! Read-only map preview canvas — renders tiles, buildings, sprites, and entity
//! markers from save file data.
//!
//! Reuses isometric rendering math from `map_editor/canvas/` but inlined here
//! to avoid extracting shared utilities (oracle recommendation — see map-preview.md).

use crate::components::map_preview::state::{EntityKind, MapPreviewState, PreviewEntity};
use crate::components::map_preview::PreviewMessage;
use crate::message::Message;
use iced::advanced::image::Image as CoreImage;
use iced::widget::canvas::{self, Frame, Geometry};
use iced::{mouse, Color, Event, Point, Rectangle, Size};

// ── Isometric tile constants (shared with map_editor) ─────────────────────────

/// Rendered width of one isometric tile in world pixels.
pub(crate) const TILE_W: f32 = 62.0;
/// Rendered height of one isometric tile in world pixels.
pub(crate) const TILE_H: f32 = 32.0;

// ── Canvas interaction state ──────────────────────────────────────────────────

/// Per-canvas interaction state (managed by Iced).
#[derive(Default)]
pub struct PreviewCanvasState {
    pub is_dragging: bool,
    pub drag_last: Option<Point>,
    pub drag_start: Option<Point>,
}

// ── Canvas Program ────────────────────────────────────────────────────────────

/// Canvas program for the read-only map preview.
pub struct MapPreviewCanvas<'a> {
    pub state: &'a MapPreviewState,
}

impl<'a> canvas::Program<Message> for MapPreviewCanvas<'a> {
    type State = PreviewCanvasState;

    fn update(
        &self,
        interaction: &mut PreviewCanvasState,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        use mouse::{Button, Event as MouseEvent, ScrollDelta};

        match event {
            Event::Mouse(MouseEvent::ButtonPressed(Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    interaction.is_dragging = true;
                    interaction.drag_last = Some(pos);
                    interaction.drag_start = Some(pos);
                    return Some(canvas::Action::capture());
                }
            }
            Event::Mouse(MouseEvent::ButtonReleased(Button::Left)) => {
                interaction.is_dragging = false;
                interaction.drag_last = None;
                interaction.drag_start = None;
            }
            Event::Mouse(MouseEvent::CursorMoved { .. }) => {
                if interaction.is_dragging {
                    if let Some(last) = interaction.drag_last {
                        if let Some(pos) = cursor.position_in(bounds) {
                            let dx = pos.x - last.x;
                            let dy = pos.y - last.y;
                            interaction.drag_last = Some(pos);
                            return Some(
                                canvas::Action::publish(Message::MapPreview(
                                    PreviewMessage::Pan(dx, dy),
                                ))
                                .and_capture(),
                            );
                        }
                    }
                }
            }
            Event::Mouse(MouseEvent::CursorLeft) => {}
            Event::Mouse(MouseEvent::WheelScrolled { delta }) if cursor.is_over(bounds) => {
                let scroll_y = match delta {
                    ScrollDelta::Lines { y, .. } => *y,
                    ScrollDelta::Pixels { y, .. } => *y / 20.0,
                };
                if scroll_y.abs() > 0.001 {
                    let magnitude = scroll_y.abs().min(3.0) * 0.12;
                    let factor = if scroll_y > 0.0 {
                        1.0 + magnitude
                    } else {
                        1.0 / (1.0 + magnitude)
                    };
                    let (cx, cy) = cursor
                        .position_in(bounds)
                        .map(|p| (p.x, p.y))
                        .unwrap_or((0.0, 0.0));
                    return Some(
                        canvas::Action::publish(Message::MapPreview(
                            PreviewMessage::Zoom(factor, cx, cy),
                        ))
                        .and_capture(),
                    );
                }
            }
            _ => {}
        }
        None
    }

    fn draw(
        &self,
        _interaction: &PreviewCanvasState,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        // If not ready, return an empty geometry
        if !self.state.is_ready() {
            let frame = Frame::new(renderer, bounds.size());
            return vec![frame.into_geometry()];
        }

        let map_data = self.state.map_data.as_ref().unwrap();
        let model = &map_data.model;
        let diagonal = self.state.diagonal;
        let pan_x = self.state.view.pan_x;
        let pan_y = self.state.view.pan_y;
        let zoom = self.state.view.zoom;

        let geometry = self
            .state
            .view
            .tile_cache
            .draw(renderer, bounds.size(), |frame| {
                // Fill background
                frame.fill_rectangle(
                    Point::ORIGIN,
                    bounds.size(),
                    Color::from_rgb(0.1, 0.1, 0.12),
                );

                // ── 1. Ground tile layer (GTL) ────────────────────────────────
                if self.state.view.show_ground && self.state.tiles_ready {
                    draw_tile_layer(
                        frame,
                        &map_data.gtl_tiles,
                        &self.state.gtl_handles,
                        diagonal,
                        pan_x,
                        pan_y,
                        zoom,
                        bounds,
                    );
                }

                // ── 2. Interlaced depth-sorted pass ────────────────────────────
                // Collect buildings, internal sprites, and entity markers, sort by
                // Y-depth, then render in order.
                if self.state.tiles_ready {
                    let nox = model.map_non_occluded_start_x;
                    let noy = model.map_non_occluded_start_y;

                    enum Item<'a> {
                        TiledObject(usize),
                        Sprite(usize),
                        Entity(&'a PreviewEntity),
                    }

                    let mut items: Vec<(i32, i32, i32, Item)> = Vec::new();

                    if self.state.view.show_buildings {
                        for (i, info) in map_data.tiled_infos.iter().enumerate() {
                            let pos = info.y + info.ids.len() as i32 * TILE_H as i32;
                            items.push((pos, 0, i as i32, Item::TiledObject(i)));
                        }
                    }

                    // Internal sprites (thrones, pillars, etc.)
                    // We use internal sprites from map_data only; the preview does
                    // not decode full sprite frame images (too expensive). We render
                    // a small coloured placeholder at each sprite position instead.
                    if self.state.view.show_internal_sprites {
                        for (i, block) in map_data.sprite_blocks.iter().enumerate() {
                            let pos = block.sprite_y + TILE_H as i32;
                            items.push((pos, 1, i as i32, Item::Sprite(i)));
                        }
                    }

                    // Entity markers
                    let entity_pos = |tx: i32, ty: i32| -> i32 {
                        let img_y = dispel_core::map::types::convert_map_coords_to_image_coords(
                            tx, ty, diagonal,
                        )
                        .1;
                        img_y + 32 - noy
                    };

                    for (i, entity) in self.state.entity_markers.iter().enumerate() {
                        let visible = match entity.kind {
                            EntityKind::Monster => self.state.view.show_monsters,
                            EntityKind::Npc => self.state.view.show_npcs,
                            EntityKind::Extra => self.state.view.show_extras,
                            EntityKind::DrawItem => self.state.view.show_draw_items,
                        };
                        if visible {
                            items.push((
                                entity_pos(entity.tile_x, entity.tile_y),
                                3,
                                i as i32,
                                Item::Entity(entity),
                            ));
                        }
                    }

                    items.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));

                    for (_, _, _, item) in &items {
                        match item {
                            Item::TiledObject(obj_i) => {
                                let info = &map_data.tiled_infos[*obj_i];
                                let base_x = (info.x as f32 + nox as f32) * zoom + pan_x;
                                let base_y = (info.y as f32 + noy as f32) * zoom + pan_y;
                                let w = TILE_W * zoom;
                                let h = TILE_H * zoom;
                                for (i, &btl_id) in info.ids.iter().enumerate() {
                                    if btl_id <= 0 {
                                        continue;
                                    }
                                    let handle_id = btl_id.unsigned_abs() as i32;
                                    let Some(handle) = self.state.btl_handles.get(&handle_id) else {
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
                            Item::Sprite(_i) => {
                                // Render a small placeholder marker at the sprite
                                // position since we don't decode sprite images in
                                // the preview.  Using the sprite_block data.
                                // For now: small cyan diamond at block origin.
                                let block = &map_data.sprite_blocks[*_i];
                                let sx = block.sprite_x as f32 * zoom + pan_x;
                                let sy = block.sprite_y as f32 * zoom + pan_y;
                                let r = 6.0 * zoom;
                                let cx = sx + TILE_W * zoom;
                                let cy = sy + TILE_H * zoom * 0.5;
                                if is_visible(cx - r, cy - r, r * 2.0, r * 2.0, bounds) {
                                    frame.fill(
                                        &diamond_path(cx, cy, r),
                                        Color::from_rgba(0.2, 0.8, 0.9, 0.5),
                                    );
                                }
                            }
                            Item::Entity(entity) => {
                                let (px, py) = tile_to_screen(
                                    entity.tile_x,
                                    entity.tile_y,
                                    diagonal,
                                    pan_x,
                                    pan_y,
                                    zoom,
                                );
                                if is_visible(px, py, TILE_W * zoom, TILE_H * zoom, bounds) {
                                    let tile_cx = px + TILE_W * zoom * 0.5;
                                    let tile_cy = py + TILE_H * zoom * 0.5;
                                    // Opacity: confirmed coords = full, uncertain = 50%
                                    let alpha = if entity.confirmed { 0.85 } else { 0.45 };
                                    match entity.kind {
                                        EntityKind::Monster => {
                                            let r = 4.0 * zoom;
                                            frame.fill(
                                                &diamond_path(tile_cx, tile_cy, r),
                                                Color::from_rgba(0.9, 0.15, 0.15, alpha),
                                            );
                                        }
                                        EntityKind::Npc => {
                                            let r = 3.5 * zoom;
                                            frame.fill(
                                                &canvas::Path::circle(
                                                    Point::new(tile_cx, tile_cy),
                                                    r,
                                                ),
                                                Color::from_rgba(0.15, 0.45, 0.9, alpha),
                                            );
                                        }
                                        EntityKind::Extra => {
                                            let s = 5.0 * zoom;
                                            frame.fill_rectangle(
                                                Point::new(tile_cx - s * 0.5, tile_cy - s * 0.5),
                                                Size::new(s, s),
                                                Color::from_rgba(0.95, 0.85, 0.1, alpha),
                                            );
                                        }
                                        EntityKind::DrawItem => {
                                            let r = 6.0 * zoom;
                                            let item_type_color = Color::from_rgba(0.6, 0.6, 0.8, alpha);
                                            frame.fill(
                                                &diamond_path(tile_cx, tile_cy, r),
                                                item_type_color,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // ── 3. Roof layer (BTL flat, on top) ──────────────────────────
                if self.state.view.show_roofs && self.state.tiles_ready {
                    draw_tile_layer(
                        frame,
                        &map_data.btl_tiles,
                        &self.state.btl_handles,
                        diagonal,
                        pan_x,
                        pan_y,
                        zoom,
                        bounds,
                    );
                }
            });

        vec![geometry]
    }

    fn mouse_interaction(
        &self,
        _state: &PreviewCanvasState,
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

// ── Inline rendering helpers (copied from map_editor/canvas/ to avoid extraction) ──

/// Convert tile coordinates to canvas-local screen coordinates.
fn tile_to_screen(
    tx: i32,
    ty: i32,
    diagonal: i32,
    pan_x: f32,
    pan_y: f32,
    zoom: f32,
) -> (f32, f32) {
    let (px, py) = dispel_core::map::types::convert_map_coords_to_image_coords(tx, ty, diagonal);
    (px as f32 * zoom + pan_x, py as f32 * zoom + pan_y)
}

/// Returns true if the rectangle overlaps the visible canvas area.
fn is_visible(x: f32, y: f32, w: f32, h: f32, bounds: Rectangle) -> bool {
    x + w > 0.0 && x < bounds.width && y + h > 0.0 && y < bounds.height
}

/// Draw a layer of tiles from a `(Coords, tile_id)` map.
fn draw_tile_layer(
    frame: &mut Frame,
    tile_map: &std::collections::HashMap<(i32, i32), i32>,
    handles: &std::collections::HashMap<i32, iced::widget::image::Handle>,
    diagonal: i32,
    pan_x: f32,
    pan_y: f32,
    zoom: f32,
    bounds: Rectangle,
) {
    let w = TILE_W * zoom;
    let h = TILE_H * zoom;
    for (&(tx, ty), &tile_id) in tile_map {
        let Some(handle) = handles.get(&tile_id) else {
            continue;
        };
        let (px, py) = tile_to_screen(tx, ty, diagonal, pan_x, pan_y, zoom);
        if !is_visible(px, py, w, h, bounds) {
            continue;
        }
        frame.draw_image(
            Rectangle::new(Point::new(px, py), Size::new(w, h)),
            CoreImage::new(handle.clone()),
        );
    }
}

/// Build a diamond (rotated square) path centered at (cx, cy) with half-size r.
fn diamond_path(cx: f32, cy: f32, r: f32) -> canvas::Path {
    canvas::Path::new(|b| {
        b.move_to(Point::new(cx, cy - r));
        b.line_to(Point::new(cx + r, cy));
        b.line_to(Point::new(cx, cy + r));
        b.line_to(Point::new(cx - r, cy));
        b.close();
    })
}
