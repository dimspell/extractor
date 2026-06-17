//! Per-instance widget state and readonly edit view.
//!
//! [`State`] is what the renderer keeps between frames — scroll position,
//! drag state, double-click tracking, minimap cache, etc.
//! [`EditView`] is a snapshot of the active in-line edit passed into the
//! widget so the renderer can draw the draft and the input handler can react.

use std::cell::Cell;
use std::cell::RefCell;
use std::time::{Duration, Instant};

use crate::ui::view::minimap::MinimapCache;

/// Time window for treating two consecutive clicks as a double-click.
pub const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(450);

/// Per-instance widget state — what the renderer keeps between frames.
#[derive(Default)]
pub struct State {
    pub scroll_offset: Cell<f32>,
    pub scroll_x: Cell<f32>,
    pub dragging_scrollbar: bool,
    pub dragging_scrollbar_x: bool,
    pub drag_start_cursor_y: f32,
    pub drag_start_cursor_x: f32,
    pub drag_start_offset: f32,
    pub drag_start_offset_x: f32,
    /// True while the user is actively drag-selecting bytes.
    pub selecting: bool,
    /// Last single-click address + timestamp, for double-click detection.
    pub last_click_addr: Option<u64>,
    pub last_click_at: Option<Instant>,
    /// Row of the cursor that we've already scrolled to.
    pub last_cursor_row: Cell<Option<u64>>,
    /// Tracks whether cursor is over either scrollbar, to avoid unnecessary
    /// redraws during cursor movement.
    pub hovering_scrollbar: Cell<bool>,
    /// True while the user is actively drag-scrolling on the minimap.
    pub dragging_minimap: bool,
    /// Cursor Y when the minimap drag started.
    pub drag_start_minimap_y: f32,
    /// Scroll offset when the minimap drag started.
    pub drag_start_minimap_scroll: f32,
    /// Tracks whether cursor is over the minimap strip, to avoid unnecessary
    /// redraws.
    pub hovering_minimap: Cell<bool>,
    /// Cached minimap pixel colors — avoids re-scanning the file every frame.
    /// Invalidated automatically when any input to the pixel computation
    /// changes (file size, colour scheme, patterns, dirty/diff sets).
    pub minimap_cache: RefCell<Option<MinimapCache>>,
    /// Shift modifier state — held while scrolling redirects vertical wheel
    /// delta to horizontal scroll.
    pub shift_pressed: Cell<bool>,
}

/// Read-only view of the active edit, threaded into the widget so the renderer
/// can draw the draft and the input handler can react.
#[derive(Debug, Clone, Copy)]
pub struct EditView<'a> {
    pub addr: u64,
    pub draft: &'a str,
}
