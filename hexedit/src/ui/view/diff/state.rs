//! Per-instance widget state for the diff view.
//!
//! One shared scroll position, one shared cursor address (both sides
//! operate on the same address coordinate). Drag state for scrollbars,
//! minimap interaction, and hover tracking live here.

use std::cell::{Cell, RefCell};

use crate::ui::view::minimap::MinimapCache;

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

    // ── Minimap ────────────────────────────────────────────────────────
    /// True while dragging on the minimap strip.
    pub dragging_minimap: bool,
    /// Cursor Y when the minimap drag started.
    pub drag_start_minimap_y: f32,
    /// Scroll offset when the minimap drag started.
    pub drag_start_minimap_scroll: f32,
    /// Tracks whether cursor is over the minimap strip.
    pub hovering_minimap: Cell<bool>,
    /// Cached minimap pixel colours.
    pub minimap_cache: RefCell<Option<MinimapCache>>,
}
