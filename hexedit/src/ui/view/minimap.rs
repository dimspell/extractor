//! Minimap / overview strip for the hex matrix.
//!
//! A 36 px-wide strip between the hex content area and the scrollbar showing
//! three independent columns of coloured pixels per viewport row.  Each
//! column represents a distinct file block, giving 3× the vertical
//! resolution of a single-column minimap.  Click or drag to scroll.
//!
//! Layout: `| content | col0(11) | col1(11) | col2(11) | scrollbar |`

use std::collections::{BTreeMap, BTreeSet};

use iced::advanced::Renderer as _;
use iced::{
    color, Background, Border, Color, Rectangle, Shadow,
};

use crate::ui::coloring::{default_byte_colors, ColorScheme};

/// Width of the minimap strip in pixels.
pub const MINIMAP_WIDTH: f32 = 36.0;

/// Number of sub-columns per pixel row.
pub const MINIMAP_COLS: usize = 3;

/// Width of each sub-column in pixels.
pub const MINIMAP_COL_WIDTH: f32 = (MINIMAP_WIDTH - 3.0) / MINIMAP_COLS as f32; // 11.0

/// Compute the minimap rectangle.
///
/// The minimap sits to the *left* of the vertical scrollbar.
///
/// | content | minimap | scrollbar |
/// |         |  36 px  |   10 px   |
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
) -> [Vec<Color>; 3] {
    if total_len == 0 || h_px == 0 {
        return Default::default();
    }
    let n = h_px as usize;
    let total_blocks = (MINIMAP_COLS as u64) * h_px as u64;
    let stride = total_len.div_ceil(total_blocks);
    let last_valid = bytes.len().saturating_sub(1) as u64;

    let mut cols: [Vec<Color>; 3] = [
        Vec::with_capacity(n),
        Vec::with_capacity(n),
        Vec::with_capacity(n),
    ];

    for block_idx in 0..total_blocks as usize {
        let start = (block_idx as u64).saturating_mul(stride).min(total_len);
        let end = ((block_idx as u64 + 1).saturating_mul(stride))
            .min(total_len)
            .min(bytes.len() as u64);
        cols[block_idx % MINIMAP_COLS].push(block_color(
            start, end, bytes, last_valid,
            pattern_by_addr, alternate_patterns, dirty, vanilla_diff,
            color_scheme, dim_nulls,
        ));
    }
    cols
}

/// Per-block colour logic shared by all 3 sub-columns.  The same priority
/// applies: pattern → dirty → diff → mean+variance brightness.
#[allow(clippy::too_many_arguments)]
fn block_color(
    block_start: u64,
    block_end: u64,
    bytes: &[u8],
    last_valid: u64,
    pattern_by_addr: &BTreeMap<u64, (usize, u8)>,
    alternate_patterns: &BTreeSet<usize>,
    dirty: &BTreeSet<u64>,
    vanilla_diff: &BTreeSet<u64>,
    color_scheme: ColorScheme,
    dim_nulls: bool,
) -> Color {
    let block_len = block_end - block_start;

    let has_pattern = if block_len > 0 {
        pattern_by_addr.range(block_start..block_end).next()
            .map(|(&addr, &(pid, ci))| (addr, (pid, ci)))
    } else {
        None
    };
    let has_dirty = block_len > 0 && dirty.range(block_start..block_end).next().is_some();
    let has_diff = block_len > 0 && vanilla_diff.range(block_start..block_end).next().is_some();

    let get = |addr: u64| bytes.get(addr.min(last_valid) as usize).copied().unwrap_or(0u8);

    let p0 = if block_len > 0 { block_start + block_len / 4 } else { block_start };
    let p1 = if block_len >= 2 { block_start + block_len / 2 } else { p0 };
    let p2 = if block_len >= 3 { block_start + block_len * 3 / 4 } else { p1 };
    let v0 = get(p0);
    let v1 = get(p1);
    let v2 = get(p2);

    if let Some((_addr, (pid, color_idx))) = has_pattern {
        let mut c = pattern_bg(color_idx);
        if alternate_patterns.contains(&pid) {
            c.r *= 0.5;
            c.g *= 0.5;
            c.b *= 0.5;
        }
        c
    } else if has_dirty {
        color!(0x4a1f1a)
    } else if has_diff {
        color!(0x232f1f)
    } else {
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
    }
}

/// Draw the minimap overview strip.
///
/// Renders 3 independent sub-columns side by side, each showing a distinct
/// ⅓ of the file blocks.  Viewport overlay, selection-range band, and
/// cursor-position marker are drawn on top.
#[allow(clippy::too_many_arguments)]
pub fn draw_minimap(
    renderer: &mut iced::Renderer,
    content_bounds: Rectangle,
    scroll: f32,
    total_h: f32,
    viewport_h: f32,
    columns: &[Vec<Color>; 3],
    selection_start: u64,
    selection_end: u64,
    cursor_addr: u64,
    total_len: u64,
    _active: bool,
) {
    let right_strip = 10.0;
    let mm_rect = minimap_rect(content_bounds, viewport_h, MINIMAP_WIDTH, right_strip);

    renderer.fill_quad(quad(mm_rect), Background::Color(color!(0x141210)));

    renderer.fill_quad(
        quad(Rectangle {
            x: mm_rect.x,
            y: mm_rect.y,
            width: 1.0,
            height: mm_rect.height,
        }),
        Background::Color(color!(0x2a2218)),
    );

    // ── Three sub-columns (batched by colour) ──────────────────────
    let inner_x = mm_rect.x + 2.0;
    for (col, col_data) in columns.iter().enumerate() {
        let cx = inner_x + col as f32 * MINIMAP_COL_WIDTH;
        let mut i = 0;
        while i < col_data.len() {
            let c = col_data[i];
            let mut j = i + 1;
            while j < col_data.len() && col_data[j] == c {
                j += 1;
            }
            renderer.fill_quad(
                quad(Rectangle {
                    x: cx,
                    y: mm_rect.y + i as f32,
                    width: MINIMAP_COL_WIDTH,
                    height: (j - i) as f32,
                }),
                Background::Color(c),
            );
            i = j;
        }
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

    // ── Selection-range band (translucent blue) ─────────────────────
    // When a multi-byte selection is active, show its file-span as a
    // translucent overlay spanning the full minimap width.
    if total_len > 0 && selection_start != selection_end {
        let sel_lo = selection_start.min(selection_end);
        let sel_hi = selection_start.max(selection_end);
        let frac_lo = (sel_lo as f32) / (total_len as f32);
        let frac_hi = (sel_hi as f32) / (total_len as f32);
        let band_y = mm_rect.y + frac_lo * mm_rect.height;
        let band_h = (frac_hi - frac_lo) * mm_rect.height;
        renderer.fill_quad(
            quad(Rectangle {
                x: mm_rect.x + 1.0,
                y: band_y,
                width: mm_rect.width - 1.0,
                height: band_h.max(1.0),
            }),
            Background::Color(Color::from_rgba(0.25, 0.45, 0.80, 0.20)),
        );
    }

    // ── Cursor-position marker (amber dot) ─────────────────────────
    if total_len > 0 {
        let frac = (cursor_addr as f32) / (total_len as f32);
        let cy = mm_rect.y + frac * mm_rect.height;
        renderer.fill_quad(
            quad(Rectangle {
                x: mm_rect.x + (mm_rect.width - 3.0) / 2.0,
                y: cy - 1.5,
                width: 3.0,
                height: 3.0,
            }),
            Background::Color(color!(0xB97024)),
        );
    }
}

/// Cached minimap pixel colors to avoid re-scanning the file every frame.
#[derive(Clone)]
pub struct MinimapCache {
    /// Three pixel arrays, one per sub-column.  Each has `h_px` entries.
    pub columns: [Vec<Color>; 3],
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
        let cols = compute_block_pixels(
            &[], 0, 10,
            &BTreeMap::new(), &BTreeSet::new(),
            &BTreeSet::new(), &BTreeSet::new(),
            ColorScheme::Monochrome, false,
        );
        assert!(cols[0].is_empty() && cols[1].is_empty() && cols[2].is_empty());
    }

    #[test]
    fn compute_block_pixels_zero_height() {
        let cols = compute_block_pixels(
            &[0xFF; 100], 100, 0,
            &BTreeMap::new(), &BTreeSet::new(),
            &BTreeSet::new(), &BTreeSet::new(),
            ColorScheme::Monochrome, false,
        );
        assert!(cols[0].is_empty() && cols[1].is_empty() && cols[2].is_empty());
    }

    #[test]
    fn compute_block_pixels_uniform_bytes() {
        let bytes = [0xFFu8; 200];
        let cols = compute_block_pixels(
            &bytes, 200, 10,
            &BTreeMap::new(), &BTreeSet::new(),
            &BTreeSet::new(), &BTreeSet::new(),
            ColorScheme::Monochrome, false,
        );
        assert_eq!(cols[0].len(), 10);
        assert_eq!(cols[1].len(), 10);
        assert_eq!(cols[2].len(), 10);
        let brightness = 0.30_f32;
        let expected = Color::from_rgb(
            0xD4 as f32 / 255.0 * 0.55 * brightness,
            0xCA as f32 / 255.0 * 0.55 * brightness,
            0xBD as f32 / 255.0 * 0.55 * brightness,
        );
        for col in 0..3 {
            for (i, &p) in cols[col].iter().enumerate() {
                assert!((p.r - expected.r).abs() < 0.0001, "col {col} pixel {i}: r");
                assert!((p.g - expected.g).abs() < 0.0001, "col {col} pixel {i}: g");
                assert!((p.b - expected.b).abs() < 0.0001, "col {col} pixel {i}: b");
            }
        }
    }

    #[test]
    fn compute_block_pixels_pattern_dominates() {
        // Pattern at addr 5.  With 100 bytes, 10 rows → 30 blocks,
        // stride = 4.  Block 1 covers [4,8) which includes addr 5.
        // Block 1 → col 1, row 0.
        let bytes = [0x00u8; 100];
        let mut patterns = BTreeMap::new();
        patterns.insert(5, (0usize, 3u8));
        let cols = compute_block_pixels(
            &bytes, 100, 10,
            &patterns, &BTreeSet::new(),
            &BTreeSet::new(), &BTreeSet::new(),
            ColorScheme::Monochrome, false,
        );
        assert_eq!(cols[1].len(), 10);
        let expected = super::pattern_bg(3);
        assert!((cols[1][0].r - expected.r).abs() < 0.0001, "r");
        assert!((cols[1][0].g - expected.g).abs() < 0.0001, "g");
        assert!((cols[1][0].b - expected.b).abs() < 0.0001, "b");
    }

    #[test]
    fn compute_block_pixels_pattern_not_at_sample_point() {
        // Pattern at addr 7, 100 bytes, 1 row → 3 blocks, stride = 34.
        // Block 0 covers [0,34) → col 0.  Range query catches addr 7.
        let bytes = [0x00u8; 100];
        let mut patterns = BTreeMap::new();
        patterns.insert(7, (0usize, 4u8));
        let cols = compute_block_pixels(
            &bytes, 100, 1,
            &patterns, &BTreeSet::new(),
            &BTreeSet::new(), &BTreeSet::new(),
            ColorScheme::Monochrome, false,
        );
        assert_eq!(cols[0].len(), 1);
        let expected = super::pattern_bg(4);
        assert!((cols[0][0].r - expected.r).abs() < 0.0001, "r");
        assert!((cols[0][0].g - expected.g).abs() < 0.0001, "g");
        assert!((cols[0][0].b - expected.b).abs() < 0.0001, "b");
    }

    #[test]
    fn compute_block_pixels_zebra_alternate_darkens() {
        let bytes = [0x00u8; 100];
        let mut patterns = BTreeMap::new();
        patterns.insert(5, (17usize, 2u8));
        let mut alternate = BTreeSet::new();
        alternate.insert(17);
        let cols = compute_block_pixels(
            &bytes, 100, 10,
            &patterns, &alternate,
            &BTreeSet::new(), &BTreeSet::new(),
            ColorScheme::Monochrome, false,
        );
        assert_eq!(cols[1].len(), 10);
        let base = super::pattern_bg(2);
        let expected = Color::from_rgb(base.r * 0.5, base.g * 0.5, base.b * 0.5);
        assert!((cols[1][0].r - expected.r).abs() < 0.0001, "r");
        assert!((cols[1][0].g - expected.g).abs() < 0.0001, "g");
        assert!((cols[1][0].b - expected.b).abs() < 0.0001, "b");
    }

    #[test]
    fn compute_block_pixels_dirty_over_diff() {
        let bytes = [0x00u8; 100];
        let mut dirty = BTreeSet::new();
        dirty.insert(5);
        let mut diff = BTreeSet::new();
        diff.insert(5);
        let cols = compute_block_pixels(
            &bytes, 100, 10,
            &BTreeMap::new(), &BTreeSet::new(),
            &dirty, &diff,
            ColorScheme::Monochrome, false,
        );
        assert_eq!(cols[1].len(), 10);
        let dirty_c = Color::from_rgb(0x4A as f32 / 255.0, 0x1F as f32 / 255.0, 0x1A as f32 / 255.0);
        assert!((cols[1][0].r - dirty_c.r).abs() < 0.0001, "r");
        assert!((cols[1][0].g - dirty_c.g).abs() < 0.0001, "g");
        assert!((cols[1][0].b - dirty_c.b).abs() < 0.0001, "b");
    }

    #[test]
    fn compute_block_pixels_diff_when_no_pattern_or_dirty() {
        let bytes = [0x00u8; 100];
        let mut diff = BTreeSet::new();
        diff.insert(5);
        let cols = compute_block_pixels(
            &bytes, 100, 10,
            &BTreeMap::new(), &BTreeSet::new(),
            &BTreeSet::new(), &diff,
            ColorScheme::Monochrome, false,
        );
        assert_eq!(cols[1].len(), 10);
        let diff_c = Color::from_rgb(0x23 as f32 / 255.0, 0x2F as f32 / 255.0, 0x1F as f32 / 255.0);
        assert!((cols[1][0].r - diff_c.r).abs() < 0.0001, "r");
        assert!((cols[1][0].g - diff_c.g).abs() < 0.0001, "g");
        assert!((cols[1][0].b - diff_c.b).abs() < 0.0001, "b");
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
        let cols = compute_block_pixels(
            &bytes, 100, 10,
            &patterns, &BTreeSet::new(),
            &dirty, &diff,
            ColorScheme::Monochrome, false,
        );
        assert_eq!(cols[1].len(), 10);
        let expected = super::pattern_bg(1);
        assert!((cols[1][0].r - expected.r).abs() < 0.0001, "r");
        assert!((cols[1][0].g - expected.g).abs() < 0.0001, "g");
        assert!((cols[1][0].b - expected.b).abs() < 0.0001, "b");
    }

    #[test]
    fn compute_block_pixels_stride_ceiling_covers_last_block() {
        let bytes = vec![0x42u8; 101];
        let cols = compute_block_pixels(
            &bytes, 101, 10,
            &BTreeMap::new(), &BTreeSet::new(),
            &BTreeSet::new(), &BTreeSet::new(),
            ColorScheme::Monochrome, false,
        );
        assert_eq!(cols[0].len(), 10);
        assert_eq!(cols[2].len(), 10);
        let _ = cols[0][9]; // last pixel in col 0 should be valid
        let _ = cols[2][9]; // last pixel in col 2 should be valid
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
            columns: [vec![Color::WHITE], vec![], vec![]],
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
            columns: [vec![Color::WHITE], vec![], vec![]],
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
            columns: [vec![Color::WHITE], vec![], vec![]],
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
            columns: [vec![Color::WHITE], vec![], vec![]],
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
        // 9 bytes, 3 rows → 9 blocks, stride = 1.  Each block is 1 byte.
        // Col 0: blocks 0, 3, 6  → bytes 0, 3, 6 = 0x00, 0xFF, 0x00
        // Col 1: blocks 1, 4, 7  → bytes 1, 4, 7 = 0x00, 0xFF, 0x00
        // Col 2: blocks 2, 5, 8  → bytes 2, 5, 8 = 0x00, 0xFF, 0x00
        // All three cols are uniform per-block (single byte → variance=0,
        // brightness=0.30) but alternate between dim 0x00 and brighter 0xFF.
        let bytes = [0x00u8, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00];
        let cols = compute_block_pixels(
            &bytes, 9, 3,
            &BTreeMap::new(), &BTreeSet::new(),
            &BTreeSet::new(), &BTreeSet::new(),
            ColorScheme::Monochrome, false,
        );
        assert_eq!(cols[0].len(), 3);
        assert_eq!(cols[1].len(), 3);
        assert_eq!(cols[2].len(), 3);

        let brightness = 0.30_f32;
        let (fg0, _) = default_byte_colors(ColorScheme::Monochrome, 0x00, false);
        let c0 = fg0.unwrap();
        let dim = Color::from_rgb(c0.r * 0.55 * brightness, c0.g * 0.55 * brightness, c0.b * 0.55 * brightness);

        let (fg1, _) = default_byte_colors(ColorScheme::Monochrome, 0xFF, false);
        let c1 = fg1.unwrap();
        let bright = Color::from_rgb(c1.r * 0.55 * brightness, c1.g * 0.55 * brightness, c1.b * 0.55 * brightness);

        // All 3 cols have the same pattern: dim, bright, dim
        let d0 = (cols[0][0].r - dim.r).abs() + (cols[0][0].g - dim.g).abs() + (cols[0][0].b - dim.b).abs();
        assert!(d0 < 0.001, "col 0 row 0 = {}, expected dim", d0);
        let b0 = (cols[0][1].r - bright.r).abs() + (cols[0][1].g - bright.g).abs() + (cols[0][1].b - bright.b).abs();
        assert!(b0 < 0.001, "col 0 row 1 = {}, expected bright", b0);
    }

    #[test]
    fn compute_block_pixels_variance_one_col_mixed_brighter_than_uniform() {
        // With stride=1 (single byte blocks), variance is always 0.
        // Use stride=4 to get multi-byte blocks with variance.
        // 24 bytes, 2 rows → 6 blocks, stride = 4.
        // Only col 0 has multiple blocks: blocks 0 and 3.
        // Block 0: [0,4) = 0x00, 0x00, 0x00, 0xFF → mixed
        // Block 3: [12,16) = 0x00, 0x00, 0x00, 0xFF → mixed (same data)
        let mut bytes = [0x00u8; 24];
        bytes[3] = 0xFF;
        bytes[15] = 0xFF;

        // Uniform blocks: all same → zero variance
        let uniform_bytes = [0x80u8; 24];

        let p_mixed = compute_block_pixels(
            &bytes, 24, 2,
            &BTreeMap::new(), &BTreeSet::new(),
            &BTreeSet::new(), &BTreeSet::new(),
            ColorScheme::Monochrome, false,
        );
        let p_uniform = compute_block_pixels(
            &uniform_bytes, 24, 2,
            &BTreeMap::new(), &BTreeSet::new(),
            &BTreeSet::new(), &BTreeSet::new(),
            ColorScheme::Monochrome, false,
        );
        // Mixed col should have at least one pixel brighter than uniform.
        let lum_mixed: f32 = p_mixed[0].iter().map(|c| c.r + c.g + c.b).sum();
        let lum_uniform: f32 = p_uniform[0].iter().map(|c| c.r + c.g + c.b).sum();
        assert!(
            lum_mixed > lum_uniform + 0.5,
            "mixed lum {lum_mixed} should be > uniform lum {lum_uniform}"
        );
    }

}
