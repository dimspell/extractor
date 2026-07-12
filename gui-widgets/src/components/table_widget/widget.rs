use super::types::{RowFlags, ScrollbarDrag, State, TableColumn};
use super::{SCROLLBAR_THICKNESS};
use crate::components::paragraph_cache::ParagraphCache;
use crate::components::table_widget::state::TableState;
use iced::advanced::Shell;
use iced::{Font, Length, Point, Rectangle, Vector};
use std::borrow::Cow;
use std::cell::RefCell;

#[cfg(feature = "accessibility")]
use std::collections::HashMap;

type ScrollCallback<'a, Message> = Box<dyn Fn(f32, f32, f32) -> Message + 'a>;

pub struct TableWidget<'a, Message> {
    /// Full display cache — `display_cache[orig_idx][col_idx]`.
    pub(crate) display_cache: &'a [Vec<String>],
    /// Visible rows in display order: `filtered_indices[visible_idx] = orig_idx`.
    pub(crate) filtered_indices: &'a [usize],
    /// Owned column widths. Rebuilt per view; cheap because N ≈ 20.
    pub(crate) columns: Vec<TableColumn>,
    /// Width of the leading id column (rendered as `format!("{}", orig_idx + 1)`).
    pub(crate) id_col_width: f32,
    /// Owned closure producing flags for a given visible-row index.
    pub(crate) row_flags: Box<dyn Fn(usize) -> RowFlags + 'a>,
    pub(crate) row_height: f32,
    pub(crate) cache: ParagraphCache,
    pub(crate) text_size: f32,
    pub(crate) font: Font,
    pub(crate) cell_padding_x: f32,
    pub(crate) width: Length,
    pub(crate) height: Length,
    /// Shared, app-owned scroll state (optional — defaults to zero offset
    /// when not provided). The widget reads this every frame for rendering
    /// and publishes changes back through `on_scroll`.
    pub(crate) table_state: Option<&'a TableState>,

    // ── Callbacks ─────────────────────────────────────────────────────
    pub(crate) on_select: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    pub(crate) on_scroll: Option<ScrollCallback<'a, Message>>,
    pub(crate) on_sort: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    pub(crate) on_open_filter: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    pub(crate) on_clear_filter: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    pub(crate) on_start_resize: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    pub(crate) on_reset_column_width: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    pub(crate) on_next_highlight: Option<Box<dyn Fn() -> Message + 'a>>,
    pub(crate) on_prev_highlight: Option<Box<dyn Fn() -> Message + 'a>>,
    pub(crate) on_escape: Option<Box<dyn Fn() -> Message + 'a>>,
    pub(crate) on_quick_filter: Option<Box<dyn Fn(usize, String) -> Message + 'a>>,

    /// Accessible label for the table grid (used by screen readers).
    #[cfg(feature = "accessibility")]
    pub(crate) accessible_label: Option<String>,
    /// NodeId → (visible_row_idx, col) mapping built during `accessibility()`
    /// and consumed by `accessibility_action()`. Stored in a `RefCell` because
    /// the trait gives `&self` for building but `&mut self` for handling actions.
    #[cfg(feature = "accessibility")]
    pub(crate) cell_node_map: RefCell<HashMap<u64, (usize, usize)>>,
}

impl<'a, Message> TableWidget<'a, Message> {
    pub fn new(
        display_cache: &'a [Vec<String>],
        filtered_indices: &'a [usize],
        columns: Vec<TableColumn>,
        id_col_width: f32,
        row_flags: impl Fn(usize) -> RowFlags + 'a,
        row_height: f32,
        cache: ParagraphCache,
    ) -> Self {
        Self {
            display_cache,
            filtered_indices,
            columns,
            id_col_width,
            row_flags: Box::new(row_flags),
            row_height,
            cache,
            text_size: 10.0,
            font: Font::MONOSPACE,
            cell_padding_x: 8.0,
            width: Length::Fill,
            height: Length::Fill,
            table_state: None,
            on_select: None,
            on_scroll: None,
            on_sort: None,
            on_open_filter: None,
            on_clear_filter: None,
            on_start_resize: None,
            on_reset_column_width: None,
            on_next_highlight: None,
            on_prev_highlight: None,
            on_escape: None,
            on_quick_filter: None,
            #[cfg(feature = "accessibility")]
            accessible_label: None,
            #[cfg(feature = "accessibility")]
            cell_node_map: RefCell::new(HashMap::new()),
        }
    }

    // ── Builder methods ───────────────────────────────────────────────

    pub fn on_select(mut self, f: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    pub fn on_scroll(mut self, f: impl Fn(f32, f32, f32) -> Message + 'a) -> Self {
        self.on_scroll = Some(Box::new(f));
        self
    }

    pub fn on_sort(mut self, f: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_sort = Some(Box::new(f));
        self
    }

    pub fn on_open_filter(mut self, f: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_open_filter = Some(Box::new(f));
        self
    }

    pub fn on_clear_filter(mut self, f: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_clear_filter = Some(Box::new(f));
        self
    }

    pub fn on_start_resize(mut self, f: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_start_resize = Some(Box::new(f));
        self
    }

    pub fn on_reset_column_width(mut self, f: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_reset_column_width = Some(Box::new(f));
        self
    }

    pub fn on_next_highlight(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_next_highlight = Some(Box::new(f));
        self
    }

    pub fn on_prev_highlight(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_prev_highlight = Some(Box::new(f));
        self
    }

    pub fn on_escape(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_escape = Some(Box::new(f));
        self
    }

    pub fn on_quick_filter(mut self, f: impl Fn(usize, String) -> Message + 'a) -> Self {
        self.on_quick_filter = Some(Box::new(f));
        self
    }

    pub fn table_state(mut self, state: &'a TableState) -> Self {
        self.table_state = Some(state);
        self
    }

    pub fn text_size(mut self, size: f32) -> Self {
        self.text_size = size;
        self
    }

    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    pub fn cell_padding_x(mut self, px: f32) -> Self {
        self.cell_padding_x = px;
        self
    }

    #[cfg(feature = "accessibility")]
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    // ── Data access ───────────────────────────────────────────────────

    pub(crate) fn n_rows(&self) -> usize {
        self.filtered_indices.len()
    }

    pub(crate) fn cell_value(
        &self,
        visible_idx: usize,
        col_idx: usize,
    ) -> Option<Cow<'_, str>> {
        let orig_idx = *self.filtered_indices.get(visible_idx)?;
        if col_idx == 0 {
            Some(Cow::Owned(format!("{}", orig_idx + 1)))
        } else {
            self.display_cache
                .get(orig_idx)
                .and_then(|row| row.get(col_idx - 1))
                .map(|s| Cow::Borrowed(s.as_str()))
        }
    }

    /// Total content width (id column + all data columns).
    pub(crate) fn total_width(&self) -> f32 {
        self.id_col_width + self.columns.iter().map(|c| c.width_px).sum::<f32>()
    }

    /// Total content height (all visible rows).
    pub(crate) fn total_height(&self) -> f32 {
        self.n_rows() as f32 * self.row_height
    }

    /// Bounds of the scrollable body region (below the header, reserving
    /// space for visible scrollbar strips).
    pub(crate) fn body_bounds(&self, bounds: Rectangle) -> Rectangle {
        use super::geometry;
        let total_w = self.total_width();
        let total_h = self.total_height();
        let header_h = geometry::header_height(self.row_height).min(bounds.height);
        let avail_h = (bounds.height - header_h).max(0.0);
        let needs_v = total_h > avail_h;
        let needs_h = total_w > bounds.width;
        let v_strip = if needs_v { SCROLLBAR_THICKNESS } else { 0.0 };
        let h_strip = if needs_h { SCROLLBAR_THICKNESS } else { 0.0 };
        Rectangle {
            x: bounds.x,
            y: bounds.y + header_h,
            width: (bounds.width - v_strip).max(0.0),
            height: (avail_h - h_strip).max(0.0),
        }
    }

    /// Convenience: current scroll offset from `table_state` (zero when
    /// no state has been provided).
    pub(crate) fn scroll_offset(&self) -> Vector {
        self.table_state
            .map_or(Vector::new(0.0, 0.0), |ts| ts.scroll_offset)
    }

    // ── Scroll logic ──────────────────────────────────────────────────

    /// Apply a clamped scroll-offset change and publish the result through
    /// `on_scroll`.  Returns `true` if either axis actually moved.
    pub(crate) fn apply_scroll(
        &self,
        state: &mut State,
        bounds: Rectangle,
        new_x: f32,
        new_y: f32,
        shell: &mut Shell<'_, Message>,
    ) -> bool {
        let body = self.body_bounds(bounds);
        let total_w = self.total_width();
        let total_h = self.total_height();
        let clamped_x = new_x.clamp(0.0, (total_w - body.width).max(0.0));
        let clamped_y = new_y.clamp(0.0, (total_h - body.height).max(0.0));
        let cur = self.scroll_offset();
        let moved = (clamped_x - cur.x).abs() > f32::EPSILON
            || (clamped_y - cur.y).abs() > f32::EPSILON;
        if moved {
            shell.request_redraw();
            if let Some(cb) = &self.on_scroll {
                shell.publish(cb(clamped_x, clamped_y, body.height));
            }
            state.last_body_height = Some(body.height);
        }
        moved
    }

    /// Continue an active scrollbar drag, computing the new offset from
    /// cursor movement since `drag.start_cursor`.
    pub(crate) fn continue_drag(
        &self,
        state: &mut State,
        bounds: Rectangle,
        drag: ScrollbarDrag,
        cursor: Point,
        shell: &mut Shell<'_, Message>,
    ) {
        let body = self.body_bounds(bounds);
        match drag.axis {
            super::types::Axis::Vertical => {
                let total_h = self.total_height();
                if total_h <= body.height {
                    return;
                }
                let thumb_h = (body.height / total_h * body.height).max(20.0);
                let travel_px = (body.height - thumb_h).max(1.0);
                let max_off = (total_h - body.height).max(1.0);
                let scale = max_off / travel_px;
                let dy = cursor.y - drag.start_cursor.y;
                self.apply_scroll(
                    state,
                    bounds,
                    self.scroll_offset().x,
                    drag.start_offset.y + dy * scale,
                    shell,
                );
            }
            super::types::Axis::Horizontal => {
                let total_w = self.total_width();
                if total_w <= body.width {
                    return;
                }
                let thumb_w = (body.width / total_w * body.width).max(20.0);
                let travel_px = (body.width - thumb_w).max(1.0);
                let max_off = (total_w - body.width).max(1.0);
                let scale = max_off / travel_px;
                let dx = cursor.x - drag.start_cursor.x;
                self.apply_scroll(
                    state,
                    bounds,
                    drag.start_offset.x + dx * scale,
                    self.scroll_offset().y,
                    shell,
                );
            }
        }
    }
}
