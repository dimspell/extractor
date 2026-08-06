//! Pure geometry helpers for the table widget.
//!
//! Every function here is a free function that depends only on the data it
//! receives — no `&self` access.  This makes the individual operations easy
//! to unit test and reason about independently of the widget's state machine.

use iced::Rectangle;

use super::types::TableColumn;

/// Width of a column (id column at index 0, data columns at 1..N).
pub fn col_width(id_col_width: f32, columns: &[TableColumn], col_idx: usize) -> f32 {
    if col_idx == 0 {
        id_col_width
    } else {
        columns[col_idx - 1].width_px
    }
}

/// Cumulative x-offsets of every column, plus one extra slot at the end
/// (so the slice has `n_cols + 1` entries, matching the original layout).
/// `positions[col]` = the left edge of column `col`; `positions[n_cols]` =
/// the right edge of the last column.
pub fn col_positions(id_col_width: f32, columns: &[TableColumn]) -> Vec<f32> {
    let n = n_cols(columns);
    let mut positions = Vec::with_capacity(n + 1);
    let mut acc = 0.0;
    positions.push(0.0);
    for c in 0..n {
        acc += col_width(id_col_width, columns, c);
        positions.push(acc);
    }
    positions
}

/// Total number of columns (id column + data columns).
pub fn n_cols(columns: &[TableColumn]) -> usize {
    columns.len() + 1
}

/// Total content width (id column + all data columns).
pub fn total_width(id_col_width: f32, columns: &[TableColumn]) -> f32 {
    id_col_width + columns.iter().map(|c| c.width_px).sum::<f32>()
}

/// Total content height (all visible rows).
pub fn total_height(n_rows: usize, row_height: f32) -> f32 {
    n_rows as f32 * row_height
}

/// Height of the frozen column-header strip (identical to a single row).
pub fn header_height(row_height: f32) -> f32 {
    row_height
}

/// Bounds of the column-header strip at the top of `bounds`.
pub fn header_bounds(bounds: Rectangle, row_height: f32) -> Rectangle {
    let h = header_height(row_height).min(bounds.height);
    Rectangle {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width,
        height: h,
    }
}

/// Bounds of the scrollable data area below the header, after reserving
/// space for visible scrollbar strips on the right and bottom edges.
pub fn body_bounds(bounds: Rectangle, total_w: f32, total_h: f32, row_height: f32) -> Rectangle {
    let header_h = header_height(row_height).min(bounds.height);
    let avail_h = (bounds.height - header_h).max(0.0);
    let needs_v = total_h > avail_h;
    let needs_h = total_w > bounds.width;
    let v_strip = if needs_v {
        super::SCROLLBAR_THICKNESS
    } else {
        0.0
    };
    let h_strip = if needs_h {
        super::SCROLLBAR_THICKNESS
    } else {
        0.0
    };
    Rectangle {
        x: bounds.x,
        y: bounds.y + header_h,
        width: (bounds.width - v_strip).max(0.0),
        height: (avail_h - h_strip).max(0.0),
    }
}

/// Inset rectangle representing the data area *after* the frozen id column.
/// Cells beyond this inset are clipped to it so the id column stays
/// permanently visible.
pub fn data_area(bounds: Rectangle, id_col_width: f32) -> Rectangle {
    let inset = id_col_width.min(bounds.width);
    Rectangle {
        x: bounds.x + inset,
        y: bounds.y,
        width: bounds.width - inset,
        height: bounds.height,
    }
}
