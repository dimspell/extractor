//! Per-instance widget state for the diff view.
//!
//! One shared scroll position, one shared cursor address (both sides
//! operate on the same address coordinate). Drag state for scrollbars,
//! minimap interaction, and hover tracking live here.

use std::cell::{Cell, RefCell};

use crate::ui::view::minimap::MinimapCache;

/// Per-instance widget state for [`super::DiffView`].
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
    /// Side of the last mouse click/drag (`true` = baseline/left).
    /// Keyboard navigation has no side, so it reuses this to keep the
    /// inspector on the file the user was last inspecting.
    pub last_clicked_baseline: Cell<bool>,

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

impl Default for State {
    fn default() -> Self {
        Self {
            scroll_offset: Cell::new(0.0),
            scroll_x: Cell::new(0.0),
            dragging_scrollbar: false,
            dragging_scrollbar_x: false,
            hovering_scrollbar: Cell::new(false),
            dragging_cursor: false,
            // No mouse interaction yet → assume the baseline side so the
            // inspector stays on the main file during keyboard navigation.
            last_clicked_baseline: Cell::new(true),
            dragging_minimap: false,
            drag_start_minimap_y: 0.0,
            drag_start_minimap_scroll: 0.0,
            hovering_minimap: Cell::new(false),
            minimap_cache: RefCell::new(None),
        }
    }
}
