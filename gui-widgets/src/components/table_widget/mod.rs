//! Custom virtualised table widget — Approach B from
//! `docs/spreadsheet_scroll_perf.md`.
//!
//! Replaces the `column(of lazy(of button(of row(of cells))))` tree that used
//! to back `build_table_content` with a single widget that owns viewport
//! state and draws cells directly. Only rows that intersect the visible
//! bounds are shaped; everything else is skipped. Reuses [`ParagraphCache`]
//! from `paragraph_cache` so the shaped paragraphs survive viewport changes.
//!
//! ## Data layout
//!
//! The widget borrows two slices: `display_cache[orig_idx][col_idx]` (full
//! row data, parallel to the catalog) and `filtered_indices[visible_idx] =
//! orig_idx` (which rows to show, in order).
//!
//! ## External vs internal scroll
//!
//! Scroll offset is owned by widget state, but each `view()` rebuild passes
//! an `external_offset` from `SpreadsheetState`. When that differs from the
//! last value the widget saw, the widget snaps to it. This is the path
//! programmatic navigation (`NavigateUp`, `NavigateNextHighlight`, …) takes
//! to move the viewport: the message handler sets `vertical_scroll_offset`
//! on the state, and the widget reads it next frame.

/// Width in pixels of the painted scrollbar thumbs along the right and
/// bottom edges of the table.
pub(crate) const SCROLLBAR_THICKNESS: f32 = 10.0;

/// Width of the right-edge resize-handle strip painted in each column header.
/// Click-drag on this strip resizes the column; double-click resets it.
pub(crate) const RESIZE_HANDLE_WIDTH: f32 = 5.0;

/// Width of the filter-open `▾` icon in each column header.
pub(crate) const FILTER_ICON_WIDTH: f32 = 14.0;

/// Width of the filter-active `◼` badge in each column header.
pub(crate) const FILTER_BADGE_WIDTH: f32 = 14.0;

/// Threshold (ms) for treating two consecutive clicks on the same resize
/// handle as a double-click — emits `on_reset_column_width` instead of
/// `on_start_resize`.
pub(crate) const DOUBLE_CLICK_MS: u128 = 400;

pub mod style;
pub mod types;
pub mod widget;
pub mod widget_trait_impl;

#[cfg(test)]
pub mod tests;

// Re-exports for public API
pub use types::RowFlags;
pub use types::TableColumn;
pub use widget::TableWidget;
