//! Custom virtualised table widget — Approach B from
//! `docs/spreadsheet_scroll_perf.md`.
//!
//! Replaces the `column(of lazy(of button(of row(of cells))))` tree in
//! `build_table_content` with a single widget that owns viewport state and
//! draws cells directly. Only rows that intersect the visible bounds are
//! shaped; everything else is skipped. Reuses [`ParagraphCache`] from
//! `cached_text` so the shaped paragraphs survive viewport changes.
//!
//! Lives behind the `table_widget` cargo feature so the existing lazy/column
//! path remains the default until step 8 deletes it.
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

use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
use iced::advanced::layout::{Layout, Limits, Node};
use iced::advanced::renderer;
use iced::advanced::text::{self, Paragraph as _};
use iced::advanced::widget::{tree, Tree, Widget};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::keyboard::key;
use iced::{
    alignment, color, keyboard, mouse, Background, Border, Color, Element, Event, Font, Length,
    Pixels, Point, Rectangle, Shadow, Size, Vector,
};

use crate::view::editor::cached_text::{ParagraphCache, ParagraphKey};

type Paragraph = GraphicsParagraph;

/// Width in pixels of the painted scrollbar thumbs along the right and
/// bottom edges of the table.
const SCROLLBAR_THICKNESS: f32 = 10.0;

/// Width of the right-edge resize-handle strip painted in each column header.
/// Click-drag on this strip resizes the column; double-click resets it.
const RESIZE_HANDLE_WIDTH: f32 = 5.0;

/// Width of the filter-open `▾` icon in each column header.
const FILTER_ICON_WIDTH: f32 = 14.0;

/// Width of the filter-active `◼` badge in each column header.
const FILTER_BADGE_WIDTH: f32 = 14.0;

/// Threshold (ms) for treating two consecutive clicks on the same resize
/// handle as a double-click — emits `on_reset_column_width` instead of
/// `on_start_resize`.
const DOUBLE_CLICK_MS: u128 = 400;

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

pub struct TableWidget<'a, Message> {
    /// Full display cache — `display_cache[orig_idx][col_idx]`.
    display_cache: &'a [Vec<String>],
    /// Visible rows in display order: `filtered_indices[visible_idx] = orig_idx`.
    filtered_indices: &'a [usize],
    /// Owned column widths. Rebuilt per view; cheap because N ≈ 20.
    columns: Vec<TableColumn>,
    /// Width of the leading id column (rendered as `format!("{}", orig_idx + 1)`).
    id_col_width: f32,
    /// Owned closure producing flags for a given visible-row index.
    row_flags: Box<dyn Fn(usize) -> RowFlags + 'a>,
    row_height: f32,
    cache: ParagraphCache,
    text_size: f32,
    font: Font,
    cell_padding_x: f32,
    width: Length,
    height: Length,
    /// Scroll offset the parent state currently holds. The widget snaps its
    /// internal offset to this value whenever it changes (allowing
    /// programmatic scroll-to-row from the message layer).
    external_offset: Vector,
    on_select: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    on_scroll: Option<Box<dyn Fn(f32, f32) -> Message + 'a>>,
    on_sort: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    on_open_filter: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    on_clear_filter: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    on_start_resize: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    on_reset_column_width: Option<Box<dyn Fn(usize) -> Message + 'a>>,
}

#[derive(Default)]
struct State {
    scroll_offset: Vector,
    /// Most recent `external_offset` we synced to. `None` until the first
    /// frame.
    last_external: Option<Vector>,
    hovered_row: Option<usize>,
    /// Which scrollbar (if any) the cursor is currently over. Used to
    /// fatten + brighten the thumb so it reads as draggable.
    hovered_scrollbar: Option<Axis>,
    /// Header sub-region the cursor is over (column index + region kind).
    /// Drives hover backgrounds on the label and resize handle.
    hovered_header: Option<(usize, HeaderRegion)>,
    /// Active scrollbar drag, if any. Contains the cursor position recorded
    /// when the drag started and the scroll offset at that moment, so the
    /// drag math can map cursor delta → offset delta linearly.
    dragging: Option<ScrollbarDrag>,
    /// Last resize-handle press, used for double-click detection. The widget
    /// emits `on_start_resize` on first press; if the next press on the same
    /// handle arrives within `DOUBLE_CLICK_MS` the second one is converted
    /// into `on_reset_column_width`.
    last_resize_click: Option<(usize, std::time::Instant)>,
}

/// Sub-region of a header cell. Used for hit-testing clicks and for hover
/// styling. `col_idx` semantics match `TableColumn` indexing — i.e. data
/// columns are 0-based; the leading id column has no header region (it is
/// painted as a static `#`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HeaderRegion {
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
enum Axis {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy)]
struct ScrollbarDrag {
    axis: Axis,
    /// Cursor position (parent coords) at drag start.
    start_cursor: Point,
    /// Scroll offset at drag start.
    start_offset: Vector,
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
        }
    }

    pub fn on_select(mut self, f: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    pub fn on_scroll(mut self, f: impl Fn(f32, f32) -> Message + 'a) -> Self {
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

    fn n_rows(&self) -> usize {
        self.filtered_indices.len()
    }

    fn n_cols(&self) -> usize {
        // +1 for the leading id column.
        self.columns.len() + 1
    }

    fn col_width(&self, col_idx: usize) -> f32 {
        if col_idx == 0 {
            self.id_col_width
        } else {
            self.columns[col_idx - 1].width_px
        }
    }

    fn total_width(&self) -> f32 {
        self.id_col_width + self.columns.iter().map(|c| c.width_px).sum::<f32>()
    }

    fn total_height(&self) -> f32 {
        self.n_rows() as f32 * self.row_height
    }

    fn cell_value(&self, visible_idx: usize, col_idx: usize) -> Option<String> {
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
    fn data_area(&self, bounds: Rectangle) -> Rectangle {
        let inset = self.id_col_width.min(bounds.width);
        Rectangle {
            x: bounds.x + inset,
            y: bounds.y,
            width: bounds.width - inset,
            height: bounds.height,
        }
    }

    /// Height reserved at the top of `bounds` for the frozen column-header
    /// row. The widget paints the header itself (labels, sort indicators,
    /// filter icons, resize handles) instead of stacking a separate `column`
    /// outside.
    fn header_height(&self) -> f32 {
        self.row_height
    }

    /// Bounds of the column-header strip (always anchored to the top of
    /// `bounds`, full-width).
    fn header_bounds(&self, bounds: Rectangle) -> Rectangle {
        let h = self.header_height().min(bounds.height);
        Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: h,
        }
    }

    /// Bounds of the scrollable body — `bounds` minus the frozen header at
    /// the top and a `SCROLLBAR_THICKNESS` strip on the right (vertical
    /// scrollbar) and bottom (horizontal scrollbar) when those scrollbars
    /// are needed. Reserving the strip in idle state means cells never sit
    /// underneath the scrollbar; the thumb's hover/drag growth extends
    /// *into* the body which is fine because the user is interacting with
    /// it at that moment.
    fn body_bounds(&self, bounds: Rectangle) -> Rectangle {
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

    /// Hit-test a cursor position against the column-header strip. Returns
    /// the column index (in `self.columns`) plus the sub-region the cursor
    /// is over. Returns `None` when the cursor is above the frozen `#` cell
    /// or outside the header strip entirely — those areas have no
    /// interaction.
    fn header_hit(
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
    fn over_scrollbar(&self, bounds: Rectangle, off: Vector, p: Point) -> bool {
        self.scrollbar_under(bounds, off, p).is_some()
    }

    /// Which scrollbar (if any) the cursor is currently over. The expanded
    /// hit-area covers the whole track so the thumb still feels grabbable
    /// when the cursor hits anywhere along the bar.
    fn scrollbar_under(&self, bounds: Rectangle, off: Vector, p: Point) -> Option<Axis> {
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
    fn apply_scroll(
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
                shell.publish(cb(clamped_x, clamped_y));
            }
        }
        moved
    }

    fn continue_drag(
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
    fn vertical_scrollbar(&self, bounds: Rectangle, off_y: f32) -> Option<(Rectangle, Rectangle)> {
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
    fn horizontal_scrollbar(&self, bounds: Rectangle, off_x: f32) -> Option<(Rectangle, Rectangle)> {
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
    fn sync_external(&self, state: &mut State, bounds: Size) {
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

// ── Style palette (pulled from style::spreadsheet_row) ────────────────────
fn cell_text_color(flags: RowFlags) -> Color {
    if flags.current_highlight {
        color!(0xffffff)
    } else if flags.highlighted {
        color!(0xfff2c0)
    } else if flags.selected {
        color!(0xffd700)
    } else {
        color!(0xd4c5a9)
    }
}

fn id_text_color(flags: RowFlags) -> Color {
    if flags.selected || flags.current_highlight {
        color!(0xffd700)
    } else {
        color!(0x6a5e54)
    }
}

fn id_cell_bg(flags: RowFlags) -> Color {
    if flags.selected || flags.current_highlight {
        color!(0x2a2218)
    } else {
        color!(0x171411)
    }
}

fn row_bg(visible_idx: usize, flags: RowFlags, hovered: bool) -> Color {
    if flags.current_highlight {
        color!(0x7a6a2a)
    } else if flags.highlighted {
        color!(0x5a4e1a)
    } else if flags.selected {
        color!(0x3a2e1a)
    } else if hovered {
        color!(0x2d2820)
    } else if visible_idx.is_multiple_of(2) {
        color!(0x1e1b17)
    } else {
        color!(0x232019)
    }
}

fn row_border(flags: RowFlags) -> Option<(Color, f32)> {
    if flags.current_highlight {
        Some((color!(0xffd700, 0.85), 2.0))
    } else if flags.highlighted {
        Some((color!(0xdaa520, 0.7), 1.0))
    } else if flags.selected {
        Some((color!(0xdaa520, 0.5), 1.0))
    } else {
        None
    }
}

impl<Message, Theme> Widget<Message, Theme, iced::Renderer> for TableWidget<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&mut self, tree: &mut Tree, _renderer: &iced::Renderer, limits: &Limits) -> Node {
        let max = limits.max();
        // Sync external scroll into widget state on every layout so
        // programmatic moves take effect even if no events fire.
        let state = tree.state.downcast_mut::<State>();
        self.sync_external(state, max);
        Node::new(Size::new(max.width, max.height))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if !cursor.is_over(bounds) {
                    return;
                }
                let (dx, dy) = match delta {
                    mouse::ScrollDelta::Lines { x, y } => {
                        (-x * self.row_height, -y * self.row_height)
                    }
                    mouse::ScrollDelta::Pixels { x, y } => (-x, -y),
                };
                let new_x = state.scroll_offset.x + dx;
                let new_y = state.scroll_offset.y + dy;
                if self.apply_scroll(state, bounds, new_x, new_y, shell) {
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(drag) = state.dragging {
                    let Some(cur) = cursor.position() else { return };
                    self.continue_drag(state, bounds, drag, cur, shell);
                    shell.capture_event();
                    return;
                }
                // Track which scrollbar (if any) the cursor is over so the
                // thumb can fatten + brighten on hover.
                let new_sb_hover = cursor
                    .position_over(bounds)
                    .and_then(|p| self.scrollbar_under(bounds, state.scroll_offset, p));
                if new_sb_hover != state.hovered_scrollbar {
                    state.hovered_scrollbar = new_sb_hover;
                    shell.request_redraw();
                }
                // Track which header sub-region (if any) the cursor is over,
                // so the label cell / resize handle can paint a hover bg.
                let new_hh = cursor
                    .position_over(bounds)
                    .and_then(|p| self.header_hit(bounds, state.scroll_offset.x, p));
                if new_hh != state.hovered_header {
                    state.hovered_header = new_hh;
                    shell.request_redraw();
                }
                let body = self.body_bounds(bounds);
                let new_hover = cursor.position_over(bounds).and_then(|p| {
                    if self.over_scrollbar(bounds, state.scroll_offset, p) {
                        return None;
                    }
                    if !body.contains(p) {
                        return None;
                    }
                    let local_y = (p.y - body.y) + state.scroll_offset.y;
                    if local_y < 0.0 {
                        return None;
                    }
                    let row = (local_y / self.row_height) as usize;
                    if row >= self.n_rows() {
                        None
                    } else {
                        Some(row)
                    }
                });
                if new_hover != state.hovered_row {
                    state.hovered_row = new_hover;
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(p) = cursor.position_over(bounds) else {
                    return;
                };
                // ── Header ────────────────────────────────────────────────
                if let Some((col, region)) = self.header_hit(bounds, state.scroll_offset.x, p) {
                    match region {
                        HeaderRegion::Label => {
                            if let Some(cb) = &self.on_sort {
                                shell.publish(cb(col));
                            }
                        }
                        HeaderRegion::FilterOpen => {
                            if let Some(cb) = &self.on_open_filter {
                                shell.publish(cb(col));
                            }
                        }
                        HeaderRegion::FilterBadge => {
                            if let Some(cb) = &self.on_clear_filter {
                                shell.publish(cb(col));
                            }
                        }
                        HeaderRegion::Resize => {
                            let now = std::time::Instant::now();
                            let is_double = state.last_resize_click.is_some_and(|(c, t)| {
                                c == col && now.duration_since(t).as_millis() < DOUBLE_CLICK_MS
                            });
                            if is_double {
                                if let Some(cb) = &self.on_reset_column_width {
                                    shell.publish(cb(col));
                                }
                                state.last_resize_click = None;
                            } else {
                                if let Some(cb) = &self.on_start_resize {
                                    shell.publish(cb(col));
                                }
                                state.last_resize_click = Some((col, now));
                            }
                        }
                    }
                    shell.capture_event();
                    return;
                }
                // ── Vertical scrollbar ────────────────────────────────────
                if let Some((track, thumb)) =
                    self.vertical_scrollbar(bounds, state.scroll_offset.y)
                {
                    if track.contains(p) {
                        if thumb.contains(p) {
                            state.dragging = Some(ScrollbarDrag {
                                axis: Axis::Vertical,
                                start_cursor: p,
                                start_offset: state.scroll_offset,
                            });
                        } else {
                            // Click outside thumb on the track — snap thumb
                            // centre to cursor.
                            let body = self.body_bounds(bounds);
                            let total_h = self.total_height();
                            let max_off = (total_h - body.height).max(1.0);
                            let travel = (body.height - thumb.height).max(1.0);
                            let target_thumb_y = (p.y - thumb.height / 2.0)
                                .clamp(body.y, body.y + travel);
                            let frac = (target_thumb_y - body.y) / travel;
                            let new_y = frac * max_off;
                            self.apply_scroll(
                                state,
                                bounds,
                                state.scroll_offset.x,
                                new_y,
                                shell,
                            );
                        }
                        shell.capture_event();
                        return;
                    }
                }
                // ── Horizontal scrollbar ──────────────────────────────────
                if let Some((track, thumb)) =
                    self.horizontal_scrollbar(bounds, state.scroll_offset.x)
                {
                    if track.contains(p) {
                        if thumb.contains(p) {
                            state.dragging = Some(ScrollbarDrag {
                                axis: Axis::Horizontal,
                                start_cursor: p,
                                start_offset: state.scroll_offset,
                            });
                        } else {
                            let body = self.body_bounds(bounds);
                            let total_w = self.total_width();
                            let max_off = (total_w - body.width).max(1.0);
                            let travel = (body.width - thumb.width).max(1.0);
                            let target_thumb_x = (p.x - thumb.width / 2.0)
                                .clamp(body.x, body.x + travel);
                            let frac = (target_thumb_x - body.x) / travel;
                            let new_x = frac * max_off;
                            self.apply_scroll(
                                state,
                                bounds,
                                new_x,
                                state.scroll_offset.y,
                                shell,
                            );
                        }
                        shell.capture_event();
                        return;
                    }
                }
                // ── Row select ────────────────────────────────────────────
                let body = self.body_bounds(bounds);
                if !body.contains(p) {
                    return;
                }
                let local_y = (p.y - body.y) + state.scroll_offset.y;
                if local_y < 0.0 {
                    return;
                }
                let row = (local_y / self.row_height) as usize;
                if row >= self.n_rows() {
                    return;
                }
                if let Some(cb) = &self.on_select {
                    shell.publish(cb(row));
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.dragging.is_some() {
                    state.dragging = None;
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                if !cursor.is_over(bounds) {
                    return;
                }
                let body = self.body_bounds(bounds);
                let page_rows = (body.height / self.row_height).floor() as i32;
                let new_y = match key {
                    keyboard::Key::Named(key::Named::PageUp) => {
                        state.scroll_offset.y - (page_rows as f32 * self.row_height)
                    }
                    keyboard::Key::Named(key::Named::PageDown) => {
                        state.scroll_offset.y + (page_rows as f32 * self.row_height)
                    }
                    keyboard::Key::Named(key::Named::Home) => 0.0,
                    keyboard::Key::Named(key::Named::End) => {
                        (self.total_height() - body.height).max(0.0)
                    }
                    _ => return,
                };
                if self.apply_scroll(state, bounds, state.scroll_offset.x, new_y, shell) {
                    shell.capture_event();
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        if let Some(p) = cursor.position_over(bounds) {
            if let Some((_col, region)) = self.header_hit(bounds, state.scroll_offset.x, p) {
                if region == HeaderRegion::Resize {
                    return mouse::Interaction::ResizingHorizontally;
                }
            }
        }

        if cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let body = self.body_bounds(bounds);
        let off = state.scroll_offset;

        if self.n_rows() == 0 || self.n_cols() == 0 {
            return;
        }

        // Clip rendering to the body (excludes scrollbar reservation strips
        // so cells never paint underneath the idle scrollbar) and intersect
        // with the renderer viewport.
        let clip = body.intersection(viewport).unwrap_or(body);

        // Width of the actual content (sum of column widths) projected into
        // the body — never wider than the body itself. Ensures the row
        // backgrounds don't extend past where the column header bar ends.
        let total_w = self.total_width();
        let content_visible_w = (total_w - off.x).clamp(0.0, body.width);

        // Backdrop — paint the table body with the default zebra-base colour
        // so columns that don't fully fill the row leave a clean background.
        let backdrop = Rectangle {
            x: clip.x,
            y: clip.y,
            width: content_visible_w.min(clip.width),
            height: clip.height,
        };
        renderer.fill_quad(
            renderer::Quad {
                bounds: backdrop,
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(color!(0x1e1b17)),
        );

        // ── Visible row range ──────────────────────────────────────────────
        let first_row = ((off.y / self.row_height).floor() as usize).min(self.n_rows());
        let last_row = (((off.y + body.height) / self.row_height).ceil() as usize)
            .min(self.n_rows());

        // ── Visible column range (cumulative x prefix sum) ─────────────────
        let n_cols = self.n_cols();
        let mut col_x: Vec<f32> = Vec::with_capacity(n_cols + 1);
        let mut acc = 0.0f32;
        col_x.push(0.0);
        for c in 0..n_cols {
            acc += self.col_width(c);
            col_x.push(acc);
        }
        let first_col = col_x
            .partition_point(|&x| x <= off.x)
            .saturating_sub(1)
            .min(n_cols.saturating_sub(1));
        let last_col = col_x
            .partition_point(|&x| x < off.x + body.width)
            .min(n_cols);

        // Clip data cells to everything right of the frozen id column so
        // horizontally-scrolled cells don't paint over it.
        let data_clip = clip
            .intersection(&self.data_area(body))
            .unwrap_or(self.data_area(body));

        for row_idx in first_row..last_row {
            let y = body.y + (row_idx as f32 * self.row_height) - off.y;
            let flags = (self.row_flags)(row_idx);
            let is_hovered = state.hovered_row == Some(row_idx);

            // Row background — clipped to the actual content width so the
            // zebra never extends past the column header bar's right edge.
            let row_w = content_visible_w.min(clip.width);
            let row_y = body.y + (row_idx as f32 * self.row_height) - off.y;
            let bg_y = row_y.max(body.y);
            let bg_height = (row_y + self.row_height).min(body.y + body.height) - bg_y;
            if bg_height > 0.0 {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: clip.x,
                            y: bg_y, // y,
                            width: row_w,
                            height: bg_height, // height: self.row_height,
                        },
                        border: Border::default(),
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    Background::Color(row_bg(row_idx, flags, is_hovered)),
                );
            }

            // ── Data cells (col_idx >= 1) ─────────────────────────────────
            for col_idx in first_col.max(1)..last_col {
                let cell_x = bounds.x + col_x[col_idx] - off.x;
                let cell_w = self.col_width(col_idx);

                let value = match self.cell_value(row_idx, col_idx) {
                    Some(v) if !v.is_empty() => v,
                    _ => continue,
                };

                let key = ParagraphKey::new(&value, self.text_size, cell_w, self.font);
                let paragraph = self.cache.get_or_insert(key, || {
                    Paragraph::with_text(text::Text {
                        content: value.as_str(),
                        bounds: Size::new(cell_w, self.row_height),
                        size: Pixels(self.text_size),
                        line_height: text::LineHeight::default(),
                        font: self.font,
                        align_x: text::Alignment::Default,
                        align_y: alignment::Vertical::Top,
                        shaping: text::Shaping::Advanced,
                        wrapping: text::Wrapping::None,
                    })
                });

                let cell_inner = Rectangle {
                    x: cell_x + self.cell_padding_x,
                    y,
                    width: (cell_w - self.cell_padding_x * 2.0).max(0.0),
                    height: self.row_height,
                };
                let position = cell_inner.anchor(
                    paragraph.min_bounds(),
                    alignment::Horizontal::Left,
                    alignment::Vertical::Center,
                );
                // Per-cell clip rect: restrict the rendered text to this
                // cell's column width so long strings don't spill over the
                // next cell. Intersected with the data area so the frozen
                // id column never gets painted over either.
                let cell_clip = data_clip
                    .intersection(&Rectangle {
                        x: cell_x,
                        y,
                        width: cell_w,
                        height: self.row_height,
                    })
                    .unwrap_or(Rectangle {
                        x: cell_x,
                        y,
                        width: 0.0,
                        height: 0.0,
                    });
                <iced::Renderer as text::Renderer>::fill_paragraph(
                    renderer,
                    &paragraph,
                    position,
                    cell_text_color(flags),
                    cell_clip,
                );
            }

            // Row border (selection / highlight outline). Drawn over data
            // cells but under the frozen id column. Clipped to the same
            // content width as the row background so it doesn't trail past
            // the header bar.
            if let Some((color, width)) = row_border(flags) {
                let border_y = row_y.max(body.y);
                let border_h = (row_y + self.row_height).min(body.y + body.height) - border_y;
                if border_h > 0.0 {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle {
                                x: clip.x,
                                y: border_y,
                                width: content_visible_w.min(clip.width),
                                height: border_h,
                            },
                            border: Border {
                                color,
                                width,
                                radius: 0.into(),
                            },
                            shadow: Shadow::default(),
                            snap: true,
                        },
                        Background::Color(Color::TRANSPARENT),
                    );
                }
            }
        }

        // ── Frozen id column ──────────────────────────────────────────────
        // Painted last so data cells can scroll behind it without bleeding
        // their zebra-stripe backgrounds through.
        let id_x = bounds.x;
        let id_w = self.id_col_width.min(bounds.width);
        for row_idx in first_row..last_row {
            let y = body.y + (row_idx as f32 * self.row_height) - off.y;
            let id_y = body.y + (row_idx as f32 * self.row_height) - off.y;
            let id_bg_y = id_y.max(body.y);
            let id_bg_h = (id_y + self.row_height).min(body.y + body.height) - id_bg_y;
            let flags = (self.row_flags)(row_idx);
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: id_x,
                        y: id_bg_y,
                        width: id_w,
                        height: id_bg_h,
                    },
                    border: Border {
                        color: color!(0x3d2b1f),
                        width: 0.5,
                        radius: 0.into(),
                    },
                    shadow: Shadow::default(),
                    snap: true,
                },
                Background::Color(id_cell_bg(flags)),
            );

            let value = match self.cell_value(row_idx, 0) {
                Some(v) if !v.is_empty() => v,
                _ => continue,
            };
            let key = ParagraphKey::new(&value, self.text_size, id_w, self.font);
            let paragraph = self.cache.get_or_insert(key, || {
                Paragraph::with_text(text::Text {
                    content: value.as_str(),
                    bounds: Size::new(id_w, self.row_height),
                    size: Pixels(self.text_size),
                    line_height: text::LineHeight::default(),
                    font: self.font,
                    align_x: text::Alignment::Default,
                    align_y: alignment::Vertical::Top,
                    shaping: text::Shaping::Advanced,
                    wrapping: text::Wrapping::None,
                })
            });
            let id_inner = Rectangle {
                x: id_x + self.cell_padding_x,
                y,
                width: (id_w - self.cell_padding_x * 2.0).max(0.0),
                height: self.row_height,
            };
            let position = id_inner.anchor(
                paragraph.min_bounds(),
                alignment::Horizontal::Left,
                alignment::Vertical::Center,
            );
            let id_clip = clip
                .intersection(&Rectangle {
                    x: id_x,
                    y: body.y,
                    width: id_w,
                    height: body.height,
                })
                .unwrap_or(Rectangle {
                    x: id_x,
                    y: body.y,
                    width: id_w,
                    height: body.height,
                });
            <iced::Renderer as text::Renderer>::fill_paragraph(
                renderer,
                &paragraph,
                position,
                id_text_color(flags),
                id_clip,
            );

            // Re-paint the row's selection/highlight border over the id cell
            // so the gold outline isn't broken at the frozen-column boundary.
            if let Some((border_color, border_width)) = row_border(flags) {
                let border_y = y.max(body.y);
                let border_h = (y + self.row_height).min(body.y + body.height) - border_y;
                if border_h > 0.0 {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle {
                                x: id_x,
                                y: border_y,
                                width: id_w,
                                height: border_h,
                            },
                            border: Border {
                                color: border_color,
                                width: border_width,
                                radius: 0.into(),
                            },
                            shadow: Shadow::default(),
                            snap: true,
                        },
                        Background::Color(Color::TRANSPARENT),
                    );
                }
            }
        }

        // ── Header ─────────────────────────────────────────────────────────
        // The header is painted after the body so it always appears on top
        // of any partially-visible top row. It consists of a full-width
        // backdrop, per-data-column cells (label + sort indicator + filter
        // icons + resize handle) that share `off.x` with the body, and a
        // frozen `#` cell on the left that doesn't horizontally scroll.
        let header = self.header_bounds(bounds);
        let header_clip = header.intersection(viewport).unwrap_or(header);
        renderer.fill_quad(
            renderer::Quad {
                bounds: header,
                border: Border {
                    color: color!(0x4a3728),
                    width: 1.0,
                    radius: 0.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(color!(0x1c1813)),
        );
        // Data-area clip (right of frozen id column) for scrolling header
        // cells.
        let header_data_rect = Rectangle {
            x: bounds.x + id_w,
            y: header.y,
            width: (bounds.width - id_w).max(0.0),
            height: header.height,
        };
        let header_data_clip = header_clip
            .intersection(&header_data_rect)
            .unwrap_or(header_data_rect);

        for col_idx in first_col.max(1)..last_col {
            let col_l_screen = bounds.x + col_x[col_idx] - off.x;
            let col_w = self.col_width(col_idx);
            let data_col = col_idx - 1;
            let column = &self.columns[data_col];

            let resize_l = col_l_screen + col_w - RESIZE_HANDLE_WIDTH;
            let filter_btn_l = resize_l - FILTER_ICON_WIDTH;
            let filter_badge_l = if column.has_filter {
                filter_btn_l - FILTER_BADGE_WIDTH
            } else {
                filter_btn_l
            };
            let label_r = filter_badge_l;

            // Hover bg on the label area.
            let label_hovered = state
                .hovered_header
                .is_some_and(|(c, r)| c == data_col && r == HeaderRegion::Label);
            if label_hovered {
                if let Some(r) = header_data_clip.intersection(&Rectangle {
                    x: col_l_screen,
                    y: header.y,
                    width: (label_r - col_l_screen).max(0.0),
                    height: header.height,
                }) {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: r,
                            border: Border::default(),
                            shadow: Shadow::default(),
                            snap: true,
                        },
                        Background::Color(color!(0x2d2218)),
                    );
                }
            }

            // Label + sort indicator.
            let sort_suffix = match column.sort {
                Some(true) => " ▲",
                Some(false) => " ▼",
                None => "",
            };
            let label = if sort_suffix.is_empty() {
                column.label.clone()
            } else {
                format!("{}{}", column.label, sort_suffix)
            };
            let avail_label_w = (label_r - col_l_screen - self.cell_padding_x * 2.0).max(0.0);
            if avail_label_w > 0.0 {
                let key = ParagraphKey::new(&label, self.text_size, avail_label_w, self.font);
                let para = self.cache.get_or_insert(key, || {
                    Paragraph::with_text(text::Text {
                        content: label.as_str(),
                        bounds: Size::new(avail_label_w, header.height),
                        size: Pixels(self.text_size),
                        line_height: text::LineHeight::default(),
                        font: self.font,
                        align_x: text::Alignment::Default,
                        align_y: alignment::Vertical::Top,
                        shaping: text::Shaping::Advanced,
                        wrapping: text::Wrapping::None,
                    })
                });
                let inner = Rectangle {
                    x: col_l_screen + self.cell_padding_x,
                    y: header.y,
                    width: avail_label_w,
                    height: header.height,
                };
                let pos = inner.anchor(
                    para.min_bounds(),
                    alignment::Horizontal::Left,
                    alignment::Vertical::Center,
                );
                let cell_clip = header_data_clip
                    .intersection(&Rectangle {
                        x: col_l_screen,
                        y: header.y,
                        width: (label_r - col_l_screen).max(0.0),
                        height: header.height,
                    })
                    .unwrap_or(Rectangle {
                        x: col_l_screen,
                        y: header.y,
                        width: 0.0,
                        height: 0.0,
                    });
                <iced::Renderer as text::Renderer>::fill_paragraph(
                    renderer,
                    &para,
                    pos,
                    color!(0xb8a898),
                    cell_clip,
                );
            }

            // Filter badge ◼ — only when a column filter is active.
            if column.has_filter {
                draw_centered_glyph(
                    renderer,
                    &self.cache,
                    "◼",
                    8.0,
                    self.font,
                    Rectangle {
                        x: filter_badge_l,
                        y: header.y,
                        width: FILTER_BADGE_WIDTH,
                        height: header.height,
                    },
                    color!(0xffd700),
                    header_data_clip,
                );
            }

            // Filter-open ▾.
            draw_centered_glyph(
                renderer,
                &self.cache,
                "▾",
                8.0,
                self.font,
                Rectangle {
                    x: filter_btn_l,
                    y: header.y,
                    width: FILTER_ICON_WIDTH,
                    height: header.height,
                },
                color!(0xb8a898),
                header_data_clip,
            );

            // Resize handle strip — slightly brighter on hover.
            let resize_hovered = state
                .hovered_header
                .is_some_and(|(c, r)| c == data_col && r == HeaderRegion::Resize);
            let handle_color = if resize_hovered {
                color!(0x6a5238)
            } else {
                color!(0x4a3728)
            };
            if let Some(r) = header_data_clip.intersection(&Rectangle {
                x: resize_l,
                y: header.y,
                width: RESIZE_HANDLE_WIDTH,
                height: header.height,
            }) {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: r,
                        border: Border::default(),
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    Background::Color(handle_color),
                );
            }
        }

        // Frozen `#` cell — painted last so scrolling header cells can't
        // bleed into it horizontally.
        let id_header = Rectangle {
            x: bounds.x,
            y: header.y,
            width: id_w,
            height: header.height,
        };
        renderer.fill_quad(
            renderer::Quad {
                bounds: id_header,
                border: Border {
                    color: color!(0x3d2b1f),
                    width: 1.0,
                    radius: 0.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(color!(0x171411)),
        );
        let key = ParagraphKey::new("#", self.text_size, id_w, self.font);
        let para = self.cache.get_or_insert(key, || {
            Paragraph::with_text(text::Text {
                content: "#",
                bounds: Size::new(id_w, header.height),
                size: Pixels(self.text_size),
                line_height: text::LineHeight::default(),
                font: self.font,
                align_x: text::Alignment::Default,
                align_y: alignment::Vertical::Top,
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::None,
            })
        });
        let id_inner = Rectangle {
            x: bounds.x + self.cell_padding_x,
            y: header.y,
            width: (id_w - self.cell_padding_x * 2.0).max(0.0),
            height: header.height,
        };
        let pos = id_inner.anchor(
            para.min_bounds(),
            alignment::Horizontal::Left,
            alignment::Vertical::Center,
        );
        <iced::Renderer as text::Renderer>::fill_paragraph(
            renderer,
            &para,
            pos,
            color!(0x6a5e54),
            id_header.intersection(viewport).unwrap_or(id_header),
        );

        // ── Scrollbars ─────────────────────────────────────────────────────
        // A scrollbar is "active" when the user is hovering its track or
        // currently dragging it — in either case the thumb fattens and the
        // colour brightens to telegraph that it's grabbable.
        let active_axis = state
            .dragging
            .map(|d| d.axis)
            .or(state.hovered_scrollbar);
        // Scrollbars live in the strips reserved on the right/bottom of the
        // full bounds (outside `body`). Pass `bounds` and `body` separately
        // so the scrollbar draw can position the track on the strip while
        // sizing the thumb against the body's visible viewport.
        draw_scrollbars(
            renderer,
            bounds,
            body,
            off,
            self.total_width(),
            self.total_height(),
            active_axis,
        );
    }
}

/// Draw a single glyph centered inside `bounds` using `cache` to avoid
/// re-shaping. Used for the small filter icons (`◼`, `▾`) in column headers.
#[allow(clippy::too_many_arguments)]
fn draw_centered_glyph(
    renderer: &mut iced::Renderer,
    cache: &ParagraphCache,
    glyph: &str,
    size: f32,
    font: Font,
    bounds: Rectangle,
    color: Color,
    clip: Rectangle,
) {
    let key = ParagraphKey::new(glyph, size, bounds.width, font);
    let para = cache.get_or_insert(key, || {
        Paragraph::with_text(text::Text {
            content: glyph,
            bounds: Size::new(bounds.width, bounds.height),
            size: Pixels(size),
            line_height: text::LineHeight::default(),
            font,
            align_x: text::Alignment::Center,
            align_y: alignment::Vertical::Top,
            shaping: text::Shaping::Advanced,
            wrapping: text::Wrapping::None,
        })
    });
    let pos = bounds.anchor(
        para.min_bounds(),
        alignment::Horizontal::Center,
        alignment::Vertical::Center,
    );
    let cell_clip = clip.intersection(&bounds).unwrap_or(bounds);
    <iced::Renderer as text::Renderer>::fill_paragraph(renderer, &para, pos, color, cell_clip);
}

/// Paint vertical and horizontal scrollbar thumbs along the right and bottom
/// edges of `bounds` to reflect `off` against the total content size.
///
/// When `active_axis` matches an axis, that scrollbar's thumb is drawn 1.5×
/// thicker and a few shades lighter so the user sees it's grabbable.
fn draw_scrollbars(
    renderer: &mut iced::Renderer,
    bounds: Rectangle,
    body: Rectangle,
    off: Vector,
    total_w: f32,
    total_h: f32,
    active_axis: Option<Axis>,
) {
    let track_color = color!(0x141210);
    let thumb_idle = color!(0x5d4037);
    let thumb_active = color!(0xB97024);
    let border_idle = color!(0x5d4037);
    let border_active = color!(0xB97024);

    if total_h > body.height {
        let track = Rectangle {
            x: bounds.x + bounds.width - SCROLLBAR_THICKNESS,
            y: body.y,
            width: SCROLLBAR_THICKNESS,
            height: body.height,
        };
        let thumb_h = (body.height / total_h * body.height).max(20.0);
        let max_off = (total_h - body.height).max(1.0);
        let thumb_y = body.y + (off.y / max_off) * (body.height - thumb_h);

        let active = active_axis == Some(Axis::Vertical);
        let extra = if active { SCROLLBAR_THICKNESS * 0.5 } else { 0.0 };
        let thumb_w = SCROLLBAR_THICKNESS - 2.0 + extra;
        // Anchor the fattened thumb to the right edge so it grows leftward
        // into the table rather than off-screen.
        let thumb_x = track.x + 1.0 - extra;

        renderer.fill_quad(
            renderer::Quad {
                bounds: track,
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(track_color),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: thumb_x,
                    y: thumb_y,
                    width: thumb_w,
                    height: thumb_h,
                },
                border: Border {
                    color: if active { border_active } else { border_idle },
                    width: if active { 1.0 } else { 0.5 },
                    radius: 0.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(if active { thumb_active } else { thumb_idle }),
        );
    }

    if total_w > body.width {
        let track = Rectangle {
            x: bounds.x,
            y: bounds.y + bounds.height - SCROLLBAR_THICKNESS,
            width: body.width,
            height: SCROLLBAR_THICKNESS,
        };
        let thumb_w = (body.width / total_w * body.width).max(20.0);
        let max_off = (total_w - body.width).max(1.0);
        let thumb_x = bounds.x + (off.x / max_off) * (body.width - thumb_w);

        let active = active_axis == Some(Axis::Horizontal);
        let extra = if active { SCROLLBAR_THICKNESS * 0.5 } else { 0.0 };
        let thumb_h = SCROLLBAR_THICKNESS - 2.0 + extra;
        let thumb_y = track.y + 1.0 - extra;

        renderer.fill_quad(
            renderer::Quad {
                bounds: track,
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(track_color),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: thumb_x,
                    y: thumb_y,
                    width: thumb_w,
                    height: thumb_h,
                },
                border: Border {
                    color: if active { border_active } else { border_idle },
                    width: if active { 1.0 } else { 0.5 },
                    radius: 0.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(if active { thumb_active } else { thumb_idle }),
        );
    }
}

impl<'a, Message, Theme> From<TableWidget<'a, Message>>
    for Element<'a, Message, Theme, iced::Renderer>
where
    Theme: 'a,
    Message: 'a,
{
    fn from(w: TableWidget<'a, Message>) -> Self {
        Element::new(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_flags(_: usize) -> RowFlags {
        RowFlags::default()
    }

    fn col(width_px: f32) -> TableColumn {
        TableColumn {
            width_px,
            label: String::new(),
            sort: None,
            has_filter: false,
        }
    }

    #[test]
    fn empty_table_does_not_panic() {
        let cache = ParagraphCache::default();
        let _w: TableWidget<'_, ()> =
            TableWidget::new(&[], &[], vec![], 42.0, no_flags, 24.0, cache);
    }

    #[test]
    fn total_dimensions_include_id_column() {
        let cache = ParagraphCache::default();
        let display: Vec<Vec<String>> = vec![vec!["a".into(), "b".into()]; 5];
        let filtered: Vec<usize> = (0..5).collect();
        let cols = vec![col(100.0), col(200.0)];
        let w: TableWidget<'_, ()> =
            TableWidget::new(&display, &filtered, cols, 42.0, no_flags, 24.0, cache);
        assert_eq!(w.total_width(), 42.0 + 100.0 + 200.0);
        assert_eq!(w.total_height(), 5.0 * 24.0);
    }

    #[test]
    fn cell_value_id_column_uses_orig_idx() {
        let display = vec![vec!["a".into()]; 3];
        let filtered = vec![2, 0, 1];
        let cols = vec![col(100.0)];
        let cache = ParagraphCache::default();
        let w: TableWidget<'_, ()> =
            TableWidget::new(&display, &filtered, cols, 42.0, no_flags, 24.0, cache);
        assert_eq!(w.cell_value(0, 0).as_deref(), Some("3"));
        assert_eq!(w.cell_value(1, 0).as_deref(), Some("1"));
        assert_eq!(w.cell_value(0, 1).as_deref(), Some("a"));
    }

    #[test]
    fn sync_external_clamps_to_content() {
        let cache = ParagraphCache::default();
        let display: Vec<Vec<String>> = vec![vec!["a".into()]; 100];
        let filtered: Vec<usize> = (0..100).collect();
        let cols = vec![col(100.0)];
        let w: TableWidget<'_, ()> =
            TableWidget::new(&display, &filtered, cols, 42.0, no_flags, 24.0, cache)
                .external_offset(0.0, 100_000.0);
        let mut state = State::default();
        let bounds = Size::new(200.0, 240.0);
        w.sync_external(&mut state, bounds);
        // total_h = 100 * 24 = 2400. Header (= row_height = 24) reserves the
        // top of the bounds; vertical scrollbar reserves 8 px on width but
        // content fits horizontally so no horizontal-scrollbar strip is
        // taken from height. body.height = 240 - 24 = 216; max_y = 2184.
        assert_eq!(state.scroll_offset.y, 2184.0);
        assert_eq!(state.last_external, Some(Vector::new(0.0, 100_000.0)));
    }

    #[test]
    fn sync_external_idempotent() {
        let cache = ParagraphCache::default();
        let display: Vec<Vec<String>> = vec![vec!["a".into()]; 50];
        let filtered: Vec<usize> = (0..50).collect();
        let cols = vec![col(100.0)];
        let w: TableWidget<'_, ()> =
            TableWidget::new(&display, &filtered, cols, 42.0, no_flags, 24.0, cache)
                .external_offset(10.0, 20.0);
        let mut state = State::default();
        let bounds = Size::new(200.0, 240.0);
        w.sync_external(&mut state, bounds);
        // Now mutate scroll_offset (simulating wheel scroll) and call again
        // with the same external — the wheel value must stick.
        state.scroll_offset.y = 50.0;
        w.sync_external(&mut state, bounds);
        assert_eq!(state.scroll_offset.y, 50.0);
    }

    #[test]
    fn cell_text_color_priority() {
        // current_highlight beats highlighted beats selected.
        let f = RowFlags {
            current_highlight: true,
            highlighted: true,
            selected: true,
        };
        assert_eq!(cell_text_color(f), color!(0xffffff));
    }
}
