//! Preview overlay canvas — entity markers, selection ring, labels.
//! Input handling delegates to shared map_render::handle_input.

use crate::components::map_render::{
    diamond_path, handle_input, is_visible, tile_to_screen, MapCanvasState, TILE_H, TILE_W,
};
use crate::editors::save_file_viewer::map_preview::message::PreviewMessage;
use crate::editors::save_file_viewer::map_preview::state::{EntityKind, MapPreviewState};
use crate::editors::save_file_viewer::SaveFileViewerMessage;
use crate::message::{Message, MessageExt};
use iced::widget::canvas::{self, Frame, Geometry, Text as CanvasText};
use iced::widget::text::Alignment as TextAlignment;
use iced::{alignment, mouse, Color, Event, Font, Point, Rectangle, Size};

/// Top canvas: renders entity markers, selection ring, and info labels.
/// Transparent background so the tile layer shows through.
pub struct MapPreviewOverlaysLayer<'a> {
    pub state: &'a MapPreviewState,
}

impl<'a> canvas::Program<Message> for MapPreviewOverlaysLayer<'a> {
    type State = MapCanvasState;

    fn update(
        &self,
        interaction: &mut MapCanvasState,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        handle_input(
            interaction,
            event,
            bounds,
            cursor,
            |cx, cy| {
                Message::save_file_viewer(SaveFileViewerMessage::MapPreview(PreviewMessage::Click(
                    cx, cy,
                )))
            },
            |dx, dy| {
                Message::save_file_viewer(SaveFileViewerMessage::MapPreview(PreviewMessage::Pan(
                    dx, dy,
                )))
            },
            |f, cx, cy| {
                Message::save_file_viewer(SaveFileViewerMessage::MapPreview(PreviewMessage::Zoom(
                    f, cx, cy,
                )))
            },
        )
    }

    fn draw(
        &self,
        _interaction: &MapCanvasState,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        if !self.state.is_ready() {
            let frame = Frame::new(renderer, bounds.size());
            return vec![frame.into_geometry()];
        }

        let diagonal = self.state.diagonal;
        let pan_x = self.state.view.pan_x;
        let pan_y = self.state.view.pan_y;
        let zoom = self.state.view.zoom;

        let mut frame = Frame::new(renderer, bounds.size());

        // ── Entity markers (fallback shapes for entities without sprites) ──
        for (i, entity) in self.state.entity_markers.iter().enumerate() {
            // Skip entities that have a decoded sprite (rendered in tile layer)
            if self.state.sprites_ready
                && self
                    .state
                    .entity_sprites
                    .get(i)
                    .and_then(|s| s.as_ref())
                    .is_some()
            {
                continue;
            }

            let visible = match entity.kind {
                EntityKind::Monster => self.state.view.show_monsters,
                EntityKind::Npc => self.state.view.show_npcs,
                EntityKind::Extra => self.state.view.show_objects,
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
                    frame.fill(
                        &diamond_path(tile_cx, tile_cy, r),
                        Color::from_rgba(0.6, 0.6, 0.8, alpha),
                    );
                }
            }
        }

        // ── Selection ring + info label ─────────────────────────────────────
        if let Some(sel_idx) = self.state.selected_marker {
            if let Some(entity) = self.state.entity_markers.get(sel_idx) {
                let visible = match entity.kind {
                    EntityKind::Monster => self.state.view.show_monsters,
                    EntityKind::Npc => self.state.view.show_npcs,
                    EntityKind::Extra => self.state.view.show_objects,
                    EntityKind::DrawItem => self.state.view.show_draw_items,
                };
                if visible {
                    let (px, py) =
                        tile_to_screen(entity.tile_x, entity.tile_y, diagonal, pan_x, pan_y, zoom);
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
                    let label_size = (11.0 * zoom).max(7.0f32);
                    // Monospace advance ≈ 0.6em; count glyphs (not bytes) for width.
                    let text_w = label.chars().count() as f32 * label_size * 0.6;
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
                        max_width: bounds.width,
                        ellipsis: iced::widget::text::Ellipsis::End,
                        wrapping: iced::widget::text::Wrapping::None,
                    });

                    // Name label above the ring (entity.label)
                    let label_size2 = (10.0 * zoom).max(7.0f32);
                    let text_w2 = entity.label.chars().count() as f32 * label_size2 * 0.6;
                    let text_h2 = label_size2 * 1.6;
                    let text_x2 = tile_cx - text_w2 * 0.5;
                    let text_y2 = tile_cy - r - text_h2 - 4.0 * zoom;
                    if !entity.label.is_empty() {
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
                            max_width: bounds.width,
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
