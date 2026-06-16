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

/// Compute minimap pixel colors by block-averaging the file.
///
/// Each pixel row represents a block of bytes.  The block color is
/// determined by priority: pattern → dirty → diff → mean byte value.
///
/// This is the cacheable part of minimap rendering — separated from the
/// actual draw call so the caller can cache the result across frames.
#[allow(clippy::too_many_arguments)]
pub fn compute_block_pixels(
    bytes: &[u8],
    total_len: u64,
    h_px: u32,
    pattern_by_addr: &BTreeMap<u64, (usize, u8)>,
    alternate_patterns: &BTreeSet<usize>,
    dirty: &BTreeSet<u64>,
    vanilla_diff: &BTreeSet<u64>,
    color_scheme: ColorScheme,
    dim_nulls: bool,
) -> Vec<Color> {
    if total_len == 0 || h_px == 0 {
        return Vec::new();
    }
    let stride = total_len.div_ceil(h_px as u64);
    let h = h_px as usize;
    let last_valid = bytes.len().saturating_sub(1) as u64;
    let mut pixels = Vec::with_capacity(h);

    for i in 0..h {
        let block_start = (i as u64).saturating_mul(stride).min(total_len);
        let block_end =
            ((i as u64 + 1).saturating_mul(stride)).min(total_len).min(bytes.len() as u64);
        let block_len = block_end - block_start;

        // ── Sample 3 positions: 25 %, 50 %, 75 % of the block ─────────
        // This gives better pattern/dirty/diff detection than a single
        // midpoint, while the variance between samples adds a natural
        // brightness heatmap (uniform blocks recede, varied blocks pop).
        let get = |addr: u64| bytes.get(addr.min(last_valid) as usize).copied().unwrap_or(0u8);

        let p0 = if block_len > 0 { block_start + block_len / 4 } else { block_start };
        let p1 = if block_len >= 2 { block_start + block_len / 2 } else { p0 };
        let p2 = if block_len >= 3 { block_start + block_len * 3 / 4 } else { p1 };
        let v0 = get(p0);
        let v1 = get(p1);
        let v2 = get(p2);

        // Priority: any sample in pattern → full-brightness pattern_bg.
        let mut pattern_hit = None;
        let mut dirty_hit = false;
        let mut diff_hit = false;
        for &addr in &[p0, p1, p2] {
            if pattern_hit.is_none() {
                pattern_hit = pattern_by_addr.get(&addr);
            }
            if !dirty_hit {
                dirty_hit = dirty.contains(&addr);
            }
            if !diff_hit {
                diff_hit = vanilla_diff.contains(&addr);
            }
        }

        let pixel = if let Some(&(pid, color_idx)) = pattern_hit {
            let mut c = pattern_bg(color_idx);
            if alternate_patterns.contains(&pid) {
                c.r *= 0.5;
                c.g *= 0.5;
                c.b *= 0.5;
            }
            c
        } else if dirty_hit {
            color!(0x4a1f1a)
        } else if diff_hit {
            color!(0x232f1f)
        } else {
            // Variance-based brightness: uniform blocks stay dim, mixed
            // blocks bloom to full brightness.  Normalised so max variance
            // (~14 450 for alternating 0x00/0xFF) maps to brightness ≈ 1.
            let vf0 = v0 as f32;
            let vf1 = v1 as f32;
            let vf2 = v2 as f32;
            let mean_f = (vf0 + vf1 + vf2) / 3.0;
            let var = ((vf0 - mean_f).powi(2) + (vf1 - mean_f).powi(2) + (vf2 - mean_f).powi(2))
                / 3.0;
            let norm = (var / 15_000.0).clamp(0.0, 1.0);
            let brightness = 0.30 + norm * 0.70;

            let mean_byte = mean_f.round() as u8;
            let (fg, _) = default_byte_colors(color_scheme, mean_byte, dim_nulls);
            let c = fg.unwrap_or(color!(0xd4cabd));
            Color::from_rgb(
                c.r * 0.55 * brightness,
                c.g * 0.55 * brightness,
                c.b * 0.55 * brightness,
            )
        };
        pixels.push(pixel);
    }
    pixels
}

/// Draw the minimap overview strip.
///
/// Renders the pre-computed pixel column, viewport overlay, cursor marker,
/// and search-match markers in the 40 px-wide strip between the hex content
/// and the vertical scrollbar.
///
/// `pixels` should contain one colour per pixel row (as returned by
/// [`compute_block_pixels`]).
#[allow(clippy::too_many_arguments)]
pub fn draw_minimap(
    renderer: &mut iced::Renderer,
    content_bounds: Rectangle,
    scroll: f32,
    total_h: f32,
    viewport_h: f32,
    pixels: &[Color],
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
    for (i, &c) in pixels.iter().enumerate() {
        let y = mm_rect.y + i as f32;
        renderer.fill_quad(
            quad(Rectangle {
                x: mm_rect.x + 2.0,
                y,
                width: mm_rect.width - 4.0,
                height: 1.0,
            }),
            Background::Color(c),
        );
    }

    // ── Viewport overlay ───────────────────────────────────────────
    if total_h > viewport_h {
        let thumb = minimap_thumb_rect(mm_rect, scroll, total_h, viewport_h);
        renderer.fill_quad(
            quad(thumb),
            Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.12)),
        );
        // Brighter 1 px border so the overlay's edges are visible on
        // both dark and light byte regions.
        let bdr = Color::from_rgba(1.0, 1.0, 1.0, 0.35);
        // Top
        renderer.fill_quad(
            quad(Rectangle {
                x: thumb.x, y: thumb.y,
                width: thumb.width, height: 1.0,
            }),
            Background::Color(bdr),
        );
        // Bottom
        renderer.fill_quad(
            quad(Rectangle {
                x: thumb.x,
                y: thumb.y + thumb.height - 1.0,
                width: thumb.width,
                height: 1.0,
            }),
            Background::Color(bdr),
        );
        // Left
        renderer.fill_quad(
            quad(Rectangle {
                x: thumb.x, y: thumb.y,
                width: 1.0, height: thumb.height,
            }),
            Background::Color(bdr),
        );
        // Right
        renderer.fill_quad(
            quad(Rectangle {
                x: thumb.x + thumb.width - 1.0,
                y: thumb.y,
                width: 1.0,
                height: thumb.height,
            }),
            Background::Color(bdr),
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

/// Cached minimap pixel colors to avoid re-scanning the file every frame.
#[derive(Clone)]
pub struct MinimapCache {
    /// One pixel color per pixel row.
    pub pixels: Vec<Color>,
    /// File length when cached.
    pub total_len: u64,
    /// Number of pixel rows when cached.
    pub h_px: u32,
    /// Color scheme when cached.
    pub color_scheme: ColorScheme,
    /// Dim-nulls setting when cached.
    pub dim_nulls: bool,
    /// Hash of pattern_by_addr contents (*not* the full map — just a
    /// checksum so we can cheaply detect changes).
    pub pattern_hash: u64,
    /// Number of dirty addresses when cached.
    pub dirty_count: usize,
    /// Number of diff addresses when cached.
    pub diff_count: usize,
}

/// Quick (non-cryptographic) hash of a pattern_by_addr map.
///
/// Used to cheaply detect whether the pattern set has changed since the
/// minimap cache was computed.
pub fn pattern_hash(patterns: &BTreeMap<u64, (usize, u8)>) -> u64 {
    let mut h: u64 = 0;
    for (&addr, &(pid, col)) in patterns {
        h = h.wrapping_mul(31).wrapping_add(addr);
        h = h.wrapping_mul(31).wrapping_add(pid as u64);
        h = h.wrapping_mul(31).wrapping_add(col as u64);
    }
    h
}

/// Check whether a cached minimap pixel array is still valid for the
/// current file state.
#[allow(clippy::too_many_arguments)]
pub fn minimap_cache_valid(
    cache: &MinimapCache,
    total_len: u64,
    h_px: u32,
    color_scheme: ColorScheme,
    dim_nulls: bool,
    pattern_by_addr: &BTreeMap<u64, (usize, u8)>,
    dirty: &BTreeSet<u64>,
    vanilla_diff: &BTreeSet<u64>,
) -> bool {
    cache.total_len == total_len
        && cache.h_px == h_px
        && cache.color_scheme == color_scheme
        && cache.dim_nulls == dim_nulls
        && cache.pattern_hash == pattern_hash(pattern_by_addr)
        && cache.dirty_count == dirty.len()
        && cache.diff_count == vanilla_diff.len()
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

    // ── compute_block_pixels —──────────────────────────────────────────

    #[test]
    fn compute_block_pixels_empty_file() {
        let pixels = compute_block_pixels(
            &[], 0, 10,
            &BTreeMap::new(), &BTreeSet::new(),
            &BTreeSet::new(), &BTreeSet::new(),
            ColorScheme::Monochrome, false,
        );
        assert!(pixels.is_empty());
    }

    #[test]
    fn compute_block_pixels_zero_height() {
        let pixels = compute_block_pixels(
            &[0xFF; 100], 100, 0,
            &BTreeMap::new(), &BTreeSet::new(),
            &BTreeSet::new(), &BTreeSet::new(),
            ColorScheme::Monochrome, false,
        );
        assert!(pixels.is_empty());
    }

    #[test]
    fn compute_block_pixels_uniform_bytes() {
        // Uniform 0xFF → no variance → brightness = 0.30 (minimum).
        let bytes = [0xFFu8; 200];
        let pixels = compute_block_pixels(
            &bytes, 200, 10,
            &BTreeMap::new(), &BTreeSet::new(),
            &BTreeSet::new(), &BTreeSet::new(),
            ColorScheme::Monochrome, false,
        );
        assert_eq!(pixels.len(), 10);
        let brightness = 0.30_f32;
        let expected = Color::from_rgb(
            0xD4 as f32 / 255.0 * 0.55 * brightness,
            0xCA as f32 / 255.0 * 0.55 * brightness,
            0xBD as f32 / 255.0 * 0.55 * brightness,
        );
        for (i, &p) in pixels.iter().enumerate() {
            assert!((p.r - expected.r).abs() < 0.0001, "pixel {i}: r");
            assert!((p.g - expected.g).abs() < 0.0001, "pixel {i}: g");
            assert!((p.b - expected.b).abs() < 0.0001, "pixel {i}: b");
        }
    }

    #[test]
    fn compute_block_pixels_pattern_dominates() {
        let bytes = [0x00u8; 100];
        let mut patterns = BTreeMap::new();
        patterns.insert(5, (0usize, 3u8));
        let pixels = compute_block_pixels(
            &bytes, 100, 10,
            &patterns, &BTreeSet::new(),
            &BTreeSet::new(), &BTreeSet::new(),
            ColorScheme::Monochrome, false,
        );
        assert_eq!(pixels.len(), 10);
        let expected = super::pattern_bg(3);
        assert!((pixels[0].r - expected.r).abs() < 0.0001, "r");
        assert!((pixels[0].g - expected.g).abs() < 0.0001, "g");
        assert!((pixels[0].b - expected.b).abs() < 0.0001, "b");
    }

    #[test]
    fn compute_block_pixels_zebra_alternate_darkens() {
        let bytes = [0x00u8; 100];
        let mut patterns = BTreeMap::new();
        patterns.insert(5, (17usize, 2u8));
        let mut alternate = BTreeSet::new();
        alternate.insert(17);
        let pixels = compute_block_pixels(
            &bytes, 100, 10,
            &patterns, &alternate,
            &BTreeSet::new(), &BTreeSet::new(),
            ColorScheme::Monochrome, false,
        );
        assert_eq!(pixels.len(), 10);
        let base = super::pattern_bg(2);
        let expected = Color::from_rgb(base.r * 0.5, base.g * 0.5, base.b * 0.5);
        assert!((pixels[0].r - expected.r).abs() < 0.0001, "r");
        assert!((pixels[0].g - expected.g).abs() < 0.0001, "g");
        assert!((pixels[0].b - expected.b).abs() < 0.0001, "b");
    }

    #[test]
    fn compute_block_pixels_dirty_over_diff() {
        let bytes = [0x00u8; 100];
        let mut dirty = BTreeSet::new();
        dirty.insert(5);
        let mut diff = BTreeSet::new();
        diff.insert(5);
        let pixels = compute_block_pixels(
            &bytes, 100, 10,
            &BTreeMap::new(), &BTreeSet::new(),
            &dirty, &diff,
            ColorScheme::Monochrome, false,
        );
        assert_eq!(pixels.len(), 10);
        let dirty_c = Color::from_rgb(0x4A as f32 / 255.0, 0x1F as f32 / 255.0, 0x1A as f32 / 255.0);
        assert!((pixels[0].r - dirty_c.r).abs() < 0.0001, "r");
        assert!((pixels[0].g - dirty_c.g).abs() < 0.0001, "g");
        assert!((pixels[0].b - dirty_c.b).abs() < 0.0001, "b");
    }

    #[test]
    fn compute_block_pixels_diff_when_no_pattern_or_dirty() {
        let bytes = [0x00u8; 100];
        let mut diff = BTreeSet::new();
        diff.insert(5);
        let pixels = compute_block_pixels(
            &bytes, 100, 10,
            &BTreeMap::new(), &BTreeSet::new(),
            &BTreeSet::new(), &diff,
            ColorScheme::Monochrome, false,
        );
        assert_eq!(pixels.len(), 10);
        let diff_c = Color::from_rgb(0x23 as f32 / 255.0, 0x2F as f32 / 255.0, 0x1F as f32 / 255.0);
        assert!((pixels[0].r - diff_c.r).abs() < 0.0001, "r");
        assert!((pixels[0].g - diff_c.g).abs() < 0.0001, "g");
        assert!((pixels[0].b - diff_c.b).abs() < 0.0001, "b");
    }

    #[test]
    fn compute_block_pixels_priority_pattern_over_dirty_and_diff() {
        let bytes = [0x00u8; 100];
        let mut patterns = BTreeMap::new();
        patterns.insert(5, (0usize, 1u8));
        let mut dirty = BTreeSet::new();
        dirty.insert(5);
        let mut diff = BTreeSet::new();
        diff.insert(5);
        let pixels = compute_block_pixels(
            &bytes, 100, 10,
            &patterns, &BTreeSet::new(),
            &dirty, &diff,
            ColorScheme::Monochrome, false,
        );
        assert_eq!(pixels.len(), 10);
        let expected = super::pattern_bg(1);
        assert!((pixels[0].r - expected.r).abs() < 0.0001, "r");
        assert!((pixels[0].g - expected.g).abs() < 0.0001, "g");
        assert!((pixels[0].b - expected.b).abs() < 0.0001, "b");
    }

    #[test]
    fn compute_block_pixels_stride_ceiling_covers_last_block() {
        let bytes = vec![0x42u8; 101];
        let pixels = compute_block_pixels(
            &bytes, 101, 10,
            &BTreeMap::new(), &BTreeSet::new(),
            &BTreeSet::new(), &BTreeSet::new(),
            ColorScheme::Monochrome, false,
        );
        assert_eq!(pixels.len(), 10);
        let _ = pixels[9]; // last pixel should be valid
    }

    // ── pattern_hash —─────────────────────────────────────────────────

    #[test]
    fn pattern_hash_empty_map() {
        assert_eq!(pattern_hash(&BTreeMap::new()), 0);
    }

    #[test]
    fn pattern_hash_different_maps_different_hashes() {
        let mut a = BTreeMap::new();
        a.insert(0, (1usize, 2u8));
        let mut b = BTreeMap::new();
        b.insert(0, (1usize, 3u8)); // different color_idx
        assert_ne!(pattern_hash(&a), pattern_hash(&b));
    }

    // ── minimap_cache_valid —────────────────────────────────────────────

    #[test]
    fn minimap_cache_valid_matches() {
        let mut patterns = BTreeMap::new();
        patterns.insert(10, (0usize, 1u8));
        let cache = MinimapCache {
            pixels: vec![Color::WHITE],
            total_len: 100,
            h_px: 10,
            color_scheme: ColorScheme::Monochrome,
            dim_nulls: false,
            pattern_hash: pattern_hash(&patterns),
            dirty_count: 5,
            diff_count: 3,
        };
        let mut dirty = BTreeSet::new();
        for i in 0..5 { dirty.insert(i); }
        let mut diff = BTreeSet::new();
        for i in 0..3 { diff.insert(100 + i); }
        assert!(minimap_cache_valid(
            &cache, 100, 10, ColorScheme::Monochrome, false,
            &patterns, &dirty, &diff,
        ));
    }

    #[test]
    fn minimap_cache_valid_detects_size_change() {
        let cache = MinimapCache {
            pixels: vec![Color::WHITE],
            total_len: 100,
            h_px: 10,
            color_scheme: ColorScheme::Monochrome,
            dim_nulls: false,
            pattern_hash: 0,
            dirty_count: 0,
            diff_count: 0,
        };
        assert!(!minimap_cache_valid(&cache, 200, 10, ColorScheme::Monochrome, false, &BTreeMap::new(), &BTreeSet::new(), &BTreeSet::new()));
    }

    #[test]
    fn minimap_cache_valid_detects_scheme_change() {
        let cache = MinimapCache {
            pixels: vec![Color::WHITE],
            total_len: 100,
            h_px: 10,
            color_scheme: ColorScheme::Monochrome,
            dim_nulls: false,
            pattern_hash: 0,
            dirty_count: 0,
            diff_count: 0,
        };
        assert!(!minimap_cache_valid(&cache, 100, 10, ColorScheme::Nybble, false, &BTreeMap::new(), &BTreeSet::new(), &BTreeSet::new()));
    }

    #[test]
    fn minimap_cache_valid_detects_dirty_count_change() {
        let cache = MinimapCache {
            pixels: vec![Color::WHITE],
            total_len: 100,
            h_px: 10,
            color_scheme: ColorScheme::Monochrome,
            dim_nulls: false,
            pattern_hash: 0,
            dirty_count: 5,
            diff_count: 0,
        };
        let dirty: BTreeSet<u64> = [1, 2, 3].into_iter().collect();
        assert!(!minimap_cache_valid(&cache, 100, 10, ColorScheme::Monochrome, false, &BTreeMap::new(), &dirty, &BTreeSet::new()));
    }

    // ── compute_block_pixels —─────────────────────────────────────────

    #[test]
    fn compute_block_pixels_multi_sample_and_variance() {
        // 20 bytes, 2 pixels, stride = 10.
        // Block 0 [0..10): bytes 0-4 = 0x00, bytes 5-9 = 0xFF.
        //   Samples: 25%=2(0x00), 50%=5(0xFF), 75%=7(0xFF)
        //   Mean = 170, variance ≈ 14 450, brightness ≈ 0.974.
        // Block 1 [10..20): all 0x00.
        //   Samples: 25%=12(0x00), 50%=15(0x00), 75%=17(0x00)
        //   Mean = 0, variance = 0, brightness = 0.30.
        let mut bytes = [0x00u8; 20];
        for i in 5..10 {
            bytes[i] = 0xFF;
        }
        let pixels = compute_block_pixels(
            &bytes, 20, 2,
            &BTreeMap::new(), &BTreeSet::new(),
            &BTreeSet::new(), &BTreeSet::new(),
            ColorScheme::Monochrome, false,
        );
        assert_eq!(pixels.len(), 2);

        // Brightness computation (replicated from the implementation for
        // self-consistency — catches logic errors, not float drift).
        fn brightness_for(v0: u8, v1: u8, v2: u8) -> f32 {
            let (vf0, vf1, vf2) = (v0 as f32, v1 as f32, v2 as f32);
            let mean_f = (vf0 + vf1 + vf2) / 3.0;
            let var =
                ((vf0 - mean_f).powi(2) + (vf1 - mean_f).powi(2) + (vf2 - mean_f).powi(2)) / 3.0;
            0.30 + (var / 15_000.0).clamp(0.0, 1.0) * 0.70
        }

        // Pixel 0: mixed block → high variance → nearly full brightness.
        let b0 = brightness_for(0x00, 0xFF, 0xFF);
        assert!((b0 - 0.974).abs() < 0.005, "pixel 0 brightness {}", b0);
        let mean0 = 170u8;
        let (fg0, _) = default_byte_colors(ColorScheme::Monochrome, mean0, false);
        let base0 = fg0.unwrap();
        let expected0 =
            Color::from_rgb(base0.r * 0.55 * b0, base0.g * 0.55 * b0, base0.b * 0.55 * b0);
        let d0 = (pixels[0].r - expected0.r).abs()
            + (pixels[0].g - expected0.g).abs()
            + (pixels[0].b - expected0.b).abs();
        assert!(d0 < 0.001, "pixel 0: diff={d0}");

        // Pixel 1: uniform block → zero variance → dim.
        let b1 = brightness_for(0x00, 0x00, 0x00);
        assert!((b1 - 0.30).abs() < 0.001, "pixel 1 brightness {}", b1);
        let mean1 = 0u8;
        let (fg1, _) = default_byte_colors(ColorScheme::Monochrome, mean1, false);
        let base1 = fg1.unwrap();
        let expected1 =
            Color::from_rgb(base1.r * 0.55 * b1, base1.g * 0.55 * b1, base1.b * 0.55 * b1);
        let d1 = (pixels[1].r - expected1.r).abs()
            + (pixels[1].g - expected1.g).abs()
            + (pixels[1].b - expected1.b).abs();
        assert!(d1 < 0.001, "pixel 1: diff={d1}");
    }

    #[test]
    fn compute_block_pixels_variance_mixed_brighter_than_uniform() {
        // Two blocks with the same mean byte but different variance.
        // The mixed block must be visibly brighter.
        let mut mixed = [0x00u8; 20];
        for b in &mut mixed[5..15] {
            *b = 0xFF;
        } // alternating block: 0s followed by 0xFFs → high variance
        let mut uniform = [0x80u8; 20]; // all same → zero variance

        let p_mixed = compute_block_pixels(
            &mixed, 20, 2,
            &BTreeMap::new(), &BTreeSet::new(),
            &BTreeSet::new(), &BTreeSet::new(),
            ColorScheme::Monochrome, false,
        );
        let p_uniform = compute_block_pixels(
            &uniform, 20, 2,
            &BTreeMap::new(), &BTreeSet::new(),
            &BTreeSet::new(), &BTreeSet::new(),
            ColorScheme::Monochrome, false,
        );
        // Mixed block should have at least one pixel brighter than uniform.
        let lum_mixed = p_mixed.iter().map(|c| c.r + c.g + c.b).sum::<f32>();
        let lum_uniform = p_uniform.iter().map(|c| c.r + c.g + c.b).sum::<f32>();
        assert!(
            lum_mixed > lum_uniform + 0.5,
            "mixed lum {lum_mixed} should be > uniform lum {lum_uniform}"
        );
    }

}
