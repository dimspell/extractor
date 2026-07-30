//! Per-instance widget state for the diff view.
//!
//! One shared scroll position, one shared cursor address (both sides
//! operate on the same address coordinate). Drag state for scrollbars
//! and hover tracking live here.

use std::cell::Cell;

/// Per-instance widget state for [`super::DiffView`].
#[derive(Default)]
pub struct State {
    /// Vertical scroll offset in logical pixels.
    pub scroll_offset: Cell<f32>,
    /// Horizontal scroll offset (content may be wider than viewport).
    pub scroll_x: Cell<f32>,
    /// True while dragging the vertical scrollbar.
    pub dragging_scrollbar: bool,
    /// True while dragging the horizontal scrollbar.
    pub dragging_scrollbar_x: bool,
    /// Tracks whether cursor is over either scrollbar.
    pub hovering_scrollbar: Cell<bool>,
    /// True while dragging the mouse to extend a selection range.
    pub dragging_cursor: bool,
}
