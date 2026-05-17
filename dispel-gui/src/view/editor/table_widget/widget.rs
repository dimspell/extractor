use super::types::{Axis, HeaderRegion, RowFlags, ScrollbarDrag, State, TableColumn};
use super::{FILTER_BADGE_WIDTH, FILTER_ICON_WIDTH, RESIZE_HANDLE_WIDTH, SCROLLBAR_THICKNESS};
use crate::view::editor::paragraph_cache::ParagraphCache;
use iced::advanced::Shell;
use iced::{Font, Length, Point, Rectangle, Size, Vector};

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
    /// Scroll offset the parent state currently holds. The widget snaps its
    /// internal offset to this value whenever it changes (allowing
    /// programmatic scroll-to-row from the message layer).
    pub(crate) external_offset: Vector,
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
    pub(crate) shift_pressed: bool,
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
            external_offset: Vector::new(0.0, 0.0),
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
            shift_pressed: false,
        }
    }

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

    pub fn shift_pressed(mut self, pressed: bool) -> Self {
        self.shift_pressed = pressed;
        self
    }

    pub fn external_offset(mut self, x: f32, y: f32) -> Self {
        self.external_offset = Vector::new(x, y);
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

    pub(crate) fn n_rows(&self) -> usize {
        self.filtered_indices.len()
    }

    pub(crate) fn n_cols(&self) -> usize {
        // +1 for the leading id column.
        self.columns.len() + 1
    }

    pub(crate) fn col_width(&self, col_idx: usize) -> f32 {
        if col_idx == 0 {
            self.id_col_width
        } else {
            self.columns[col_idx - 1].width_px
        }
    }

    pub(crate) fn total_width(&self) -> f32 {
        self.id_col_width + self.columns.iter().map(|c| c.width_px).sum::<f32>()
    }

    pub(crate) fn total_height(&self) -> f32 {
        self.n_rows() as f32 * self.row_height
    }

    pub(crate) fn cell_value(&self, visible_idx: usize, col_idx: usize) -> Option<String> {
        let orig_idx = *self.filtered_indices.get(visible_idx)?;
        if col_idx == 0 {
            Some(format!("{}", orig_idx + 1))
        } else {
            self.display_cache
                .get(orig_idx)
                .and_then(|row| row.get(col_idx - 1))
                .cloned()
        }
    }

    /// Width of the data area (excludes the frozen id column).
    pub(crate) fn data_area(&self, bounds: Rectangle) -> Rectangle {
        let inset = self.id_col_width.min(bounds.width);
        Rectangle {
            x: bounds.x + inset,
            y: bounds.y,
            width: bounds.width - inset,
            height: bounds.height,
        }
    }

    /// Height reserved at the top of `bounds` for the frozen column-header row.
    pub(crate) fn header_height(&self) -> f32 {
        self.row_height
    }

    /// Bounds of the column-header strip.
    pub(crate) fn header_bounds(&self, bounds: Rectangle) -> Rectangle {
        let h = self.header_height().min(bounds.height);
        Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: h,
        }
    }

    /// Bounds of the scrollable body.
    pub(crate) fn body_bounds(&self, bounds: Rectangle) -> Rectangle {
        let header_h = self.header_height().min(bounds.height);
        let avail_h = (bounds.height - header_h).max(0.0);
        let needs_v = self.total_height() > avail_h;
        let needs_h = self.total_width() > bounds.width;
        let v_strip = if needs_v { SCROLLBAR_THICKNESS } else { 0.0 };
        let h_strip = if needs_h { SCROLLBAR_THICKNESS } else { 0.0 };
        Rectangle {
            x: bounds.x,
            y: bounds.y + header_h,
            width: (bounds.width - v_strip).max(0.0),
            height: (avail_h - h_strip).max(0.0),
        }
    }

    /// Hit-test a cursor position against the column-header strip.
    pub(crate) fn header_hit(
        &self,
        bounds: Rectangle,
        off_x: f32,
        p: Point,
    ) -> Option<(usize, HeaderRegion)> {
        let header = self.header_bounds(bounds);
        if !header.contains(p) {
            return None;
        }
        let id_w = self.id_col_width.min(bounds.width);
        let id_r = bounds.x + id_w;
        if p.x < id_r {
            return None;
        }
        let local_x = (p.x - id_r) + off_x;
        if local_x < 0.0 {
            return None;
        }
        let mut acc = 0.0_f32;
        for (col, c) in self.columns.iter().enumerate() {
            let col_l = acc;
            let col_r = col_l + c.width_px;
            if local_x < col_r {
                let rel = local_x - col_l;
                let resize_l = c.width_px - RESIZE_HANDLE_WIDTH;
                let filter_btn_l = resize_l - FILTER_ICON_WIDTH;
                let filter_badge_l = if c.has_filter {
                    filter_btn_l - FILTER_BADGE_WIDTH
                } else {
                    filter_btn_l
                };
                let region = if rel >= resize_l {
                    HeaderRegion::Resize
                } else if rel >= filter_btn_l {
                    HeaderRegion::FilterOpen
                } else if c.has_filter && rel >= filter_badge_l {
                    HeaderRegion::FilterBadge
                } else {
                    HeaderRegion::Label
                };
                return Some((col, region));
            }
            acc = col_r;
        }
        None
    }

    /// True when the cursor is over either scrollbar's track.
    pub(crate) fn over_scrollbar(&self, bounds: Rectangle, off: Vector, p: Point) -> bool {
        self.scrollbar_under(bounds, off, p).is_some()
    }

    /// Which scrollbar (if any) the cursor is currently over. The expanded
    /// hit-area covers the whole track so the thumb still feels grabbable
    /// when the cursor hits anywhere along the bar.
    pub(crate) fn scrollbar_under(&self, bounds: Rectangle, off: Vector, p: Point) -> Option<Axis> {
        if let Some((track, _)) = self.vertical_scrollbar(bounds, off.y) {
            if track.contains(p) {
                return Some(Axis::Vertical);
            }
        }
        if let Some((track, _)) = self.horizontal_scrollbar(bounds, off.x) {
            if track.contains(p) {
                return Some(Axis::Horizontal);
            }
        }
        None
    }

    /// Apply a clamped scroll-offset change. Returns `true` if either axis
    /// actually moved. Updates `last_external` so the next layout's
    /// external-sync diff treats the new value as "in sync".
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
        let moved = (clamped_x - state.scroll_offset.x).abs() > f32::EPSILON
            || (clamped_y - state.scroll_offset.y).abs() > f32::EPSILON;
        if moved {
            state.scroll_offset.x = clamped_x;
            state.scroll_offset.y = clamped_y;
            state.last_external = Some(state.scroll_offset);
            shell.request_redraw();
            if let Some(cb) = &self.on_scroll {
                shell.publish(cb(clamped_x, clamped_y, body.height));
            }
            state.last_body_height = Some(body.height);
        }
        moved
    }

    /// Continue an active scrollbar drag, updating the scroll offset based
    /// on cursor movement since drag start.
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
            Axis::Vertical => {
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
                    state.scroll_offset.x,
                    drag.start_offset.y + dy * scale,
                    shell,
                );
            }
            Axis::Horizontal => {
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
                    state.scroll_offset.y,
                    shell,
                );
            }
        }
    }

    /// Geometry of the vertical scrollbar (track + thumb), or `None` if the
    /// content fits vertically. The track sits in the reserved strip on the
    /// right edge of `bounds`; thumb travel uses `body.height` so it doesn't
    /// extend past the horizontal scrollbar's corner reservation.
    pub(crate) fn vertical_scrollbar(
        &self,
        bounds: Rectangle,
        off_y: f32,
    ) -> Option<(Rectangle, Rectangle)> {
        let body = self.body_bounds(bounds);
        let total_h = self.total_height();
        if total_h <= body.height {
            return None;
        }
        let track = Rectangle {
            x: bounds.x + bounds.width - SCROLLBAR_THICKNESS,
            y: body.y,
            width: SCROLLBAR_THICKNESS,
            height: body.height,
        };
        let thumb_h = (body.height / total_h * body.height).max(20.0);
        let max_off = (total_h - body.height).max(1.0);
        let thumb_y = body.y + (off_y / max_off) * (body.height - thumb_h);
        let thumb = Rectangle {
            x: track.x + 1.0,
            y: thumb_y,
            width: SCROLLBAR_THICKNESS - 2.0,
            height: thumb_h,
        };
        Some((track, thumb))
    }

    /// Geometry of the horizontal scrollbar (track + thumb), or `None` if the
    /// content fits horizontally.
    pub(crate) fn horizontal_scrollbar(
        &self,
        bounds: Rectangle,
        off_x: f32,
    ) -> Option<(Rectangle, Rectangle)> {
        let body = self.body_bounds(bounds);
        let total_w = self.total_width();
        if total_w <= body.width {
            return None;
        }
        let track = Rectangle {
            x: bounds.x,
            y: bounds.y + bounds.height - SCROLLBAR_THICKNESS,
            width: body.width,
            height: SCROLLBAR_THICKNESS,
        };
        let thumb_w = (body.width / total_w * body.width).max(20.0);
        let max_off = (total_w - body.width).max(1.0);
        let thumb_x = bounds.x + (off_x / max_off) * (body.width - thumb_w);
        let thumb = Rectangle {
            x: thumb_x,
            y: track.y + 1.0,
            width: thumb_w,
            height: SCROLLBAR_THICKNESS - 2.0,
        };
        Some((track, thumb))
    }

    /// Sync `state.scroll_offset` to `external_offset` when the parent state
    /// has moved (programmatic scroll). Idempotent — only triggers when the
    /// external value differs from what we last observed, so the user's own
    /// wheel-scroll updates aren't clobbered.
    pub(crate) fn sync_external(&self, state: &mut State, bounds: Size) {
        let need_sync = state
            .last_external
            .is_none_or(|last| last != self.external_offset);
        if !need_sync {
            return;
        }
        let body = self.body_bounds(Rectangle {
            x: 0.0,
            y: 0.0,
            width: bounds.width,
            height: bounds.height,
        });
        let total_w = self.total_width();
        let total_h = self.total_height();
        let max_x = (total_w - body.width).max(0.0);
        let max_y = (total_h - body.height).max(0.0);
        state.scroll_offset.x = self.external_offset.x.clamp(0.0, max_x);
        state.scroll_offset.y = self.external_offset.y.clamp(0.0, max_y);
        state.last_external = Some(self.external_offset);
    }
}
