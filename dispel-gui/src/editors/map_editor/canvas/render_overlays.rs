// ── MapCanvasOverlaysLayer — overlay elements (collisions, events, entities) ──

use super::hit_test::{entity_tile, find_hovered_element};
use super::input::MapCanvas;
use crate::components::map_render::{
    MapCanvasState, TILE_H, TILE_W, diamond_path, draw_item_color, is_visible, screen_to_tile,
    tile_center, tile_to_screen,
};
use crate::editors::map_editor::message::SelectedEntity;
use crate::editors::map_editor::state::MapEditorState;
use crate::message::Message;
use iced::widget::canvas::{self, Action, Frame, Geometry, Text as CanvasText};
use iced::widget::text::Alignment as TextAlignment;
use iced::{Color, Event, Font, Point, Rectangle, alignment, mouse};

/// Canvas Program for overlay elements (collisions, events, entities).
/// Drawn on top of tiles canvas using a separate canvas in a Stack.
pub struct MapCanvasOverlaysLayer<'a> {
    pub state: &'a MapEditorState,
    pub tab_id: usize,
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
        let static_geometry =
            self.state
                .view
                .overlay_cache
                .draw(renderer, bounds.size(), |frame| {
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
                            // Draw diamond instead of rectangle
                            let cx = px + w * 0.5; // Center x
                            let cy = py + h * 0.5; // Center y
                            let dx = w * 0.5; // Half width
                            let dy = h * 0.5; // Half height
                            frame.fill(
                                &canvas::Path::new(|b| {
                                    b.move_to(Point::new(cx, cy - dy)); // Top
                                    b.line_to(Point::new(cx + dx, cy)); // Right
                                    b.line_to(Point::new(cx, cy + dy)); // Bottom
                                    b.line_to(Point::new(cx - dx, cy)); // Left
                                    b.close();
                                }),
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
                            let label_size = (11.0 * zoom).max(6.0);
                            frame.fill_text(CanvasText {
                                content: event.event_id.to_string(),
                                position: Point::new(ecx, ecy - 10.0 * zoom),
                                color: Color::WHITE,
                                size: iced::Pixels(label_size),
                                font: Font::DEFAULT,
                                align_x: TextAlignment::Center,
                                align_y: alignment::Vertical::Bottom,
                                shaping: iced::widget::text::Shaping::Basic,
                                line_height: iced::widget::text::LineHeight::default(),
                                max_width: f32::INFINITY,
                                ellipsis: iced::widget::text::Ellipsis::None,
                                wrapping: iced::widget::text::Wrapping::None,
                            });
                        }
                    }

                    // Draw items overlay (coloured diamond + item_id label)
                    if self.state.view.show_draw_items {
                        for di in &self.state.data.draw_items {
                            let (px, py) = tile_to_screen(
                                di.x_coord, di.y_coord, diagonal, pan_x, pan_y, zoom,
                            );
                            if is_visible(px, py, TILE_W * zoom, TILE_H * zoom, bounds) {
                                let (tile_cx, tile_cy) = tile_center(px, py, zoom);
                                let color = draw_item_color(
                                    di.item
                                        .item_type()
                                        .unwrap_or(dispel_core::ItemTypeId::Other),
                                );
                                let r = 6.0 * zoom;
                                frame.fill(&diamond_path(tile_cx, tile_cy, r), color);
                                let label = di.item.item_id().to_string();
                                let label_size = (9.0 * zoom).max(6.0);
                                frame.fill_text(CanvasText {
                                    content: label,
                                    position: Point::new(tile_cx, tile_cy - r - 2.0 * zoom),
                                    color: Color::WHITE,
                                    size: iced::Pixels(label_size),
                                    font: Font::MONOSPACE,
                                    align_x: TextAlignment::Center,
                                    align_y: alignment::Vertical::Bottom,
                                    shaping: iced::widget::text::Shaping::Basic,
                                    line_height: iced::widget::text::LineHeight::default(),
                                    max_width: f32::INFINITY,
                                    ellipsis: iced::widget::text::Ellipsis::None,
                                    wrapping: iced::widget::text::Wrapping::None,
                                });
                            }
                        }
                    }

                    // NPC waypoint arrows overlay
                    if self.state.view.show_npc_waypoints {
                        const ARROW_COLORS: [Color; 4] = [
                            Color::from_rgb(0.2, 0.8, 0.2),
                            Color::from_rgb(0.2, 0.2, 0.8),
                            Color::from_rgb(0.8, 0.2, 0.2),
                            Color::from_rgb(0.8, 0.8, 0.2),
                        ];

                        for npc in &self.state.data.npcs {
                            let waypoints = [
                                (npc.goto1_x, npc.goto1_y),
                                (npc.goto2_x, npc.goto2_y),
                                (npc.goto3_x, npc.goto3_y),
                                (npc.goto4_x, npc.goto4_y),
                            ]
                            .into_iter()
                            .filter(|&(x, y)| x != 0 || y != 0)
                            .collect::<Vec<(i32, i32)>>();

                            if waypoints.len() < 2 {
                                continue;
                            }

                            for j in 0..waypoints.len() {
                                let (sx, sy) = tile_to_screen(
                                    waypoints[j].0,
                                    waypoints[j].1,
                                    diagonal,
                                    pan_x,
                                    pan_y,
                                    zoom,
                                );
                                let (ex, ey) = tile_to_screen(
                                    waypoints[(j + 1) % waypoints.len()].0,
                                    waypoints[(j + 1) % waypoints.len()].1,
                                    diagonal,
                                    pan_x,
                                    pan_y,
                                    zoom,
                                );

                                if !is_visible(sx, sy, TILE_W * zoom, TILE_H * zoom, bounds)
                                    && !is_visible(ex, ey, TILE_W * zoom, TILE_H * zoom, bounds)
                                {
                                    continue;
                                }

                                let dx = ex - sx;
                                let dy = ey - sy;
                                let length = (dx * dx + dy * dy).sqrt();
                                if length < 1.0 {
                                    continue;
                                }

                                let nx = dx / length;
                                let ny = dy / length;
                                let head_length = 8.0 * zoom;
                                let head_width = 4.0 * zoom;
                                let color = ARROW_COLORS[j % ARROW_COLORS.len()];

                                let line_end_x = ex - head_length * nx;
                                let line_end_y = ey - head_length * ny;

                                frame.stroke(
                                    &canvas::Path::new(|b| {
                                        b.move_to(Point::new(sx, sy));
                                        b.line_to(Point::new(line_end_x, line_end_y));
                                    }),
                                    canvas::Stroke::default()
                                        .with_color(color)
                                        .with_width(2.0 * zoom),
                                );

                                frame.fill(
                                    &canvas::Path::new(|b| {
                                        let hx1 = line_end_x + head_width * ny;
                                        let hy1 = line_end_y - head_width * nx;
                                        let hx2 = line_end_x - head_width * ny;
                                        let hy2 = line_end_y + head_width * nx;
                                        b.move_to(Point::new(ex, ey));
                                        b.line_to(Point::new(hx1, hy1));
                                        b.line_to(Point::new(hx2, hy2));
                                        b.close();
                                    }),
                                    color,
                                );

                                let label_cx = sx + TILE_W * zoom * 0.5;
                                let label_cy = sy + TILE_H * zoom * 0.5 - 14.0 * zoom;
                                let label_size = (11.0 * zoom).max(6.0);
                                frame.fill_text(CanvasText {
                                    content: (j + 1).to_string(),
                                    position: Point::new(label_cx, label_cy),
                                    color: Color::WHITE,
                                    size: iced::Pixels(label_size),
                                    font: Font::DEFAULT,
                                    align_x: TextAlignment::Center,
                                    align_y: alignment::Vertical::Bottom,
                                    shaping: iced::widget::text::Shaping::Basic,
                                    line_height: iced::widget::text::LineHeight::default(),
                                    max_width: f32::INFINITY,
                                    ellipsis: iced::widget::text::Ellipsis::None,
                                    wrapping: iced::widget::text::Wrapping::None,
                                });
                            }
                        }
                    }

                    // Selection ring
                    if let Some(sel) = self.state.view.selected_entity
                        && let Some((stx, sty)) = entity_tile(sel, self.state)
                    {
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
                });

        // ── Cursor-dependent overlay (uncached) ────────────────────────────────
        // Redrawn every frame; kept separate so mouse moves don't bust the cache above.
        // Use the cursor argument (resolved at render time) rather than stored state,
        // which can lag when two stacked canvases race on MouseMoved dispatch.
        let (cursor_cx, cursor_cy) = cursor
            .position_in(bounds)
            .map(|p| (p.x, p.y))
            .unwrap_or((f32::NAN, f32::NAN));

        let hovered_element = if cursor_cx.is_finite() && cursor_cy.is_finite() {
            find_hovered_element(self.state, cursor_cx, cursor_cy)
        } else {
            None
        };

        let mut cursor_frame = Frame::new(renderer, bounds.size());

        // Cursor tile highlight (uses screen_to_tile so the diamond is always
        // drawn at the tile whose diamond actually contains the cursor).
        if cursor_cx.is_finite()
            && cursor_cy.is_finite()
            && let Some((tile_x, tile_y)) = screen_to_tile(
                cursor_cx,
                cursor_cy,
                diagonal,
                pan_x,
                pan_y,
                zoom,
                model.tiled_map_width,
                model.tiled_map_height,
            )
        {
            let (px, py) = tile_to_screen(tile_x, tile_y, diagonal, pan_x, pan_y, zoom);
            let w = TILE_W * zoom;
            let h = TILE_H * zoom;
            // Brighter green when hovering over a clickable element.
            let alpha = if hovered_element.is_some() {
                0.40
            } else {
                0.15
            };
            // Draw diamond instead of rectangle
            let cx = px + w * 0.5; // Center x
            let cy = py + h * 0.5; // Center y
            let dx = w * 0.5; // Half width
            let dy = h * 0.5; // Half height
            cursor_frame.fill(
                &canvas::Path::new(|b| {
                    b.move_to(Point::new(cx, cy - dy)); // Top
                    b.line_to(Point::new(cx + dx, cy)); // Right
                    b.line_to(Point::new(cx, cy + dy)); // Bottom
                    b.line_to(Point::new(cx - dx, cy)); // Left
                    b.close();
                }),
                Color::from_rgba(0.2, 0.9, 0.3, alpha),
            );
        }

        // Extract selected entity tile coords for comparison below
        let (selected_tile_x, selected_tile_y) = self
            .state
            .view
            .selected_entity
            .and_then(|sel| entity_tile(sel, self.state))
            .unwrap_or((i32::MAX, i32::MAX));

        // Hover ring for entities (monsters/NPCs/extras only — shown with a yellow ring)
        if let Some(hov) = &hovered_element
            && *hov != self.state.view.selected_entity.unwrap_or(*hov)
            && let Some((htx, hty)) = entity_tile(*hov, self.state)
        {
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

        // Hover highlight for collision tiles (red circle)
        if let Some(SelectedEntity::CollisionTile(ctx, cty)) = hovered_element
            && (ctx != selected_tile_x || cty != selected_tile_y)
        {
            let (cpx, cpy) = tile_to_screen(ctx, cty, diagonal, pan_x, pan_y, zoom);
            let ccx = cpx + TILE_W * zoom * 0.5;
            let ccy = cpy + TILE_H * zoom * 0.5;
            let r = 14.0 * zoom;
            cursor_frame.stroke(
                &canvas::Path::circle(Point::new(ccx, ccy), r),
                canvas::Stroke::default()
                    .with_color(Color::from_rgba(0.8, 0.1, 0.1, 0.6))
                    .with_width(2.0 * zoom),
            );
        }

        // Hover highlight for event tiles (magenta circle)
        if let Some(SelectedEntity::EventTile(ctx, cty)) = hovered_element
            && (ctx != selected_tile_x || cty != selected_tile_y)
        {
            let (epx, epy) = tile_to_screen(ctx, cty, diagonal, pan_x, pan_y, zoom);
            let ecx = epx + TILE_W * zoom * 0.5;
            let ecy = epy + TILE_H * zoom * 0.5;
            let r = 14.0 * zoom;
            cursor_frame.stroke(
                &canvas::Path::circle(Point::new(ecx, ecy), r),
                canvas::Stroke::default()
                    .with_color(Color::from_rgba(0.8, 0.1, 0.8, 0.6))
                    .with_width(2.0 * zoom),
            );
        }

        // Tile-coordinate label (top-left corner).
        // Use screen_to_tile for accuracy when over a diamond; fall back to the
        // approximate rounding when between tiles (cursor outside any diamond).
        if cursor_cx.is_finite() && cursor_cy.is_finite() {
            let (tile_x, tile_y) = screen_to_tile(
                cursor_cx,
                cursor_cy,
                diagonal,
                pan_x,
                pan_y,
                zoom,
                model.tiled_map_width,
                model.tiled_map_height,
            )
            .unwrap_or_else(|| {
                // Fallback: approximate rounding (may be off between tiles).
                let world_x = (cursor_cx - pan_x) / zoom;
                let world_y = (cursor_cy - pan_y) / zoom;
                let a = world_x / 32.0;
                let b = (world_y - (diagonal as f32 / 2.0 * 16.0)) / 16.0;
                (
                    ((a - b) / 2.0).round() as i32,
                    ((a + b) / 2.0).round() as i32,
                )
            });
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
                ellipsis: iced::widget::text::Ellipsis::None,
                wrapping: iced::widget::text::Wrapping::None,
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
                ellipsis: iced::widget::text::Ellipsis::None,
                wrapping: iced::widget::text::Wrapping::None,
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
            if find_hovered_element(self.state, pos.x, pos.y).is_some() {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::Grab
            }
        } else {
            mouse::Interaction::Idle
        }
    }
}
