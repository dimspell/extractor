use iced::{Point, Vector};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct TableColumn {
    pub width_px: f32,
    pub label: String,
    /// `None` = unsorted, `Some(true)` = ascending, `Some(false)` = descending.
    pub sort: Option<bool>,
    pub has_filter: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RowFlags {
    pub selected: bool,
    pub highlighted: bool,
    pub current_highlight: bool,
}

#[derive(Default)]
pub(crate) struct State {
    pub(crate) scroll_offset: Vector,
    /// Most recent `external_offset` we synced to. `None` until the first
    /// frame.
    pub(crate) last_external: Option<Vector>,
    pub(crate) hovered_row: Option<usize>,
    /// Which scrollbar (if any) the cursor is currently over. Used to
    /// fatten + brighten the thumb so it reads as draggable.
    pub(crate) hovered_scrollbar: Option<Axis>,
    /// Header sub-region the cursor is over (column index + region kind).
    /// Drives hover backgrounds on the label and resize handle.
    pub(crate) hovered_header: Option<(usize, HeaderRegion)>,
    /// Active scrollbar drag, if any. Contains the cursor position recorded
    /// when the drag started and the scroll offset at that moment, so the
    /// drag math can map cursor delta → offset delta linearly.
    pub(crate) dragging: Option<ScrollbarDrag>,
    /// Last resize-handle press, used for double-click detection. The widget
    /// emits `on_start_resize` on first press; if the next press on the same
    /// handle arrives within `DOUBLE_CLICK_MS` the second one is converted
    /// into `on_reset_column_width`.
    pub(crate) last_resize_click: Option<(usize, Instant)>,
    /// Body height observed on the most recent event. Used to detect
    /// viewport-size changes and republish `on_scroll` so the parent's
    /// cached `viewport_height` stays fresh — programmatic scroll-to math
    /// (arrow nav, highlight jumps) depends on it.
    pub(crate) last_body_height: Option<f32>,
    /// Shift modifier state — held while scrolling redirects vertical wheel
    /// delta to horizontal scroll.
    pub(crate) shift_pressed: bool,
}

/// Sub-region of a header cell. Used for hit-testing clicks and for hover
/// styling. `col_idx` semantics match `TableColumn` indexing — i.e. data
/// columns are 0-based; the leading id column has no header region (it is
/// painted as a static `#`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeaderRegion {
    /// Label area on the left — left-click triggers `on_sort`.
    Label,
    /// Filter-open `▾` icon — left-click triggers `on_open_filter`.
    FilterOpen,
    /// Filter-active `◼` badge (only present when the column has an active
    /// filter) — left-click triggers `on_clear_filter`.
    FilterBadge,
    /// Right-edge resize handle — press triggers `on_start_resize`,
    /// double-click triggers `on_reset_column_width`.
    Resize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Axis {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ScrollbarDrag {
    pub(crate) axis: Axis,
    /// Cursor position (parent coords) at drag start.
    pub(crate) start_cursor: Point,
    /// Scroll offset at drag start.
    pub(crate) start_offset: Vector,
}
