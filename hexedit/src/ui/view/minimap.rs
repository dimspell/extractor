//! Minimap / overview strip for the hex matrix.
//!
//! A 40 px-wide strip between the hex content area and the scrollbar showing
//! the entire file as a column of coloured pixels.  Each pixel row represents
//! a block of bytes, coloured using the same scheme as the matrix.
//! Click or drag on the minimap to scroll.

use std::collections::{BTreeMap, BTreeSet};

use iced::advanced::Renderer as _;
use iced::{
    color, Background, Border, Color, Rectangle, Shadow,
};

use crate::ui::coloring::{default_byte_colors, ColorScheme};

/// Width of the minimap strip in pixels.
pub const MINIMAP_WIDTH: f32 = 40.0;

/// Size of search / cursor marker dots on the minimap.
pub const MINIMAP_MARKER_SIZE: f32 = 3.0;

/// Compute the minimap rectangle.
///
/// The minimap sits to the *left* of the vertical scrollbar.
///
/// | content | minimap | scrollbar |
/// |         |  40 px  |   10 px   |
pub fn minimap_rect(
    content_bounds: Rectangle,
    viewport_h: f32,
    minimap_w: f32,
    scrollbar_w: f32,
) -> Rectangle {
    Rectangle {
        x: content_bounds.x + content_bounds.width - scrollbar_w - minimap_w,
        y: content_bounds.y,
        width: minimap_w,
        height: viewport_h,
    }
}

/// Compute the viewport-overlay (thumb) rectangle on the minimap.
///
/// This is the translucent overlay showing which portion of the file is
/// currently visible in the hex matrix.  Its height and position mirror
/// how the scrollbar thumb works.
pub fn minimap_thumb_rect(
    mm_rect: Rectangle,
    scroll: f32,
    total_h: f32,
    viewport_h: f32,
) -> Rectangle {
    let h = thumb_height(mm_rect, total_h);
    let max_off = (total_h - viewport_h).max(1.0);
    let y = mm_rect.y + (scroll / max_off) * (mm_rect.height - h);
    Rectangle {
        x: mm_rect.x + 1.0,
        y: y.clamp(mm_rect.y, mm_rect.y + mm_rect.height - h),
        width: (mm_rect.width - 2.0).max(0.0),
        height: h,
    }
}

/// Convert a y-pixel position on the minimap to a scroll offset.
///
/// Used for click-to-scroll on the minimap track.
pub fn minimap_scroll_from_y(
    y: f32,
    mm_rect: Rectangle,
    total_h: f32,
    viewport_h: f32,
) -> f32 {
    if total_h <= viewport_h {
        return 0.0;
    }
    let frac = ((y - mm_rect.y) / mm_rect.height).clamp(0.0, 1.0);
    let max_scroll = (total_h - viewport_h).max(1.0);
    frac * max_scroll
}

/// Compute the scroll-offset change corresponding to a pixel delta on the
/// minimap.  Used during drag-scrolling on the minimap.
pub fn minimap_pixel_to_scroll(
    dy: f32,
    mm_rect: Rectangle,
    total_h: f32,
    viewport_h: f32,
) -> f32 {
    if total_h <= viewport_h {
        return 0.0;
    }
    let thumb_h = thumb_height(mm_rect, total_h);
    let travel = (mm_rect.height - thumb_h).max(1.0);
    let max_scroll = (total_h - viewport_h).max(1.0);
    dy * (max_scroll / travel)
}

/// Thumb height: the visible fraction of the minimap.
fn thumb_height(mm_rect: Rectangle, total_h: f32) -> f32 {
    if total_h <= 0.0 || mm_rect.height <= 0.0 {
        return mm_rect.height;
    }
    (mm_rect.height / total_h * mm_rect.height)
        .max(20.0)
        .min(mm_rect.height)
}

/// Draw the minimap overview strip.
///
/// Renders the byte-value heatmap, viewport overlay, cursor marker, and
/// search-match markers in the 40 px-wide strip between the hex content
/// and the vertical scrollbar.
#[allow(clippy::too_many_arguments)]
pub fn draw_minimap(
    renderer: &mut iced::Renderer,
    content_bounds: Rectangle,
    scroll: f32,
    total_h: f32,
    viewport_h: f32,
    bytes: &[u8],
    pattern_by_addr: &BTreeMap<u64, (usize, u8)>,
    alternate_patterns: &BTreeSet<usize>,
    dirty: &BTreeSet<u64>,
    vanilla_diff: &BTreeSet<u64>,
    color_scheme: ColorScheme,
    dim_nulls: bool,
    search_match_starts: &[u64],
    cursor_addr: u64,
    total_len: u64,
    _active: bool,
) {
    let right_strip = 10.0; // SCROLLBAR_THICKNESS
    let mm_rect = minimap_rect(content_bounds, viewport_h, MINIMAP_WIDTH, right_strip);

    // Track background (same colour as scrollbar track).
    renderer.fill_quad(
        quad(mm_rect),
        Background::Color(color!(0x141210)),
    );

    // ── Left border (1 px separator from content area) ──────────────
    renderer.fill_quad(
        quad(Rectangle {
            x: mm_rect.x,
            y: mm_rect.y,
            width: 1.0,
            height: mm_rect.height,
        }),
        Background::Color(color!(0x2a2218)),
    );

    // ── Pixel column ───────────────────────────────────────────────
    let h_px = mm_rect.height.max(1.0);
    let stride = if total_len <= 1 || h_px <= 1.0 {
        1
    } else {
        (total_len / h_px as u64).max(1)
    };
    for i in 0..(h_px as u64) {
        let addr = (i * stride).min(bytes.len().saturating_sub(1) as u64);
        let byte = bytes.get(addr as usize).copied().unwrap_or(0);

        let pixel_color = if let Some((_, color_idx)) = pattern_by_addr.get(&addr) {
            let mut c = pattern_bg(*color_idx);
            // Zebra-stripe: darken every other pattern in a repeated group.
            if let Some((pid, _)) = pattern_by_addr.get(&addr) {
                if alternate_patterns.contains(pid) {
                    c = Color::from_rgb(c.r * 0.5, c.g * 0.5, c.b * 0.5);
                }
            }
            c
        } else if dirty.contains(&addr) {
            color!(0x4a1f1a)
        } else if vanilla_diff.contains(&addr) {
            color!(0x232f1f)
        } else {
            let (fg, _) = default_byte_colors(color_scheme, byte, dim_nulls);
            let c = fg.unwrap_or(color!(0xd4cabd));
            Color::from_rgb(c.r * 0.55, c.g * 0.55, c.b * 0.55)
        };

        let y = mm_rect.y + i as f32;
        renderer.fill_quad(
            quad(Rectangle {
                x: mm_rect.x + 2.0,
                y,
                width: mm_rect.width - 4.0,
                height: 1.0,
            }),
            Background::Color(pixel_color),
        );
    }

    // ── Viewport overlay ───────────────────────────────────────────
    if total_h > viewport_h {
        let thumb = minimap_thumb_rect(mm_rect, scroll, total_h, viewport_h);
        renderer.fill_quad(
            quad(thumb),
            Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.12)),
        );
    }

    // ── Search-match markers (green dots) ──────────────────────────
    let marker_sz = MINIMAP_MARKER_SIZE;
    for &match_start in search_match_starts {
        let frac = if total_len > 0 {
            (match_start as f32) / (total_len as f32)
        } else {
            0.0
        };
        let my = mm_rect.y + frac * mm_rect.height;
        renderer.fill_quad(
            quad(Rectangle {
                x: mm_rect.x + (mm_rect.width - marker_sz) / 2.0,
                y: my - marker_sz / 2.0,
                width: marker_sz,
                height: marker_sz,
            }),
            Background::Color(color!(0x4a7a2a)),
        );
    }

    // ── Cursor-position marker (amber dot) ─────────────────────────
    if total_len > 0 {
        let frac = (cursor_addr as f32) / (total_len as f32);
        let cy = mm_rect.y + frac * mm_rect.height;
        renderer.fill_quad(
            quad(Rectangle {
                x: mm_rect.x + (mm_rect.width - marker_sz) / 2.0,
                y: cy - marker_sz / 2.0,
                width: marker_sz,
                height: marker_sz,
            }),
            Background::Color(color!(0xB97024)),
        );
    }
}

fn quad(bounds: Rectangle) -> iced::advanced::renderer::Quad {
    iced::advanced::renderer::Quad {
        bounds,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: true,
    }
}

fn pattern_bg(idx: u8) -> Color {
    match idx % 16 {
        0 => color!(0x1a3a4f),
        1 => color!(0x4f2e1a),
        2 => color!(0x1a4f2e),
        3 => color!(0x3b1a4f),
        4 => color!(0x4f4a1a),
        5 => color!(0x2e1a4f),
        6 => color!(0x4f1a1a),
        7 => color!(0x1a3b3b),
        8 => color!(0x3b2e1a),
        9 => color!(0x2e4f1a),
        10 => color!(0x4f2e3b),
        11 => color!(0x1a4f4f),
        12 => color!(0x4f251a),
        13 => color!(0x1a3b25),
        14 => color!(0x3b3b1a),
        15 => color!(0x251a4f),
        _ => color!(0x1a3a4f),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── minimap_rect ────────────────────────────────────────────────────

    #[test]
    fn minimap_rect_sits_between_content_and_scrollbar() {
        let cb = Rectangle {
            x: 0.0,
            y: 16.0,
            width: 800.0,
            height: 284.0,
        };
        let r = minimap_rect(cb, 284.0, 40.0, 10.0);
        assert_eq!(r.x, 750.0);
        assert_eq!(r.y, 16.0);
        assert_eq!(r.width, 40.0);
        assert_eq!(r.height, 284.0);
    }

    #[test]
    fn minimap_rect_zero_viewport() {
        let cb = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 0.0,
        };
        let r = minimap_rect(cb, 0.0, 40.0, 10.0);
        assert_eq!(r.height, 0.0);
    }

    // ── minimap_thumb_rect ──────────────────────────────────────────────

    #[test]
    fn minimap_thumb_shows_visible_portion() {
        let mm = Rectangle {
            x: 750.0,
            y: 16.0,
            width: 40.0,
            height: 284.0,
        };
        // File is 4× viewport height: thumb should be ~71 px tall
        // (284 / (284*4) * 284 ≈ 71), positioned at top when scroll = 0.
        let r = minimap_thumb_rect(mm, 0.0, 284.0 * 4.0, 284.0);
        assert_eq!(r.x, 751.0);
        assert_eq!(r.y, 16.0);
        assert_eq!(r.width, 38.0);
        assert!((r.height - 71.0).abs() < 1.0, "height {}", r.height);
    }

    #[test]
    fn minimap_thumb_scrolled_mid_file() {
        let mm = Rectangle {
            x: 750.0,
            y: 16.0,
            width: 40.0,
            height: 284.0,
        };
        // File is 4× viewport_h. Total scrollable = 284*4 - 284 = 852.
        // Scroll = 426 (midpoint). Thumb should be near the middle.
        let r = minimap_thumb_rect(mm, 426.0, 284.0 * 4.0, 284.0);
        assert_eq!(r.x, 751.0);
        let expected_y = 16.0 + (284.0 - 71.0) / 2.0;
        assert!((r.y - expected_y).abs() < 1.0, "y {} != {}", r.y, expected_y);
        assert_eq!(r.width, 38.0);
    }

    #[test]
    fn minimap_thumb_scrolled_to_end() {
        let mm = Rectangle {
            x: 750.0,
            y: 16.0,
            width: 40.0,
            height: 284.0,
        };
        // Scroll = 852 (max). Thumb at bottom.
        let max_scroll = 284.0 * 4.0 - 284.0;
        let r = minimap_thumb_rect(mm, max_scroll, 284.0 * 4.0, 284.0);
        assert!((r.y - (mm.y + mm.height - r.height)).abs() < 1.0);
    }

    #[test]
    fn minimap_thumb_fills_when_content_fits() {
        let mm = Rectangle {
            x: 750.0,
            y: 16.0,
            width: 40.0,
            height: 284.0,
        };
        let r = minimap_thumb_rect(mm, 0.0, 100.0, 284.0);
        assert_eq!(r.height, mm.height);
        assert_eq!(r.y, 16.0);
    }

    #[test]
    fn minimap_thumb_empty_file() {
        let mm = Rectangle {
            x: 750.0,
            y: 16.0,
            width: 40.0,
            height: 284.0,
        };
        let r = minimap_thumb_rect(mm, 0.0, 0.0, 284.0);
        assert_eq!(r.height, 284.0);
    }

    // ── minimap_scroll_from_y ───────────────────────────────────────────

    #[test]
    fn minimap_scroll_from_y_top_equals_zero() {
        let mm = Rectangle {
            x: 750.0,
            y: 16.0,
            width: 40.0,
            height: 284.0,
        };
        let s = minimap_scroll_from_y(16.0, mm, 284.0 * 4.0, 284.0);
        assert_eq!(s, 0.0);
    }

    #[test]
    fn minimap_scroll_from_y_bottom_equals_max() {
        let mm = Rectangle {
            x: 750.0,
            y: 16.0,
            width: 40.0,
            height: 284.0,
        };
        let s = minimap_scroll_from_y(300.0, mm, 284.0 * 4.0, 284.0);
        let max_scroll = 284.0 * 4.0 - 284.0;
        assert!((s - max_scroll).abs() < 1.0, "scroll {} != {}", s, max_scroll);
    }

    #[test]
    fn minimap_scroll_from_y_midpoint() {
        let mm = Rectangle {
            x: 750.0,
            y: 16.0,
            width: 40.0,
            height: 284.0,
        };
        let s = minimap_scroll_from_y(16.0 + 284.0 / 2.0, mm, 284.0 * 4.0, 284.0);
        let max_scroll = 284.0 * 4.0 - 284.0;
        assert!((s - max_scroll / 2.0).abs() < 1.0, "scroll {} != {}", s, max_scroll / 2.0);
    }

    #[test]
    fn minimap_scroll_from_y_clamps_outside() {
        let mm = Rectangle {
            x: 750.0,
            y: 16.0,
            width: 40.0,
            height: 284.0,
        };
        // y above minimap → 0
        assert_eq!(minimap_scroll_from_y(0.0, mm, 284.0 * 4.0, 284.0), 0.0);
        // y way below → max
        let s = minimap_scroll_from_y(9999.0, mm, 284.0 * 4.0, 284.0);
        assert!((s - (284.0 * 4.0 - 284.0)).abs() < 1.0);
    }

    #[test]
    fn minimap_scroll_from_y_no_scroll_needed() {
        let mm = Rectangle {
            x: 750.0,
            y: 16.0,
            width: 40.0,
            height: 284.0,
        };
        // total_h < viewport_h → no scrolling possible
        assert_eq!(minimap_scroll_from_y(200.0, mm, 100.0, 284.0), 0.0);
    }

    // ── minimap_pixel_to_scroll ─────────────────────────────────────────

    #[test]
    fn minimap_pixel_to_scroll_positive_delta() {
        let mm = Rectangle {
            x: 750.0,
            y: 16.0,
            width: 40.0,
            height: 284.0,
        };
        let delta = minimap_pixel_to_scroll(10.0, mm, 284.0 * 4.0, 284.0);
        // Dragging 10 px down in a file 4× viewport should advance ~38 px.
        assert!(delta > 30.0 && delta < 50.0, "delta={}", delta);
    }

    #[test]
    fn minimap_pixel_to_scroll_negative_delta() {
        let mm = Rectangle {
            x: 750.0,
            y: 16.0,
            width: 40.0,
            height: 284.0,
        };
        let delta = minimap_pixel_to_scroll(-10.0, mm, 284.0 * 4.0, 284.0);
        assert!(delta < -30.0 && delta > -50.0, "delta={}", delta);
    }

    #[test]
    fn minimap_pixel_to_scroll_no_scroll_needed() {
        let mm = Rectangle {
            x: 750.0,
            y: 16.0,
            width: 40.0,
            height: 284.0,
        };
        assert_eq!(minimap_pixel_to_scroll(50.0, mm, 100.0, 284.0), 0.0);
    }

}
