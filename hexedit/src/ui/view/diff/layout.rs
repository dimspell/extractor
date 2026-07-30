//! Layout constants and helpers for the dual-buffer diff view.
//!
//! Layout (left → right):
//!
//! ```text
//! [addr_col] [hex_A] [ascii_A] [mid_gap] [hex_B] [ascii_B] [ann_col]
//! ```
//!
//! The address column is shared. Both hex+ASCII blocks use the same
//! per-cell widths as the regular matrix. A configurable `mid_gap`
//! separates the two sides visually.

/// Height of the column header row (same as row height).
pub const HEADER_HEIGHT: f32 = 16.0;

/// Single row height for data rows.
pub const ROW_HEIGHT: f32 = 16.0;

/// Width of one hex nibble cell in the hex column.
pub const HEX_CELL_WIDTH: f32 = 18.0;

/// Width of one ASCII character cell.
pub const ASCII_CELL_WIDTH: f32 = 9.0;

/// Width of the address gutter (shared between both sides).
pub const ADDR_COL_WIDTH: f32 = 88.0;

/// Extra gap between the 8th and 9th byte groups within each hex block.
pub const GROUP_GAP: f32 = 8.0;

/// Gap between the hex block and the ASCII column within one side.
pub const COLUMN_GAP: f32 = 6.0;

/// Gap between the two sides (baseline ASCII → comparison hex).
pub const MID_GAP: f32 = 18.0;

/// Gap between the comparison ASCII column and the annotation column.
pub const ANN_COL_GAP: f32 = 8.0;

/// Minimum annotation column width.
#[allow(dead_code)]
pub const MIN_ANN_COL_WIDTH: f32 = 40.0;

/// Maximum annotation column width.
pub const MAX_ANN_COL_WIDTH: f32 = 200.0;

/// Thickness of scrollbars.
pub const SCROLLBAR_THICKNESS: f32 = 10.0;

/// Number of extra rows rendered off-screen for smooth scrolling.
pub const OVERSCAN: u32 = 2;

/// Font size for all text in the diff view.
pub const TEXT_SIZE: f32 = 11.0;

/// Return the number of group gaps for a given `bytes_per_row`.
/// e.g. 16 → 1 gap (between bytes 7 and 8), 32 → 3 gaps.
pub fn group_count(bpr: usize) -> usize {
    bpr.div_ceil(8).saturating_sub(1)
}

/// Compute the total content width for the full diff layout.
pub fn total_content_width(bpr: usize, has_annotations: bool) -> f32 {
    let hex_block = bpr as f32 * HEX_CELL_WIDTH + group_count(bpr) as f32 * GROUP_GAP;
    let ascii_block = bpr as f32 * ASCII_CELL_WIDTH;
    let side = hex_block + COLUMN_GAP + ascii_block;
    let mut w = ADDR_COL_WIDTH + side + MID_GAP + side;
    if has_annotations {
        w += ANN_COL_GAP + MAX_ANN_COL_WIDTH;
    }
    w
}

/// The x-coordinate of the baseline hex block start.
pub fn baseline_hex_start(addr_col_w: f32) -> f32 {
    addr_col_w
}

/// The x-coordinate of the baseline ASCII column start.
pub fn baseline_ascii_start(addr_col_w: f32, bpr: usize) -> f32 {
    addr_col_w + bpr as f32 * HEX_CELL_WIDTH + group_count(bpr) as f32 * GROUP_GAP + COLUMN_GAP
}

/// The x-coordinate of the comparison hex block start.
pub fn comparison_hex_start(addr_col_w: f32, bpr: usize) -> f32 {
    baseline_ascii_start(addr_col_w, bpr) + bpr as f32 * ASCII_CELL_WIDTH + MID_GAP
}

/// The x-coordinate of the comparison ASCII column start.
pub fn comparison_ascii_start(addr_col_w: f32, bpr: usize) -> f32 {
    comparison_hex_start(addr_col_w, bpr) + bpr as f32 * HEX_CELL_WIDTH
        + group_count(bpr) as f32 * GROUP_GAP
        + COLUMN_GAP
}

/// Number of visible rows based on viewport height.
pub fn page_rows(viewport_h: f32) -> u64 {
    ((viewport_h / ROW_HEIGHT) as u64).max(1)
}

/// Compute the visible row range given scroll offset and viewport dimensions.
pub fn visible_row_range(
    scroll: f32,
    viewport_h: f32,
    row_h: f32,
    total_rows: u64,
    overscan: u32,
) -> std::ops::Range<usize> {
    if viewport_h <= 0.0 || total_rows == 0 {
        return 0..0;
    }
    let first = (scroll / row_h).floor() as i64 - overscan as i64;
    let last = ((scroll + viewport_h) / row_h).ceil() as i64 + overscan as i64;
    let first = first.max(0) as usize;
    let last = (last as usize).max(first).min(total_rows as usize);
    first..last
}

/// Clamp scroll offset so content fills the viewport (no empty space at bottom).
pub fn clamp_scroll(scroll: f32, total_h: f32, viewport_h: f32) -> f32 {
    if total_h <= viewport_h {
        return 0.0;
    }
    let max_offset = total_h - viewport_h;
    scroll.clamp(0.0, max_offset)
}

/// Compute the scroll offset that centres `addr` in the viewport.
#[allow(dead_code)]
pub fn center_scroll_on(
    current_scroll: f32,
    addr: u64,
    bpr: u64,
    viewport_h: f32,
    total_h: f32,
) -> f32 {
    let row = addr / bpr;
    let row_top = row as f32 * ROW_HEIGHT;
    let row_bot = row_top + ROW_HEIGHT;
    if row_top >= current_scroll && row_bot <= current_scroll + viewport_h {
        return current_scroll; // already visible
    }
    let center = row_top - (viewport_h / 2.0) + (ROW_HEIGHT / 2.0);
    clamp_scroll(center, total_h, viewport_h)
}

/// Scroll just enough to make `addr` visible.
pub fn scroll_to_make_visible(
    current_scroll: f32,
    addr: u64,
    bpr: u64,
    viewport_h: f32,
    total_h: f32,
) -> f32 {
    let row = addr / bpr;
    let row_top = row as f32 * ROW_HEIGHT;
    let row_bot = row_top + ROW_HEIGHT;
    if row_top >= current_scroll && row_bot <= current_scroll + viewport_h {
        return current_scroll; // already visible
    }
    if row_top < current_scroll {
        clamp_scroll(row_top, total_h, viewport_h)
    } else {
        clamp_scroll(row_bot - viewport_h, total_h, viewport_h)
    }
}
