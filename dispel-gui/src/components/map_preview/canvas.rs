//! Read-only map preview canvas — tiles layer (images) + overlays layer (primitives).
//!
//! Split into two stacked `canvas::Program` implementations to fix entity marker
//! layering: images in the bottom layer (Y-sorted with buildings), primitive
//! markers in the top transparent overlay layer.
//!
//! Both layers handle mouse input by delegating to the shared `handle_input()`
//! function, mirroring the proven two-canvas pattern from `map_editor/canvas/`.

use crate::components::map_preview::state::{EntityKind, MapPreviewState};
use crate::components::map_preview::PreviewMessage;
use crate::message::Message;
use iced::advanced::image::Image as CoreImage;
use iced::widget::canvas::{self, Frame, Geometry, Text as CanvasText};
use iced::widget::text::Alignment as TextAlignment;
use iced::{alignment, mouse, Color, Event, Font, Point, Rectangle, Size};

// ── Isometric tile constants (shared with map_editor) ─────────────────────────

/// Rendered width of one isometric tile in world pixels.
pub(crate) const TILE_W: f32 = 62.0;
/// Rendered height of one isometric tile in world pixels.
pub(crate) const TILE_H: f32 = 32.0;

// ── Canvas interaction state ──────────────────────────────────────────────────

/// Per-canvas interaction state (managed by Iced).
///
/// Each canvas layer gets its own independent instance.
#[derive(Default)]
pub struct PreviewCanvasState {
    pub is_dragging: bool,
    pub drag_last: Option<Point>,
    pub drag_start: Option<Point>,
}

// ── Shared input handling (both layers delegate here) ─────────────────────────

/// Shared pan/zoom/click handler used by both canvas layers.
///
/// Mirrors `MapCanvas::update()` from `map_editor/canvas/input.rs`.
fn handle_input(
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
            // Emit click only if released inside canvas and barely moved from press.
            if let Some(start) = interaction.drag_start.take() {
                if let Some(pos) = cursor.position_in(bounds) {
                    let dx = pos.x - start.x;
                    let dy = pos.y - start.y;
                    if dx * dx + dy * dy < 25.0 {
                        return Some(
                            canvas::Action::publish(
                                Message::MapPreview(PreviewMessage::Click(pos.x, pos.y)),
                            )
                            .and_capture(),
                        );
                    }
                }
            }
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

// ── Tile Layer (images only) ──────────────────────────────────────────────────

/// Bottom canvas: renders tiles, buildings, and roof images.
///
/// Also handles mouse input (by delegating to `handle_input`) — this matches the
/// map editor's two-canvas pattern where both layers process input.
pub struct MapPreviewTilesLayer<'a> {
    pub state: &'a MapPreviewState,
}

impl<'a> canvas::Program<Message> for MapPreviewTilesLayer<'a> {
    type State = PreviewCanvasState;

    fn update(
        &self,
        interaction: &mut PreviewCanvasState,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        handle_input(interaction, event, bounds, cursor)
    }

    fn draw(
        &self,
        _interaction: &PreviewCanvasState,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
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

                // ── 2. Interlaced depth-sorted pass (buildings + internal sprites + entity sprites) ──
                if self.state.tiles_ready {
                    let nox = model.map_non_occluded_start_x;
                    let noy = model.map_non_occluded_start_y;

                    enum TileItem {
                        TiledObject(usize),
                        InternalSprite(usize),
                        EntitySprite(usize),
                    }

                    let mut items: Vec<(i32, i32, i32, TileItem)> = Vec::new();

                    // Buildings
                    if self.state.view.show_buildings {
                        for (i, info) in map_data.tiled_infos.iter().enumerate() {
                            let pos = info.y + info.ids.len() as i32 * TILE_H as i32;
                            items.push((pos, 0, i as i32, TileItem::TiledObject(i)));
                        }
                    }

                    // Internal map sprites (thrones, decor, vases …)
                    if self.state.view.show_internal_sprites {
                        for (i, spr) in self.state.internal_sprites.iter().enumerate() {
                            items.push((spr.sort_y, 1, 0, TileItem::InternalSprite(i)));
                        }
                    }

                    // Entity sprites (only those with decoded sprites)
                    if self.state.sprites_ready {
                        let entity_pos = |tx: i32, ty: i32| -> i32 {
                            let img_y =
                                dispel_core::map::types::convert_map_coords_to_image_coords(
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
                            if !visible {
                                continue;
                            }
                            let has_sprite = self
                                .state
                                .entity_sprites
                                .get(i)
                                .and_then(|s| s.as_ref())
                                .is_some();
                            if !has_sprite {
                                continue;
                            }
                            let pos = entity_pos(entity.tile_x, entity.tile_y);
                            items.push((pos, 2, i as i32, TileItem::EntitySprite(i)));
                        }
                    }

                    items.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));

                    for (_, _, _, item) in &items {
                        match item {
                            TileItem::TiledObject(obj_i) => {
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
                                    let Some(handle) =
                                        self.state.btl_handles.get(&handle_id)
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
                            TileItem::InternalSprite(i) => {
                                let spr = &self.state.internal_sprites[*i];
                                let sx = spr.x as f32 * zoom + pan_x;
                                let sy = spr.y as f32 * zoom + pan_y;
                                let sw = spr.width as f32 * zoom;
                                let sh = spr.height as f32 * zoom;
                                if is_visible(sx, sy, sw, sh, bounds) {
                                    frame.draw_image(
                                        Rectangle::new(
                                            Point::new(sx, sy),
                                            Size::new(sw, sh),
                                        ),
                                        CoreImage::new(spr.handle.clone()),
                                    );
                                }
                            }
                            TileItem::EntitySprite(i) => {
                                let entity = &self.state.entity_markers[*i];
                                let sprite =
                                    self.state.entity_sprites[*i].as_ref().unwrap();
                                let (px, py) = tile_to_screen(
                                    entity.tile_x,
                                    entity.tile_y,
                                    diagonal,
                                    pan_x,
                                    pan_y,
                                    zoom,
                                );
                                let tile_cx = px + TILE_W * zoom * 0.5;
                                let tile_cy = py + TILE_H * zoom * 0.5;
                                let w = sprite.width as f32 * zoom;
                                let h = sprite.height as f32 * zoom;
                                let dest_x = tile_cx - sprite.origin_x as f32 * zoom;
                                let dest_y = tile_cy - sprite.origin_y as f32 * zoom;
                                if is_visible(dest_x, dest_y, w, h, bounds) {
                                    frame.draw_image(
                                        Rectangle::new(
                                            Point::new(dest_x, dest_y),
                                            Size::new(w, h),
                                        ),
                                        CoreImage::new(sprite.handle.clone()),
                                    );
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

// ── Overlay Layer (primitives only) ───────────────────────────────────────────

/// Top canvas: renders entity markers, draw items, and sprite placeholder
/// shapes. Transparent background so the tile layer shows through.
/// No cache — fewer than 100 items, rendering fresh each frame is fine.
///
/// Also handles mouse input (by delegating to `handle_input`).
pub struct MapPreviewOverlaysLayer<'a> {
    pub state: &'a MapPreviewState,
}

impl<'a> canvas::Program<Message> for MapPreviewOverlaysLayer<'a> {
    type State = PreviewCanvasState;

    fn update(
        &self,
        interaction: &mut PreviewCanvasState,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        handle_input(interaction, event, bounds, cursor)
    }

    fn draw(
        &self,
        _interaction: &PreviewCanvasState,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        // Return empty (transparent) frame when not ready
        if !self.state.is_ready() {
            let frame = Frame::new(renderer, bounds.size());
            return vec![frame.into_geometry()];
        }

        let diagonal = self.state.diagonal;
        let pan_x = self.state.view.pan_x;
        let pan_y = self.state.view.pan_y;
        let zoom = self.state.view.zoom;

        let mut frame = Frame::new(renderer, bounds.size());
        // No background fill — transparent overlay on top of tile layer

        // ── Entity markers (fallback shapes for entities without sprites) ──
        for (i, entity) in self.state.entity_markers.iter().enumerate() {
            // Skip entities that have a decoded sprite — those are rendered
            // as images in the tile layer (Y-sorted with buildings).
            if self.state.sprites_ready {
                if self
                    .state
                    .entity_sprites
                    .get(i)
                    .and_then(|s| s.as_ref())
                    .is_some()
                {
                    continue;
                }
            }

            let visible = match entity.kind {
                EntityKind::Monster => self.state.view.show_monsters,
                EntityKind::Npc => self.state.view.show_npcs,
                EntityKind::Extra => self.state.view.show_extras,
                EntityKind::DrawItem => self.state.view.show_draw_items,
            };
            if !visible {
                continue;
            }

            let (px, py) =
                tile_to_screen(entity.tile_x, entity.tile_y, diagonal, pan_x, pan_y, zoom);
            if !is_visible(px, py, TILE_W * zoom, TILE_H * zoom, bounds) {
                continue;
            }

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
                        &canvas::Path::circle(Point::new(tile_cx, tile_cy), r),
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
                    frame.fill(&diamond_path(tile_cx, tile_cy, r), item_type_color);
                }
            }
        }

        // ── Selection ring + info label ─────────────────────────────────────
        if let Some(sel_idx) = self.state.selected_marker {
            if let Some(entity) = self.state.entity_markers.get(sel_idx) {
                let visible = match entity.kind {
                    EntityKind::Monster => self.state.view.show_monsters,
                    EntityKind::Npc => self.state.view.show_npcs,
                    EntityKind::Extra => self.state.view.show_extras,
                    EntityKind::DrawItem => self.state.view.show_draw_items,
                };
                if visible {
                    let (px, py) = tile_to_screen(
                        entity.tile_x, entity.tile_y, diagonal, pan_x, pan_y, zoom,
                    );
                    let tile_cx = px + TILE_W * zoom * 0.5;
                    let tile_cy = py + TILE_H * zoom * 0.5;
                    let r = 14.0 * zoom;

                    // Bright gold selection ring
                    frame.stroke(
                        &canvas::Path::circle(Point::new(tile_cx, tile_cy), r),
                        canvas::Stroke::default()
                            .with_color(Color::from_rgba(1.0, 0.9, 0.2, 0.9))
                            .with_width(2.0 * zoom),
                    );

                    // Floating info label below the ring
                    let kind_label = match entity.kind {
                        EntityKind::Monster => "Monster",
                        EntityKind::Npc => "NPC",
                        EntityKind::Extra => "Extra",
                        EntityKind::DrawItem => "DrawItem",
                    };
                    let coords = format!("({}, {})", entity.tile_x, entity.tile_y);
                    let label = if let Some(db_id) = entity.db_id {
                        format!("{} #{}  {}", kind_label, db_id, coords)
                    } else {
                        format!("{}  {}", kind_label, coords)
                    };
                    let label_size = (11.0 * zoom).max(7.0);
                    // Semi-transparent dark background pill
                    let text_w = (label.len() as f32 * label_size * 0.6).min(300.0 * zoom);
                    let text_h = label_size * 1.6;
                    let text_x = tile_cx - text_w * 0.5;
                    let text_y = tile_cy + r + 4.0 * zoom;
                    frame.fill_rectangle(
                        Point::new(text_x, text_y),
                        Size::new(text_w, text_h),
                        Color::from_rgba(0.0, 0.0, 0.0, 0.55),
                    );
                    frame.fill_text(CanvasText {
                        content: label,
                        position: Point::new(tile_cx, text_y),
                        color: Color::WHITE,
                        size: iced::Pixels(label_size),
                        font: Font::MONOSPACE,
                        align_x: TextAlignment::Center,
                        align_y: alignment::Vertical::Center,
                        shaping: iced::widget::text::Shaping::Basic,
                        line_height: iced::widget::text::LineHeight::default(),
                        max_width: text_w,
                        ellipsis: iced::widget::text::Ellipsis::None,
                        wrapping: iced::widget::text::Wrapping::None,
                    });

                    // Name label above the ring (entity.label)
                    let label_size2 = (10.0 * zoom).max(7.0);
                    let text_w2 = (entity.label.len() as f32 * label_size2 * 0.6).min(300.0 * zoom);
                    let text_h2 = label_size2 * 1.6;
                    let text_x2 = tile_cx - text_w2 * 0.5;
                    let text_y2 = tile_cy - r - text_h2 - 4.0 * zoom;
                    if entity.label.len() > 0 {
                        frame.fill_rectangle(
                            Point::new(text_x2, text_y2),
                            Size::new(text_w2, text_h2),
                            Color::from_rgba(0.0, 0.0, 0.0, 0.55),
                        );
                        frame.fill_text(CanvasText {
                            content: entity.label.clone(),
                            position: Point::new(tile_cx, text_y2 + text_h2 * 0.5),
                            color: Color::WHITE,
                            size: iced::Pixels(label_size2),
                            font: Font::MONOSPACE,
                            align_x: TextAlignment::Center,
                            align_y: alignment::Vertical::Center,
                            shaping: iced::widget::text::Shaping::Basic,
                            line_height: iced::widget::text::LineHeight::default(),
                            max_width: text_w2,
                            ellipsis: iced::widget::text::Ellipsis::End,
                            wrapping: iced::widget::text::Wrapping::None,
                        });
                    }
                }
            }
        }

        vec![frame.into_geometry()]
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
pub(crate) fn tile_to_screen(
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
