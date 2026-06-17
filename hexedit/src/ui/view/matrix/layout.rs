//! Pure layout helpers: geometry constants, scroll clamping, row/page math,
//! hit-testing, and shared utility functions.
//!
//! None of these functions touch widget state — they are all deterministic
//! functions of their parameters.

use iced::{Point, Rectangle};
use std::ops::Range;

// ── Cell metrics ──────────────────────────────────────────────────────

/// Default cell metrics. Tuned for 11px monospace.
pub const TEXT_SIZE: f32 = 11.0;
pub const ROW_HEIGHT: f32 = 16.0;
pub const HEX_CELL_WIDTH: f32 = 20.0;
pub const ASCII_CELL_WIDTH: f32 = 9.0;
pub const GROUP_GAP: f32 = 8.0;
pub const COLUMN_GAP: f32 = 12.0;
pub const ANN_COL_GAP: f32 = 16.0;

/// Height of the fixed column header row above the hex area.
pub const HEADER_HEIGHT: f32 = 16.0;

/// Maximum width of the annotation column when computed from content.
pub const MAX_ANN_COL_WIDTH: f32 = 400.0;
/// Minimum annotation column width when no annotations exist.
pub const MIN_ANN_COL_WIDTH: f32 = 200.0;
pub const SCROLLBAR_THICKNESS: f32 = 10.0;

/// How many extra rows to render above/below the viewport so wheel scrolls
/// don't reveal blank bands during rapid scroll.
pub const OVERSCAN: u64 = 2;

// ── Geometry helpers ──────────────────────────────────────────────────

/// Number of inter-group gaps for a row of `bpr` bytes.
///
/// Returns `ceil(bpr / 8) - 1`, i.e. 0 for ≤8 bytes, 1 for 9–16, etc.
pub fn group_count(bpr: usize) -> usize {
    bpr.div_ceil(8).saturating_sub(1)
}

/// Visible row range `[first, last)` for the current scroll state, including
/// overscan. Both bounds are clamped to `[0, total_rows)`.
pub fn visible_row_range(
    scroll: f32,
    viewport_height: f32,
    row_height: f32,
    total_rows: u64,
    overscan: u64,
) -> Range<u64> {
    if total_rows == 0 || row_height <= 0.0 {
        return 0..0;
    }
    let scroll = scroll.max(0.0);
    let raw_first = ((scroll / row_height).floor() as i64 - overscan as i64).max(0) as u64;
    let first = raw_first.min(total_rows);
    let visible = (viewport_height / row_height).ceil() as u64 + overscan * 2 + 1;
    let last = first.saturating_add(visible).min(total_rows);
    first..last
}

/// Clamp vertical scroll offset so the last row doesn't scroll past the
/// viewport bottom.
pub fn clamp_scroll(scroll: f32, total_height: f32, viewport_height: f32) -> f32 {
    let max_off = (total_height - viewport_height).max(0.0);
    scroll.clamp(0.0, max_off)
}

/// Clamp horizontal scroll offset.
pub fn clamp_scroll_x(scroll: f32, content_w: f32, view_w: f32) -> f32 {
    let max_off = (content_w - view_w).max(0.0);
    scroll.clamp(0.0, max_off)
}

/// Number of complete rows that fit in `viewport_height`. Used for
/// PageUp/PageDown nav and "ensure visible" math.
pub fn page_rows(viewport_height: f32) -> u64 {
    (viewport_height / ROW_HEIGHT).floor().max(1.0) as u64
}

/// Adjust `scroll` so `addr` is centered in the viewport. Returns the new
/// scroll offset (clamped to valid range).
pub fn ensure_visible(
    scroll: f32,
    addr: u64,
    bytes_per_row: u64,
    viewport_height: f32,
    total_height: f32,
) -> f32 {
    let bpr = bytes_per_row.max(1);
    let row = addr / bpr;
    let row_top = row as f32 * ROW_HEIGHT;
    let row_bot = row_top + ROW_HEIGHT;
    if row_top >= scroll && row_bot <= scroll + viewport_height {
        return clamp_scroll(scroll, total_height, viewport_height);
    }
    let center = row_top - (viewport_height - ROW_HEIGHT) / 2.0;
    clamp_scroll(center, total_height, viewport_height)
}

/// Hit-test: convert a screen point inside `bounds` to a byte address.
/// Considers both the hex column and the ASCII column.
pub fn addr_at(
    point: Point,
    bounds: Rectangle,
    scroll: f32,
    scroll_x: f32,
    bytes_per_row: u8,
    total_len: u64,
    addr_col_width: f32,
) -> Option<u64> {
    if total_len == 0 {
        return None;
    }
    if !bounds.contains(point) {
        return None;
    }
    let bpr = bytes_per_row.max(1) as f32;
    let local_y = (point.y - bounds.y) + scroll;
    if local_y < 0.0 {
        return None;
    }
    let row = (local_y / ROW_HEIGHT) as u64;

    let hex_start = bounds.x + addr_col_width - scroll_x;
    let bpr_usize = bytes_per_row.max(1) as usize;
    let hex_end =
        hex_start + bpr * HEX_CELL_WIDTH + group_count(bpr_usize) as f32 * GROUP_GAP;
    let ascii_start = hex_end + COLUMN_GAP;
    let ascii_end = ascii_start + bpr * ASCII_CELL_WIDTH;

    let col = if point.x >= hex_start && point.x < hex_end {
        // Account for inter-group gaps when figuring out the column index.
        let mut x = point.x - hex_start;
        let mut col = 0u64;
        for c in 0..bytes_per_row.max(1) as u64 {
            let g = (c / 8) as f32;
            let cell_l = c as f32 * HEX_CELL_WIDTH + g * GROUP_GAP;
            let cell_r = cell_l + HEX_CELL_WIDTH;
            if x < cell_r {
                col = c;
                x = -1.0; // sentinel: found
                break;
            }
            col = c;
        }
        if x >= 0.0 {
            // Past the last cell — clamp.
            col = bytes_per_row.saturating_sub(1) as u64;
        }
        col
    } else if point.x >= ascii_start && point.x < ascii_end {
        ((point.x - ascii_start) / ASCII_CELL_WIDTH) as u64
    } else {
        return None;
    };

    let addr = row * bytes_per_row as u64 + col;
    if addr >= total_len {
        Some(total_len - 1)
    } else {
        Some(addr)
    }
}
