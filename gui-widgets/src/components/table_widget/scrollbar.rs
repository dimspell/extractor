//! Scrollbar geometry and rendering.
//!
//! Provides `impl TableWidget` methods for computing scrollbar track/thumb
//! rectangles (used by event handling) and a free `draw_scrollbars` function
//! for painting them.

use iced::advanced::renderer;
use iced::advanced::Renderer as _;
use iced::{
    color, Background, Border, Point, Rectangle, Shadow, Vector,
};

use super::types::Axis;
use super::widget::TableWidget;
use super::SCROLLBAR_THICKNESS;

// ── Scrollbar geometry (impl methods, used by event system) ──────────

impl<'a, Message> TableWidget<'a, Message> {
    /// Vertical scrollbar track + thumb rectangles, or `None` when content
    /// fits vertically.
    pub(crate) fn vertical_scrollbar(
        &self,
        bounds: Rectangle,
        off_y: f32,
    ) -> Option<(Rectangle, Rectangle)> {
        let body = self.body_bounds(bounds);
        let total_h = self.total_height();
        if total_h <= body.height {
            return None;
        }
        let track = Rectangle {
            x: bounds.x + bounds.width - SCROLLBAR_THICKNESS,
            y: body.y,
            width: SCROLLBAR_THICKNESS,
            height: body.height,
        };
        let thumb_h = (body.height / total_h * body.height).max(20.0);
        let max_off = (total_h - body.height).max(1.0);
        let thumb_y = body.y + (off_y / max_off) * (body.height - thumb_h);
        let thumb = Rectangle {
            x: track.x + 1.0,
            y: thumb_y,
            width: SCROLLBAR_THICKNESS - 2.0,
            height: thumb_h,
        };
        Some((track, thumb))
    }

    /// Horizontal scrollbar track + thumb rectangles, or `None` when content
    /// fits horizontally.
    pub(crate) fn horizontal_scrollbar(
        &self,
        bounds: Rectangle,
        off_x: f32,
    ) -> Option<(Rectangle, Rectangle)> {
        let body = self.body_bounds(bounds);
        let total_w = self.total_width();
        if total_w <= body.width {
            return None;
        }
        let track = Rectangle {
            x: bounds.x,
            y: bounds.y + bounds.height - SCROLLBAR_THICKNESS,
            width: body.width,
            height: SCROLLBAR_THICKNESS,
        };
        let thumb_w = (body.width / total_w * body.width).max(20.0);
        let max_off = (total_w - body.width).max(1.0);
        let thumb_x = bounds.x + (off_x / max_off) * (body.width - thumb_w);
        let thumb = Rectangle {
            x: thumb_x,
            y: track.y + 1.0,
            width: thumb_w,
            height: SCROLLBAR_THICKNESS - 2.0,
        };
        Some((track, thumb))
    }

    /// Which scrollbar (if any) the cursor is currently over. The hit-area
    /// covers the whole track so the thumb still feels grabbable when the
    /// cursor hits anywhere along the bar.
    pub(crate) fn scrollbar_under(
        &self,
        bounds: Rectangle,
        off: Vector,
        p: Point,
    ) -> Option<Axis> {
        if let Some((track, _)) = self.vertical_scrollbar(bounds, off.y) {
            if track.contains(p) {
                return Some(Axis::Vertical);
            }
        }
        if let Some((track, _)) = self.horizontal_scrollbar(bounds, off.x) {
            if track.contains(p) {
                return Some(Axis::Horizontal);
            }
        }
        None
    }

    /// Convenience predicate for the cursor-over-scrollbar check.
    pub(crate) fn over_scrollbar(&self, bounds: Rectangle, off: Vector, p: Point) -> bool {
        self.scrollbar_under(bounds, off, p).is_some()
    }
}

// ── Scrollbar painting (free function) ───────────────────────────────

/// Paint vertical and horizontal scrollbar thumbs along the right and bottom
/// edges of `bounds` to reflect `off` against the total content size.
///
/// When `active_axis` matches an axis, that scrollbar's thumb is drawn 1.5×
/// thicker and a few shades lighter so the user can see it's grabbable.
pub(crate) fn draw_scrollbars(
    renderer: &mut iced::Renderer,
    bounds: Rectangle,
    body: Rectangle,
    off: Vector,
    total_w: f32,
    total_h: f32,
    active_axis: Option<Axis>,
) {
    let track_color = color!(0x141210);
    let thumb_idle = color!(0x5d4037);
    let thumb_active = color!(0xB97024);
    let border_idle = color!(0x5d4037);
    let border_active = color!(0xB97024);

    // ── Vertical ──────────────────────────────────────────────────────
    if total_h > body.height {
        let track = Rectangle {
            x: bounds.x + bounds.width - SCROLLBAR_THICKNESS,
            y: body.y,
            width: SCROLLBAR_THICKNESS,
            height: body.height,
        };
        let thumb_h = (body.height / total_h * body.height).max(20.0);
        let max_off = (total_h - body.height).max(1.0);
        let thumb_y = body.y + (off.y / max_off) * (body.height - thumb_h);

        let active = active_axis == Some(Axis::Vertical);
        let extra = if active {
            SCROLLBAR_THICKNESS * 0.5
        } else {
            0.0
        };
        let thumb_w = SCROLLBAR_THICKNESS - 2.0 + extra;
        let thumb_x = track.x + 1.0 - extra;

        renderer.fill_quad(
            renderer::Quad {
                bounds: track,
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(track_color),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: thumb_x,
                    y: thumb_y,
                    width: thumb_w,
                    height: thumb_h,
                },
                border: Border {
                    color: if active { border_active } else { border_idle },
                    width: if active { 1.0 } else { 0.5 },
                    radius: 0.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(if active { thumb_active } else { thumb_idle }),
        );
    }

    // ── Horizontal ────────────────────────────────────────────────────
    if total_w > body.width {
        let track = Rectangle {
            x: bounds.x,
            y: bounds.y + bounds.height - SCROLLBAR_THICKNESS,
            width: body.width,
            height: SCROLLBAR_THICKNESS,
        };
        let thumb_w = (body.width / total_w * body.width).max(20.0);
        let max_off = (total_w - body.width).max(1.0);
        let thumb_x = bounds.x + (off.x / max_off) * (body.width - thumb_w);

        let active = active_axis == Some(Axis::Horizontal);
        let extra = if active {
            SCROLLBAR_THICKNESS * 0.5
        } else {
            0.0
        };
        let thumb_h = SCROLLBAR_THICKNESS - 2.0 + extra;
        let thumb_y = track.y + 1.0 - extra;

        renderer.fill_quad(
            renderer::Quad {
                bounds: track,
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(track_color),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: thumb_x,
                    y: thumb_y,
                    width: thumb_w,
                    height: thumb_h,
                },
                border: Border {
                    color: if active { border_active } else { border_idle },
                    width: if active { 1.0 } else { 0.5 },
                    radius: 0.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(if active { thumb_active } else { thumb_idle }),
        );
    }
}
