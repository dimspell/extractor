//! Per-instance widget state for the diff view.
//!
//! One shared scroll position, one shared cursor address (both sides
//! operate on the same address coordinate). Drag state for scrollbars,
//! minimap interaction, and hover tracking live here.

use std::cell::{Cell, Ref, RefCell};
use std::collections::BTreeSet;

use crate::ui::view::minimap::MinimapCache;

use super::{DiffView, DisplayRows};

/// Identity of the data used to build compact review rows. The comparison diff
/// is immutable while displayed, so this lets us avoid rebuilding a large
/// projection on every redraw or pointer event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DisplayRowsKey {
    diff_ptr: usize,
    diff_len: usize,
    first_diff: Option<u64>,
    last_diff: Option<u64>,
    total_rows: u64,
    bytes_per_row: u8,
    review_mode: bool,
}

struct DisplayRowsCache {
    key: DisplayRowsKey,
    rows: DisplayRows,
}

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
    /// Cached source-row → display-row projection for diff review mode.
    review_rows_cache: RefCell<Option<DisplayRowsCache>>,
}

impl State {
    /// Get the display projection for the current widget without scanning all
    /// differing bytes again. This is especially important for a large added
    /// block, where every added byte has an entry in the diff set.
    pub(super) fn display_rows<'a, Message>(
        &'a self,
        widget: &DiffView<'_, Message>,
    ) -> Ref<'a, DisplayRows> {
        let diff: &BTreeSet<u64> = widget.diff;
        let key = DisplayRowsKey {
            diff_ptr: diff as *const BTreeSet<u64> as usize,
            diff_len: diff.len(),
            first_diff: diff.first().copied(),
            last_diff: diff.last().copied(),
            total_rows: widget.total_rows(),
            bytes_per_row: widget.bytes_per_row,
            review_mode: widget.diff_review,
        };
        let is_current = self
            .review_rows_cache
            .borrow()
            .as_ref()
            .is_some_and(|cache| cache.key == key);
        if !is_current {
            *self.review_rows_cache.borrow_mut() = Some(DisplayRowsCache {
                key,
                rows: widget.build_display_rows(),
            });
        }
        Ref::map(self.review_rows_cache.borrow(), |cache| {
            &cache.as_ref().expect("display rows cache initialized").rows
        })
    }
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
            review_rows_cache: RefCell::new(None),
        }
    }
}
