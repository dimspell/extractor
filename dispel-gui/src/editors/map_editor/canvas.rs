/// Canvas Program for the visual map editor.
///
/// Renders isometric tiles from GTL (ground) and BTL (building) layers using
/// decoded image handles stored in `MapEditorState`. Supports mouse drag for
/// panning and scroll wheel for zooming.
use crate::message::editor::map_editor::{MapEditorMessage, SelectedEntity};
use crate::message::{Message, MessageExt};
use crate::state::map_editor::{EntitySpriteHandle, MapEditorState};
use iced::advanced::image::Image as CoreImage;
use iced::widget::canvas::{self, Action, Frame, Geometry, Text as CanvasText};
use iced::widget::image::Handle;
use iced::widget::text::Alignment as TextAlignment;
use iced::{alignment, mouse, Color, Event, Font, Point, Rectangle, Size};
use std::collections::HashMap;

// ── Tile geometry constants ───────────────────────────────────────────────────

/// Rendered width of one tile in pixels (isometric diamond).
pub const TILE_W: f32 = 62.0;
/// Rendered height of one tile in pixels (isometric diamond).
pub const TILE_H: f32 = 32.0;

/// Pixel-space hover radius (world pixels). Entity is considered hovered when
/// the cursor world position is within this many pixels of the tile centre.
const HOVER_RADIUS_PX: f32 = 40.0;

// ── Interaction state ─────────────────────────────────────────────────────────

/// Per-canvas interaction state (managed by Iced).
#[derive(Default)]
pub struct MapCanvasState {
    is_dragging: bool,
    /// Canvas-local drag anchor, set in position_in coordinates.
    drag_last: Option<Point>,
    /// Canvas-local press position used to distinguish click from drag.
    drag_start: Option<Point>,
    /// Entity currently under the cursor (for hover highlight + pointer cursor).
    pub hovered_entity: Option<SelectedEntity>,
}

// ── Canvas Program ────────────────────────────────────────────────────────────

/// Borrowed view of the map editor state, used as the canvas Program.
pub struct MapCanvas<'a> {
    pub state: &'a MapEditorState,
    pub tab_id: usize,
}

/// Canvas Program for tile layers only (GTL, BTL, internal sprites).
/// Images always draw on top of primitives within a single canvas,
/// so we split into two canvases: tiles first, then overlays.
pub struct MapCanvasTilesLayer<'a> {
    pub state: &'a MapEditorState,
    pub tab_id: usize,
}

/// Canvas Program for overlay elements (collisions, events, entities).
/// Drawn on top of tiles canvas using a separate canvas in a Stack.
pub struct MapCanvasOverlaysLayer<'a> {
    pub state: &'a MapEditorState,
    pub tab_id: usize,
}

impl<'a> canvas::Program<Message> for MapCanvas<'a> {
    type State = MapCanvasState;

    fn update(
        &self,
        interaction: &mut MapCanvasState,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        use mouse::{Button, Event as MouseEvent, ScrollDelta};

        match event {
            Event::Mouse(MouseEvent::ButtonPressed(Button::Left)) => {
                // cursor.position_in(bounds) → canvas-local coords, None if outside canvas.
                // This is the critical fix: cursor.position() returns ABSOLUTE window
                // coordinates and is Some() even when the cursor is over the inspector
                // panel. Using position_in ensures we only start a drag/click when the
                // cursor is actually inside the canvas area.
                if let Some(pos) = cursor.position_in(bounds) {
                    interaction.is_dragging = true;
                    interaction.drag_last = Some(pos);
                    interaction.drag_start = Some(pos);
                    return Some(Action::capture());
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
                                Action::publish(Message::map_editor(
                                    MapEditorMessage::CanvasClicked(self.tab_id, pos.x, pos.y),
                                ))
                                .and_capture(),
                            );
                        }
                    }
                }
            }
            Event::Mouse(MouseEvent::CursorMoved { .. }) => {
                if interaction.is_dragging {
                    // Use position_from for drag: gives canvas-local coords but works
                    // even when the cursor strays outside the canvas bounds.
                    if let Some(last) = interaction.drag_last {
                        if let Some(pos) = cursor.position_from(bounds.position()) {
                            let dx = pos.x - last.x;
                            let dy = pos.y - last.y;
                            interaction.drag_last = Some(pos);
                            return Some(
                                Action::publish(Message::map_editor(MapEditorMessage::PanChanged(
                                    self.tab_id,
                                    dx,
                                    dy,
                                )))
                                .and_capture(),
                            );
                        }
                    }
                } else {
                    // Update hover entity and cursor tile-coordinate overlay.
                    if let Some(pos) = cursor.position_in(bounds) {
                        // Recompute hovered entity each frame (cheap).
                        let hover = self.find_hovered_entity(pos.x, pos.y);
                        interaction.hovered_entity = hover;
                        return Some(Action::publish(Message::map_editor(
                            MapEditorMessage::MouseMoved(
                                self.tab_id,
                                pos.x,
                                pos.y,
                                bounds.width,
                                bounds.height,
                            ),
                        )));
                    } else {
                        interaction.hovered_entity = None;
                        return Some(Action::publish(Message::map_editor(
                            MapEditorMessage::MouseMoved(self.tab_id, f32::NAN, f32::NAN, 0.0, 0.0),
                        )));
                    }
                }
            }
            Event::Mouse(MouseEvent::CursorLeft) => {
                interaction.hovered_entity = None;
                return Some(Action::publish(Message::map_editor(
                    MapEditorMessage::MouseMoved(self.tab_id, f32::NAN, f32::NAN, 0.0, 0.0),
                )));
            }
            Event::Mouse(MouseEvent::WheelScrolled { delta }) => {
                if cursor.is_over(bounds) {
                    let scroll_y = match delta {
                        ScrollDelta::Lines { y, .. } => *y,
                        ScrollDelta::Pixels { y, .. } => *y / 20.0,
                    };
                    if scroll_y.abs() > 0.001 {
                        // Multiplicative zoom: symmetric in/out, natural on trackpads.
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
                            Action::publish(Message::map_editor(MapEditorMessage::ZoomChanged(
                                self.tab_id,
                                factor,
                                cx,
                                cy,
                            )))
                            .and_capture(),
                        );
                    }
                }
            }
            _ => {}
        }
        None
    }

    fn draw(
        &self,
        _interaction: &MapCanvasState,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let frame = Frame::new(renderer, bounds.size());
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        interaction: &MapCanvasState,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if interaction.is_dragging {
            return mouse::Interaction::Grabbing;
        }
        if cursor.is_over(bounds) {
            if interaction.hovered_entity.is_some() {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::Grab
            }
        } else {
            mouse::Interaction::default()
        }
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

impl<'a> MapCanvas<'a> {
    /// Find the entity (if any) within hover range of the given canvas-local point.
    fn find_hovered_entity(&self, cx: f32, cy: f32) -> Option<SelectedEntity> {
        find_hovered_entity_impl(self.state, cx, cy)
    }
}

/// Free function so both `MapCanvas` and `MapCanvasOverlaysLayer::draw` can use it
#[allow(clippy::question_mark)]
/// without going through the message pipeline.  Also called from the update handler
/// for `CanvasClicked` so click and hover share the same hit-test logic.
pub fn find_hovered_entity_impl(
    state: &MapEditorState,
    cx: f32,
    cy: f32,
) -> Option<SelectedEntity> {
    let Some(map_handle) = state.map_data() else {
        return None;
    };
    let model = &map_handle.0.model;
    let diagonal = model.tiled_map_width + model.tiled_map_height;

    // Convert canvas-local to world pixel space.
    let world_x = (cx - state.view.pan_x) / state.view.zoom;
    let world_y = (cy - state.view.pan_y) / state.view.zoom;

    let r2 = HOVER_RADIUS_PX * HOVER_RADIUS_PX;
    let mut best: Option<(f32, SelectedEntity)> = None;

    for (i, m) in state.data.monsters.iter().enumerate() {
        let (wx, wy) = tile_world_center(m.pos_x, m.pos_y, diagonal);
        let d2 = (world_x - wx).powi(2) + (world_y - wy).powi(2);
        if d2 < r2 && best.as_ref().is_none_or(|(bd, _)| d2 < *bd) {
            best = Some((d2, SelectedEntity::Monster(i)));
        }
    }
    for (i, n) in state.data.npcs.iter().enumerate() {
        let (nx, ny) = npc_pos(n);
        let (wx, wy) = tile_world_center(nx, ny, diagonal);
        let d2 = (world_x - wx).powi(2) + (world_y - wy).powi(2);
        if d2 < r2 && best.as_ref().is_none_or(|(bd, _)| d2 < *bd) {
            best = Some((d2, SelectedEntity::Npc(i)));
        }
    }
    for (i, e) in state.data.extra_refs.iter().enumerate() {
        let (wx, wy) = tile_world_center(e.x_pos, e.y_pos, diagonal);
        let d2 = (world_x - wx).powi(2) + (world_y - wy).powi(2);
        if d2 < r2 && best.as_ref().is_none_or(|(bd, _)| d2 < *bd) {
            best = Some((d2, SelectedEntity::Extra(i)));
        }
    }

    best.map(|(_, e)| e)
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

impl<'a> canvas::Program<Message> for MapCanvasOverlaysLayer<'a> {
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
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let Some(map_handle) = self.state.map_data() else {
            let frame = Frame::new(renderer, bounds.size());
            return vec![frame.into_geometry()];
        };
        let map_data = &map_handle.0;
        let model = &map_data.model;
        let diagonal = model.tiled_map_width + model.tiled_map_height;

        let pan_x = self.state.view.pan_x;
        let pan_y = self.state.view.pan_y;
        let zoom = self.state.view.zoom;

        // ── Static overlay (cached) ────────────────────────────────────────────
        // Cleared on pan, zoom, layer toggle, selection change, entity edit.
        // NOT cleared on MouseMoved, so collision/event cells aren't redrawn each frame.
        #[allow(unused_mut)]
        let static_geometry =
            self.state
                .view
                .overlay_cache
                .draw(renderer, bounds.size(), |mut frame| {
                    // Collision overlay
                    if self.state.view.show_collisions {
                        for (&(tx, ty), &blocked) in &map_data.collisions {
                            if !blocked {
                                continue;
                            }
                            let (px, py) = tile_to_screen(tx, ty, diagonal, pan_x, pan_y, zoom);
                            let w = TILE_W * zoom;
                            let h = TILE_H * zoom;
                            if !is_visible(px, py, w, h, bounds) {
                                continue;
                            }
                            frame.fill_rectangle(
                                Point::new(px, py),
                                Size::new(w, h),
                                Color::from_rgba(0.8, 0.1, 0.1, 0.3),
                            );
                        }
                    }

                    // Event overlay
                    if self.state.view.show_events {
                        for (&(tx, ty), event) in &map_data.events {
                            if event.event_id == 0 {
                                continue;
                            }
                            let (px, py) = tile_to_screen(tx, ty, diagonal, pan_x, pan_y, zoom);
                            if !is_visible(px, py, TILE_W * zoom, TILE_H * zoom, bounds) {
                                continue;
                            }
                            let r = 3.0 * zoom;
                            let ecx = px + TILE_W * zoom * 0.5;
                            let ecy = py + TILE_H * zoom * 0.5;
                            frame.fill(
                                &canvas::Path::circle(Point::new(ecx, ecy), r),
                                Color::from_rgb(0.8, 0.1, 0.8),
                            );
                        }
                    }

                    // Selection ring
                    if let Some(sel) = self.state.view.selected_entity {
                        if let Some((stx, sty)) = entity_tile(sel, self.state) {
                            let (px, py) = tile_to_screen(stx, sty, diagonal, pan_x, pan_y, zoom);
                            let r = 14.0 * zoom;
                            let scx = px + TILE_W * zoom * 0.5;
                            let scy = py + TILE_H * zoom * 0.5;
                            frame.stroke(
                                &canvas::Path::circle(Point::new(scx, scy), r),
                                canvas::Stroke::default()
                                    .with_color(Color::from_rgba(1.0, 0.9, 0.2, 0.9))
                                    .with_width(2.0 * zoom),
                            );
                        }
                    }
                });

        // ── Cursor-dependent overlay (uncached) ────────────────────────────────
        // Redrawn every frame; kept separate so mouse moves don't bust the cache above.
        // Use the cursor argument (resolved at render time) rather than stored state,
        // which can lag when two stacked canvases race on MouseMoved dispatch.
        let (cursor_cx, cursor_cy) = cursor
            .position_in(bounds)
            .map(|p| (p.x, p.y))
            .unwrap_or((f32::NAN, f32::NAN));

        let hovered_entity = if cursor_cx.is_finite() && cursor_cy.is_finite() {
            find_hovered_entity_impl(self.state, cursor_cx, cursor_cy)
        } else {
            None
        };

        let mut cursor_frame = Frame::new(renderer, bounds.size());

        // Cursor tile highlight
        if cursor_cx.is_finite() && cursor_cy.is_finite() {
            let world_x = (cursor_cx - pan_x) / zoom;
            let world_y = (cursor_cy - pan_y) / zoom;
            let a = world_x / 32.0;
            let b = (world_y - (diagonal as f32 / 2.0 * 16.0)) / 16.0;
            let tile_x = ((a - b) / 2.0).round() as i32;
            let tile_y = ((a + b) / 2.0).round() as i32;
            let (px, py) = tile_to_screen(tile_x, tile_y, diagonal, pan_x, pan_y, zoom);
            let w = TILE_W * zoom;
            let h = TILE_H * zoom;
            // Brighter green when hovering over a clickable entity.
            let alpha = if hovered_entity.is_some() { 0.40 } else { 0.15 };
            cursor_frame.fill_rectangle(
                Point::new(px, py),
                Size::new(w, h),
                Color::from_rgba(0.2, 0.9, 0.3, alpha),
            );
        }

        // Hover ring (only when not already the selected entity)
        if let Some(hov) = hovered_entity {
            if hov != self.state.view.selected_entity.unwrap_or(hov) {
                if let Some((htx, hty)) = entity_tile(hov, self.state) {
                    let (px, py) = tile_to_screen(htx, hty, diagonal, pan_x, pan_y, zoom);
                    let r = 14.0 * zoom;
                    let hcx = px + TILE_W * zoom * 0.5;
                    let hcy = py + TILE_H * zoom * 0.5;
                    cursor_frame.stroke(
                        &canvas::Path::circle(Point::new(hcx, hcy), r),
                        canvas::Stroke::default()
                            .with_color(Color::from_rgba(1.0, 0.9, 0.2, 0.45))
                            .with_width(2.0 * zoom),
                    );
                }
            }
        }

        // Tile-coordinate label (top-left corner)
        if cursor_cx.is_finite() && cursor_cy.is_finite() {
            let world_x = (cursor_cx - pan_x) / zoom;
            let world_y = (cursor_cy - pan_y) / zoom;
            let a = world_x / 32.0;
            let b = (world_y - (diagonal as f32 / 2.0 * 16.0)) / 16.0;
            let tile_x = ((a - b) / 2.0).round() as i32;
            let tile_y = ((a + b) / 2.0).round() as i32;
            let label = format!("X: {}  Y: {}", tile_x, tile_y);
            cursor_frame.fill_text(CanvasText {
                content: label.clone(),
                position: Point::new(11.5, 11.5),
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.75),
                size: iced::Pixels(13.0),
                font: Font::DEFAULT,
                align_x: TextAlignment::Left,
                align_y: alignment::Vertical::Top,
                shaping: iced::widget::text::Shaping::Basic,
                line_height: iced::widget::text::LineHeight::default(),
                max_width: f32::INFINITY,
            });
            cursor_frame.fill_text(CanvasText {
                content: label,
                position: Point::new(10.0, 10.0),
                color: Color::WHITE,
                size: iced::Pixels(13.0),
                font: Font::DEFAULT,
                align_x: TextAlignment::Left,
                align_y: alignment::Vertical::Top,
                shaping: iced::widget::text::Shaping::Basic,
                line_height: iced::widget::text::LineHeight::default(),
                max_width: f32::INFINITY,
            });
        }

        vec![static_geometry, cursor_frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &MapCanvasState,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if let Some(pos) = cursor.position_in(bounds) {
            // Recompute hover directly rather than reading interaction.hovered_entity,
            // which belongs to the overlay layer's own State instance and may lag
            // one frame behind the tile layer's instance.
            if find_hovered_entity_impl(self.state, pos.x, pos.y).is_some() {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::Grab
            }
        } else {
            mouse::Interaction::Idle
        }
    }
}

/// Return the tile coordinates for an entity.
fn entity_tile(sel: SelectedEntity, state: &MapEditorState) -> Option<(i32, i32)> {
    match sel {
        SelectedEntity::Monster(i) => state.data.monsters.get(i).map(|m| (m.pos_x, m.pos_y)),
        SelectedEntity::Npc(i) => state.data.npcs.get(i).map(|n| {
            let (x, y) = npc_pos(n);
            (x, y)
        }),
        SelectedEntity::Extra(i) => state.data.extra_refs.get(i).map(|e| (e.x_pos, e.y_pos)),
    }
}

/// First active waypoint (or goto1 fallback) for an NPC.
fn npc_pos(n: &dispel_core::NPC) -> (i32, i32) {
    [
        (n.goto1_filled, n.goto1_x, n.goto1_y),
        (n.goto2_filled, n.goto2_x, n.goto2_y),
        (n.goto3_filled, n.goto3_x, n.goto3_y),
        (n.goto4_filled, n.goto4_x, n.goto4_y),
    ]
    .iter()
    .find(|(filled, _, _)| i32::from(*filled) != 0)
    .map(|&(_, x, y)| (x, y))
    .unwrap_or((n.goto1_x, n.goto1_y))
}

/// World pixel centre of an isometric tile.
fn tile_world_center(tx: i32, ty: i32, diagonal: i32) -> (f32, f32) {
    let (px, py) = dispel_core::map::types::convert_map_coords_to_image_coords(tx, ty, diagonal);
    (px as f32 + TILE_W * 0.5, py as f32 + TILE_H * 0.5)
}

/// Screen-space centre of a tile bounding box at the given zoom.
/// `px`, `py` are the top-left corner returned by `tile_to_screen`.
#[inline]
fn tile_center(px: f32, py: f32, zoom: f32) -> (f32, f32) {
    (px + TILE_W * zoom * 0.5, py + TILE_H * zoom * 0.5)
}

// ── Shared draw helpers ───────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn draw_tile_layer(
    frame: &mut Frame,
    tile_map: &std::collections::HashMap<dispel_core::map::types::Coords, i32>,
    handles: &HashMap<i32, Handle>,
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
        let rect = Rectangle::new(Point::new(px, py), Size::new(w, h));
        frame.draw_image(rect, CoreImage::new(handle.clone()));
    }
}

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

/// Returns true if the rectangle overlaps the visible canvas area (canvas-local coords).
fn is_visible(x: f32, y: f32, w: f32, h: f32, bounds: Rectangle) -> bool {
    x + w > 0.0 && x < bounds.width && y + h > 0.0 && y < bounds.height
}

/// Render an entity sprite handle onto the canvas frame.
fn draw_entity_sprite(
    frame: &mut Frame,
    spr: &EntitySpriteHandle,
    tile_cx: f32,
    tile_cy: f32,
    zoom: f32,
) {
    let w = spr.width as f32 * zoom;
    let h = spr.height as f32 * zoom;
    let dest_x = if spr.flip {
        tile_cx + (spr.origin_x as f32 - spr.width as f32) * zoom
    } else {
        tile_cx - spr.origin_x as f32 * zoom
    };
    let dest_y = tile_cy - spr.origin_y as f32 * zoom;
    frame.draw_image(
        Rectangle::new(Point::new(dest_x, dest_y), Size::new(w, h)),
        CoreImage::new(spr.handle.clone()),
    );
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

// ── Tile decoder ──────────────────────────────────────────────────────────────

/// Decode a single tile from raw RGB565 bytes to a 62×32 RGBA image.
pub fn decode_tile_to_rgba(tile_bytes: &[u8]) -> Vec<u8> {
    let mut rgba = vec![0u8; 62 * 32 * 4];
    let mut src = 0usize;

    for y in 0u32..32 {
        let hs = y.min(31 - y) as usize;
        let x_offset = (15 - hs) * 2;
        let width = 2 + hs * 4;

        for x in 0..width {
            let pixel565 = u16::from_le_bytes([tile_bytes[src * 2], tile_bytes[src * 2 + 1]]);
            src += 1;

            let r5 = ((pixel565 >> 11) & 0x1F) as u32;
            let g6 = ((pixel565 >> 5) & 0x3F) as u32;
            let b5 = (pixel565 & 0x1F) as u32;

            let r = (r5 * 255 / 31) as u8;
            let g = (g6 * 255 / 63) as u8;
            let b = (b5 * 255 / 31) as u8;
            let a = if r == 0 && g == 0 && b == 0 {
                0u8
            } else {
                255u8
            };

            let dst = (y as usize * 62 + x_offset + x) * 4;
            rgba[dst] = r;
            rgba[dst + 1] = g;
            rgba[dst + 2] = b;
            rgba[dst + 3] = a;
        }
    }

    rgba
}

/// Decode all unique tile IDs referenced in the given HashMap from a tileset file.
pub fn decode_tileset_file(
    path: &std::path::Path,
    tile_ids: &std::collections::HashSet<i32>,
) -> Result<HashMap<i32, Vec<u8>>, String> {
    use std::io::{Read, Seek, SeekFrom};

    if tile_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut file =
        std::fs::File::open(path).map_err(|e| format!("Cannot open {}: {}", path.display(), e))?;

    let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let max_tiles = (file_size / 2048) as i32;

    let mut result = HashMap::with_capacity(tile_ids.len());
    let mut buf = [0u8; 2048];

    for &tile_id in tile_ids {
        if tile_id < 0 || tile_id >= max_tiles {
            continue;
        }
        let offset = tile_id as u64 * 2048;
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| format!("Seek error in {}: {}", path.display(), e))?;
        file.read_exact(&mut buf)
            .map_err(|e| format!("Read error in {}: {}", path.display(), e))?;
        result.insert(tile_id, decode_tile_to_rgba(&buf));
    }

    Ok(result)
}
