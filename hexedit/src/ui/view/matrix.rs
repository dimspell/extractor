//! Custom Iced widget rendering the virtualized hex matrix.
//!
//! Layout (left → right): address gutter, hex bytes (grouped 8 with a small
//! gap), ASCII gutter, scrollbar. Only rows in the viewport are touched per
//! frame; everything else is virtual.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
use iced::advanced::layout::{Layout, Limits, Node};
use iced::advanced::renderer;
use iced::advanced::text::{self, Paragraph as _};
use iced::advanced::widget::{tree, Tree, Widget};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::keyboard::{self, key};
use iced::mouse;
use iced::{
    alignment, color, Background, Border, Color, Element, Event, Font, Length, Pixels, Point,
    Rectangle, Shadow, Size,
};

use crate::coloring::{default_byte_colors, ColorScheme};
use crate::domain::byte_stats::entropy_to_color;
use crate::domain::write_mode::WriteMode;
use crate::pattern::{pattern_bg, pattern_fg};
use crate::selection::{NavDir, Selection};
use crate::ui::view::minimap::{self, MINIMAP_WIDTH};
use gui_widgets::components::paragraph_cache::{ParagraphCache, ParagraphKey};

type Paragraph = GraphicsParagraph;

/// Default cell metrics. Tuned for 11px monospace.
const TEXT_SIZE: f32 = 11.0;
const ROW_HEIGHT: f32 = 16.0;
const HEX_CELL_WIDTH: f32 = 20.0;
const ASCII_CELL_WIDTH: f32 = 9.0;
const GROUP_GAP: f32 = 8.0;
const COLUMN_GAP: f32 = 12.0;
const ANN_COL_GAP: f32 = 16.0;

/// Height of the fixed column header row above the hex area. Same as a
/// data row so labels align vertically with the first row beneath them.
const HEADER_HEIGHT: f32 = 16.0;

/// Maximum width of the annotation column when computed from content.
const MAX_ANN_COL_WIDTH: f32 = 400.0;
/// Minimum annotation column width when no annotations exist.
const MIN_ANN_COL_WIDTH: f32 = 200.0;
const SCROLLBAR_THICKNESS: f32 = 10.0;

/// How many extra rows to render above/below the viewport so wheel scrolls
/// don't reveal blank bands during rapid scroll.
const OVERSCAN: u64 = 2;

/// Time window for treating two consecutive clicks as a double-click.
const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(450);

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
    /// Selection cursor from the previous frame, used to detect external
    /// selection changes (e.g. via NavigateToPattern).
    last_cursor: Cell<Option<u64>>,
    /// Row of the cursor that we've already scrolled to.
    last_cursor_row: Cell<Option<u64>>,
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
    pub minimap_cache: RefCell<Option<minimap::MinimapCache>>,
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

pub struct HexMatrix<'a, Message> {
    bytes: &'a [u8],
    bytes_per_row: u8,
    selection: Selection,
    edit: Option<EditView<'a>>,
    dirty: &'a BTreeSet<u64>,
    /// Bytes that already differ from vanilla (load-time + cumulative).
    /// Distinct from `dirty` (= dirtied this session); tinted differently.
    vanilla_diff: &'a BTreeSet<u64>,
    /// Fast lookup: byte address → (pattern id, color_idx).
    patterns: &'a BTreeMap<u64, (usize, u8)>,
    /// Search results: all byte addresses covered by any match.
    search_match_set: &'a BTreeSet<u64>,
    /// Length (in bytes) of the current search query.
    search_query_len: u64,
    /// Start address of the current (navigated-to) match, if any.
    search_current_addr: Option<u64>,
    /// Start addresses of all search matches, for scrollbar markers.
    search_match_starts: &'a [u64],
    /// Precomputed row-address → list of `(pattern_id, annotation)` segments
    /// for the annotation column. Each segment is coloured independently so
    /// only the active pattern's annotation appears highlighted.
    row_annotations: &'a BTreeMap<u64, Vec<(usize, String)>>,
    /// Pattern ids whose span contains the cursor — their annotation segments
    /// are rendered in a brighter colour.
    active_patterns: &'a BTreeSet<usize>,
    /// Pattern ids that should use a darkened background (zebra-striping
    /// within their group — every other pattern in the same group).
    alternate_patterns: BTreeSet<usize>,
    cache: ParagraphCache,
    width: Length,
    height: Length,
    on_select_at: Option<Box<dyn Fn(u64) -> Message + 'a>>,
    on_extend_to: Option<Box<dyn Fn(u64) -> Message + 'a>>,
    on_nav: Option<Box<dyn Fn(NavDir, bool) -> Message + 'a>>,
    on_begin_edit: Option<Box<dyn Fn(u64) -> Message + 'a>>,
    on_edit_type: Option<Box<dyn Fn(char) -> Message + 'a>>,
    on_edit_backspace: Option<Box<dyn Fn() -> Message + 'a>>,
    on_edit_cancel: Option<Box<dyn Fn() -> Message + 'a>>,
    on_delete_byte: Option<Box<dyn Fn() -> Message + 'a>>,
    on_edit_commit: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    on_right_click: Option<Box<dyn Fn(u64) -> Message + 'a>>,
    on_create_pattern: Option<Box<dyn Fn() -> Message + 'a>>,
    on_open_goto: Option<Box<dyn Fn() -> Message + 'a>>,
    on_open_search: Option<Box<dyn Fn() -> Message + 'a>>,
    on_copy_selection: Option<Box<dyn Fn() -> Message + 'a>>,
    on_paste: Option<Box<dyn Fn() -> Message + 'a>>,
    show_decimal: bool,
    on_toggle_addr_format: Option<Box<dyn Fn() -> Message + 'a>>,
    /// Which colour scheme the matrix uses for default byte foreground.
    color_scheme: ColorScheme,
    /// When true, `0x00` bytes use a dim colour regardless of the active scheme.
    dim_nulls: bool,
    /// Active write mode — controls which keystrokes are accepted.
    write_mode: WriteMode,
    /// Pre-computed per-row entropy values for colour bands in the address
    /// gutter. Each entry: `(row_start_addr, entropy)`. When `Some`, a thin
    /// coloured bar is drawn on the left of each row in the gutter.
    entropy_bands: Option<&'a [(u64, f64)]>,
    /// Whether the minimap overview strip is visible.
    show_minimap: bool,
}

impl<'a, Message> HexMatrix<'a, Message> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bytes: &'a [u8],
        bytes_per_row: u8,
        selection: Selection,
        edit: Option<EditView<'a>>,
        dirty: &'a BTreeSet<u64>,
        vanilla_diff: &'a BTreeSet<u64>,
        patterns: &'a BTreeMap<u64, (usize, u8)>,
        search_match_set: &'a BTreeSet<u64>,
        search_query_len: u64,
        search_current_addr: Option<u64>,
        search_match_starts: &'a [u64],
        row_annotations: &'a BTreeMap<u64, Vec<(usize, String)>>,
        active_patterns: &'a BTreeSet<usize>,
        alternate_patterns: BTreeSet<usize>,
        cache: ParagraphCache,
        color_scheme: ColorScheme,
        dim_nulls: bool,
    ) -> Self {
        Self {
            bytes,
            bytes_per_row: bytes_per_row.max(1),
            selection,
            edit,
            dirty,
            vanilla_diff,
            patterns,
            search_match_set,
            search_query_len,
            search_current_addr,
            search_match_starts,
            row_annotations,
            active_patterns,
            alternate_patterns,
            cache,
            color_scheme,
            dim_nulls,
            write_mode: WriteMode::Hex,
            width: Length::Fill,
            height: Length::Fill,
            on_select_at: None,
            on_extend_to: None,
            on_nav: None,
            on_begin_edit: None,
            on_edit_type: None,
            on_edit_backspace: None,
            on_edit_cancel: None,
            on_delete_byte: None,
            on_edit_commit: None,
            on_right_click: None,
            on_create_pattern: None,
            on_open_goto: None,
            on_open_search: None,
            on_copy_selection: None,
            on_paste: None,
            show_decimal: false,
            on_toggle_addr_format: None,
            entropy_bands: None,
            show_minimap: true,
        }
    }

    /// Enable or disable the minimap overview strip.
    pub fn show_minimap(mut self, v: bool) -> Self {
        self.show_minimap = v;
        self
    }

    /// Attach pre-computed per-row entropy bands for the address gutter.
    pub fn entropy_bands(mut self, bands: Option<&'a [(u64, f64)]>) -> Self {
        self.entropy_bands = bands;
        self
    }

    pub fn on_select_at(mut self, f: impl Fn(u64) -> Message + 'a) -> Self {
        self.on_select_at = Some(Box::new(f));
        self
    }

    pub fn on_extend_to(mut self, f: impl Fn(u64) -> Message + 'a) -> Self {
        self.on_extend_to = Some(Box::new(f));
        self
    }

    pub fn on_nav(mut self, f: impl Fn(NavDir, bool) -> Message + 'a) -> Self {
        self.on_nav = Some(Box::new(f));
        self
    }

    pub fn on_begin_edit(mut self, f: impl Fn(u64) -> Message + 'a) -> Self {
        self.on_begin_edit = Some(Box::new(f));
        self
    }

    pub fn on_edit_type(mut self, f: impl Fn(char) -> Message + 'a) -> Self {
        self.on_edit_type = Some(Box::new(f));
        self
    }

    pub fn on_edit_backspace(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_edit_backspace = Some(Box::new(f));
        self
    }

    pub fn on_edit_cancel(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_edit_cancel = Some(Box::new(f));
        self
    }

    pub fn on_delete_byte(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_delete_byte = Some(Box::new(f));
        self
    }

    pub fn on_edit_commit(mut self, f: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_edit_commit = Some(Box::new(f));
        self
    }

    pub fn on_right_click(mut self, f: impl Fn(u64) -> Message + 'a) -> Self {
        self.on_right_click = Some(Box::new(f));
        self
    }

    pub fn on_create_pattern(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_create_pattern = Some(Box::new(f));
        self
    }

    pub fn on_open_goto(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_open_goto = Some(Box::new(f));
        self
    }

    pub fn on_open_search(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_open_search = Some(Box::new(f));
        self
    }

    pub fn on_copy_selection(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_copy_selection = Some(Box::new(f));
        self
    }

    pub fn on_paste(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_paste = Some(Box::new(f));
        self
    }

    pub fn show_decimal(mut self, v: bool) -> Self {
        self.show_decimal = v;
        self
    }

    pub fn on_toggle_addr_format(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_toggle_addr_format = Some(Box::new(f));
        self
    }

    /// Set the active write mode — affects which keystrokes are accepted as
    /// input (hex digits only, or any printable character for text modes).
    pub fn write_mode(mut self, mode: WriteMode) -> Self {
        self.write_mode = mode;
        self
    }

    fn addr_col_width(&self) -> f32 {
        let char_w = 9.0;
        let pad = 16.0;
        let chars = if self.show_decimal {
            let max_addr = self.bytes.len().saturating_sub(1);
            format!("{}", max_addr).len().max(1)
        } else {
            8usize
        };
        chars as f32 * char_w + pad
    }

    fn total_rows(&self) -> u64 {
        let bpr = self.bytes_per_row as u64;
        if self.bytes.is_empty() {
            0
        } else {
            self.bytes.len().div_ceil(bpr as usize) as u64
        }
    }

    fn total_height(&self) -> f32 {
        self.total_rows() as f32 * ROW_HEIGHT
    }

    fn ascii_start_x(&self, bounds_x: f32) -> f32 {
        let bpr = self.bytes_per_row as usize;
        bounds_x
            + self.addr_col_width()
            + (bpr as f32) * HEX_CELL_WIDTH
            + group_count(bpr) as f32 * GROUP_GAP
            + COLUMN_GAP
    }

    /// Dynamic annotation column width: computed from the longest visible
    /// annotation string, capped at [`MAX_ANN_COL_WIDTH`].
    fn annotation_col_width(&self) -> f32 {
        if self.row_annotations.is_empty() {
            return 0.0;
        }
        let max_chars = self
            .row_annotations
            .values()
            .map(|segments| {
                let text_len: usize = segments.iter().map(|(_, t)| t.len()).sum();
                let separators = segments.len().saturating_sub(1) * 3; // " │ " = 3 chars
                text_len + separators
            })
            .max()
            .unwrap_or(0);
        // Estimate pixel width using ASCII_CELL_WIDTH (9px monospace).
        let estimated = max_chars as f32 * ASCII_CELL_WIDTH;
        estimated.clamp(MIN_ANN_COL_WIDTH, MAX_ANN_COL_WIDTH)
    }

    fn annotation_start_x(&self, bounds_x: f32) -> f32 {
        self.ascii_start_x(bounds_x) + (self.bytes_per_row as f32) * ASCII_CELL_WIDTH + ANN_COL_GAP
    }

    /// Total width of the address + hex + ASCII + annotation content area.
    fn total_content_width(&self) -> f32 {
        let bpr = self.bytes_per_row as usize;
        let mut w = self.addr_col_width()
            + (bpr as f32) * HEX_CELL_WIDTH
            + group_count(bpr) as f32 * GROUP_GAP
            + COLUMN_GAP
            + (bpr as f32) * ASCII_CELL_WIDTH;
        if !self.row_annotations.is_empty() {
            w += ANN_COL_GAP + self.annotation_col_width();
        }
        w
    }

    /// Viewport height available for content rows (below the fixed column
    /// header), accounting for horizontal scrollbar and minimap.
    fn content_viewport_h(&self, bounds_h: f32, bounds_w: f32) -> f32 {
        let right_reserved = self.right_strip();
        let needs_hscroll = self.total_content_width() > bounds_w - right_reserved;
        let header_h = HEADER_HEIGHT;
        if needs_hscroll {
            (bounds_h - header_h - SCROLLBAR_THICKNESS).max(0.0)
        } else {
            (bounds_h - header_h).max(0.0)
        }
    }

    /// Total width reserved on the right side (scrollbar + optional minimap).
    fn right_strip(&self) -> f32 {
        if self.show_minimap {
            SCROLLBAR_THICKNESS + MINIMAP_WIDTH
        } else {
            SCROLLBAR_THICKNESS
        }
    }
}

/// Pure helper — clamp `scroll_offset` and compute `[first, last)` visible
/// rows including overscan. Extracted so it can be unit-tested.
pub fn visible_row_range(
    scroll: f32,
    viewport_height: f32,
    row_height: f32,
    total_rows: u64,
    overscan: u64,
) -> std::ops::Range<u64> {
    if total_rows == 0 || row_height <= 0.0 {
        return 0..0;
    }
    let scroll = scroll.max(0.0);
    let raw_first = ((scroll / row_height).floor() as i64 - overscan as i64).max(0) as u64;
    let first = raw_first.min(total_rows);
    let visible = (viewport_height / row_height).ceil() as u64 + overscan * 2 + 1;
    let last = first.saturating_add(visible).min(total_rows);
    first..last
}

/// Clamp `scroll` to `[0, max_scroll]`.
fn clamp_scroll(scroll: f32, total_height: f32, viewport_height: f32) -> f32 {
    let max_off = (total_height - viewport_height).max(0.0);
    scroll.clamp(0.0, max_off)
}

/// Clamp horizontal scroll. `content_w` = total content width,
/// `view_w` = viewport width for content (after reserving scrollbar / address gutter).
fn clamp_scroll_x(scroll: f32, content_w: f32, view_w: f32) -> f32 {
    let max_off = (content_w - view_w).max(0.0);
    scroll.clamp(0.0, max_off)
}

/// Number of complete rows that fit in `viewport_height`. Used both for
/// PageUp/PageDown nav and for "ensure visible" math.
fn page_rows(viewport_height: f32) -> u64 {
    (viewport_height / ROW_HEIGHT).floor().max(1.0) as u64
}

/// Adjust `scroll` to center `addr` in the viewport. Returns the new scroll value.
pub fn ensure_visible(
    scroll: f32,
    addr: u64,
    bytes_per_row: u64,
    viewport_height: f32,
    total_height: f32,
) -> f32 {
    let bpr = bytes_per_row.max(1);
    let row = addr / bpr;
    let row_top = row as f32 * ROW_HEIGHT;
    let row_bot = row_top + ROW_HEIGHT;
    if row_top >= scroll && row_bot <= scroll + viewport_height {
        return clamp_scroll(scroll, total_height, viewport_height);
    }
    let center = row_top - (viewport_height - ROW_HEIGHT) / 2.0;
    clamp_scroll(center, total_height, viewport_height)
}

/// Hit-test: convert a screen point inside `bounds` to a byte address.
/// Considers both the hex column and the ASCII column.
pub fn addr_at(
    point: Point,
    bounds: Rectangle,
    scroll: f32,
    scroll_x: f32,
    bytes_per_row: u8,
    total_len: u64,
    addr_col_width: f32,
) -> Option<u64> {
    if total_len == 0 {
        return None;
    }
    if !bounds.contains(point) {
        return None;
    }
    let bpr = bytes_per_row.max(1) as f32;
    let local_y = (point.y - bounds.y) + scroll;
    if local_y < 0.0 {
        return None;
    }
    let row = (local_y / ROW_HEIGHT) as u64;

    let hex_start = bounds.x + addr_col_width - scroll_x;
    let bpr_usize = bytes_per_row.max(1) as usize;
    let hex_end = hex_start + bpr * HEX_CELL_WIDTH + group_count(bpr_usize) as f32 * GROUP_GAP;
    let ascii_start = hex_end + COLUMN_GAP;
    let ascii_end = ascii_start + bpr * ASCII_CELL_WIDTH;

    let col = if point.x >= hex_start && point.x < hex_end {
        // Account for inter-group gaps when figuring out the column index.
        let mut x = point.x - hex_start;
        let mut col = 0u64;
        for c in 0..bytes_per_row.max(1) as u64 {
            let g = (c / 8) as f32;
            let cell_l = c as f32 * HEX_CELL_WIDTH + g * GROUP_GAP;
            let cell_r = cell_l + HEX_CELL_WIDTH;
            if x < cell_r {
                col = c;
                x = -1.0; // sentinel: found
                break;
            }
            col = c;
        }
        if x >= 0.0 {
            // Past the last cell — clamp.
            col = bytes_per_row.saturating_sub(1) as u64;
        }
        col
    } else if point.x >= ascii_start && point.x < ascii_end {
        ((point.x - ascii_start) / ASCII_CELL_WIDTH) as u64
    } else {
        return None;
    };

    let addr = row * bytes_per_row as u64 + col;
    if addr >= total_len {
        Some(total_len - 1)
    } else {
        Some(addr)
    }
}

fn shape_glyph(cache: &ParagraphCache, glyph: &str, font: Font) -> Paragraph {
    let key = ParagraphKey::new(glyph, TEXT_SIZE, 64.0, font);
    cache.get_or_insert(key, || {
        Paragraph::with_text(text::Text {
            content: glyph,
            bounds: Size::new(64.0, ROW_HEIGHT),
            size: Pixels(TEXT_SIZE),
            line_height: text::LineHeight::default(),
            font,
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Top,
            shaping: text::Shaping::Basic,
            wrapping: text::Wrapping::None,
        })
    })
}

fn ascii_repr(b: u8) -> &'static str {
    // Printable ASCII window. Anything else collapses to a placeholder so
    // the column visually aligns with the hex side.
    const TABLE: [&str; 95] = [
        " ", "!", "\"", "#", "$", "%", "&", "'", "(", ")", "*", "+", ",", "-", ".", "/", "0", "1",
        "2", "3", "4", "5", "6", "7", "8", "9", ":", ";", "<", "=", ">", "?", "@", "A", "B", "C",
        "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U",
        "V", "W", "X", "Y", "Z", "[", "\\", "]", "^", "_", "`", "a", "b", "c", "d", "e", "f", "g",
        "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y",
        "z", "{", "|", "}", "~",
    ];
    if (0x20..0x7F).contains(&b) {
        TABLE[(b - 0x20) as usize]
    } else {
        "·"
    }
}

const HEX_DIGITS: [&str; 16] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F",
];

impl<'a, Message, Theme> Widget<Message, Theme, iced::Renderer> for HexMatrix<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();
        let cursor = self.selection.cursor;
        if state.last_cursor.get() != Some(cursor) {
            state.last_cursor.set(Some(cursor));
        }
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &iced::Renderer, limits: &Limits) -> Node {
        let max = limits.max();
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
        let total_h = self.total_height();
        let total_len = self.bytes.len() as u64;
        let viewport_h = self.content_viewport_h(bounds.height, bounds.width);
    let content_bounds = Rectangle {
        x: bounds.x,
        y: bounds.y + HEADER_HEIGHT,
        width: bounds.width,
        height: viewport_h,
    };

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if !cursor.is_over(bounds) {
                    return;
                }
                let shift = state.shift_pressed.get();
                match delta {
                    mouse::ScrollDelta::Lines { y, x, .. } => {
                        if shift {
                            // Shift redirects vertical wheel to horizontal.
                            let content_w = self.total_content_width();
                            let avail_w = bounds.width - SCROLLBAR_THICKNESS;
                            let sx = state.scroll_x.get();
                            let horiz = if *x != 0.0 { *x } else { *y };
                            let nsx =
                                clamp_scroll_x(sx - horiz * ROW_HEIGHT * 3.0, content_w, avail_w);
                            if (nsx - sx).abs() > f32::EPSILON {
                                state.scroll_x.set(nsx);
                                shell.request_redraw();
                            }
                        } else {
                            let dy = -y * ROW_HEIGHT * 3.0;
                            let so = state.scroll_offset.get();
                            let new = clamp_scroll(so + dy, total_h, viewport_h);
                            if (new - so).abs() > f32::EPSILON {
                                state.scroll_offset.set(new);
                                shell.request_redraw();
                            }
                        }
                        if !shift && *x != 0.0 {
                            let content_w = self.total_content_width();
                            let avail_w = bounds.width - SCROLLBAR_THICKNESS;
                            let sx = state.scroll_x.get();
                            let nsx = clamp_scroll_x(sx - x * ROW_HEIGHT * 3.0, content_w, avail_w);
                            if (nsx - sx).abs() > f32::EPSILON {
                                state.scroll_x.set(nsx);
                                shell.request_redraw();
                            }
                        }
                        shell.capture_event();
                    }
                    mouse::ScrollDelta::Pixels { y, x } => {
                        if shift {
                            let content_w = self.total_content_width();
                            let avail_w = bounds.width - SCROLLBAR_THICKNESS;
                            let sx = state.scroll_x.get();
                            let horiz = if *x != 0.0 { *x } else { *y };
                            let nsx = clamp_scroll_x(sx - horiz, content_w, avail_w);
                            if (nsx - sx).abs() > f32::EPSILON {
                                state.scroll_x.set(nsx);
                                shell.request_redraw();
                            }
                        } else {
                            let so = state.scroll_offset.get();
                            let new = clamp_scroll(so - y, total_h, viewport_h);
                            if (new - so).abs() > f32::EPSILON {
                                state.scroll_offset.set(new);
                                shell.request_redraw();
                            }
                        }
                        if !shift && *x != 0.0 {
                            let content_w = self.total_content_width();
                            let avail_w = bounds.width - SCROLLBAR_THICKNESS;
                            let sx = state.scroll_x.get();
                            let nsx = clamp_scroll_x(sx - x, content_w, avail_w);
                            if (nsx - sx).abs() > f32::EPSILON {
                                state.scroll_x.set(nsx);
                                shell.request_redraw();
                            }
                        }
                        shell.capture_event();
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(p) = cursor.position_over(bounds) else {
                    return;
                };

                // Horizontal scrollbar takes precedence.
                let htrack = hscrollbar_track(bounds);
                let content_w = self.total_content_width();
                let avail_w = bounds.width - SCROLLBAR_THICKNESS;
                let needs_hscroll = content_w > avail_w;
                if needs_hscroll && htrack.contains(p) {
                    let hthumb = hscrollbar_thumb(htrack, state.scroll_x.get(), content_w, avail_w);
                    if hthumb.contains(p) {
                        state.dragging_scrollbar_x = true;
                        state.drag_start_cursor_x = p.x;
                        state.drag_start_offset_x = state.scroll_x.get();
                    } else {
                        let dir = if p.x < hthumb.x { -1.0 } else { 1.0 };
                        let nsx = clamp_scroll_x(
                            state.scroll_x.get() + dir * avail_w,
                            content_w,
                            avail_w,
                        );
                        if (nsx - state.scroll_x.get()).abs() > f32::EPSILON {
                            state.scroll_x.set(nsx);
                            shell.request_redraw();
                        }
                    }
                    shell.capture_event();
                    return;
                }

                // Vertical scrollbar (sits below the column header).
                let content_bounds = Rectangle {
                    x: bounds.x,
                    y: bounds.y + HEADER_HEIGHT,
                    width: bounds.width,
                    height: viewport_h,
                };
                let scrollbar = scrollbar_track(content_bounds, viewport_h);
                if scrollbar.contains(p) && total_h > viewport_h {
                    let thumb = scrollbar_thumb(scrollbar, state.scroll_offset.get(), total_h);
                    if thumb.contains(p) {
                        state.dragging_scrollbar = true;
                        state.drag_start_cursor_y = p.y;
                        state.drag_start_offset = state.scroll_offset.get();
                    } else {
                        let dir = if p.y < thumb.y { -1.0 } else { 1.0 };
                        let new = clamp_scroll(
                            state.scroll_offset.get() + dir * viewport_h,
                            total_h,
                            viewport_h,
                        );
                        state.scroll_offset.set(new);
                        shell.request_redraw();
                    }
                    shell.capture_event();
                    return;
                }

                // Minimap hit-test (when enabled, sits between content and scrollbar).
                if self.show_minimap && total_h > viewport_h {
                    let mm_rect = minimap::minimap_rect(content_bounds, viewport_h, MINIMAP_WIDTH, SCROLLBAR_THICKNESS);
                    if mm_rect.contains(p) {
                        let thumb_r = minimap::minimap_thumb_rect(mm_rect, state.scroll_offset.get(), total_h, viewport_h);
                        if thumb_r.contains(p) {
                            state.dragging_minimap = true;
                            state.drag_start_minimap_y = p.y;
                            state.drag_start_minimap_scroll = state.scroll_offset.get();
                        } else {
                            let new_scroll = minimap::minimap_scroll_from_y(
                                p.y, mm_rect, total_h, viewport_h,
                            );
                            let clamped = clamp_scroll(new_scroll, total_h, viewport_h);
                            state.scroll_offset.set(clamped);
                        }
                        shell.capture_event();
                        return;
                    }
                }

                // Gutter click → toggle address format.
                if p.x >= bounds.x && p.x < bounds.x + self.addr_col_width() {
                    if let Some(cb) = &self.on_toggle_addr_format {
                        shell.publish(cb());
                    }
                    shell.request_redraw();
                    shell.capture_event();
                    return;
                }

                // Header area (above content rows) -> ignore for byte selection.
                if p.y < bounds.y + HEADER_HEIGHT {
                    return;
                }

                // Cell click → selection (and maybe edit on double-click).
                if let Some(addr) = addr_at(
                    p,
                    content_bounds,
                    state.scroll_offset.get(),
                    state.scroll_x.get(),
                    self.bytes_per_row,
                    total_len,
                    self.addr_col_width(),
                ) {
                    let now = Instant::now();
                    let is_double = matches!(
                        (state.last_click_addr, state.last_click_at),
                        (Some(prev), Some(at))
                            if prev == addr && now.duration_since(at) <= DOUBLE_CLICK_WINDOW
                    );
                    state.last_click_addr = Some(addr);
                    state.last_click_at = Some(now);

                    if is_double {
                        if let Some(cb) = &self.on_begin_edit {
                            shell.publish(cb(addr));
                            shell.request_redraw();
                            shell.capture_event();
                            return;
                        }
                    }

                    state.selecting = true;
                    if let Some(cb) = &self.on_select_at {
                        shell.publish(cb(addr));
                    }
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                let Some(p) = cursor.position_over(bounds) else {
                    return;
                };
                // Header area -> no right-click target.
                if p.y < bounds.y + HEADER_HEIGHT {
                    return;
                }
                if let Some(addr) = addr_at(
                    p,
                    content_bounds,
                    state.scroll_offset.get(),
                    state.scroll_x.get(),
                    self.bytes_per_row,
                    total_len,
                    self.addr_col_width(),
                ) {
                    if let Some(cb) = &self.on_right_click {
                        shell.publish(cb(addr));
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                // ── Column-header area for scrollbar-track hover ──────
                let content_bounds = Rectangle {
                    x: bounds.x,
                    y: bounds.y + HEADER_HEIGHT,
                    width: bounds.width,
                    height: viewport_h,
                };

                // Repaint on hover transitions over scrollbar tracks and minimap.
                if cursor.is_over(bounds) {
                    if let Some(p) = cursor.position() {
                        let vtrack = scrollbar_track(content_bounds, viewport_h);
                        let htrack = hscrollbar_track(bounds);
                        let now_hovering = vtrack.contains(p) || htrack.contains(p);
                        if now_hovering != state.hovering_scrollbar.get() {
                            state.hovering_scrollbar.set(now_hovering);
                            shell.request_redraw();
                        }
                        // Minimap hover.
                        if self.show_minimap && total_h > viewport_h {
                            let mm_rect = minimap::minimap_rect(content_bounds, viewport_h, MINIMAP_WIDTH, SCROLLBAR_THICKNESS);
                            let now_mm_hover = mm_rect.contains(p);
                            if now_mm_hover != state.hovering_minimap.get() {
                                state.hovering_minimap.set(now_mm_hover);
                                shell.request_redraw();
                            }
                        }
                    }
                }
                if state.dragging_scrollbar_x {
                    let Some(p) = cursor.position() else { return };
                    let htrack = hscrollbar_track(bounds);
                    let content_w = self.total_content_width();
                    let avail_w = bounds.width - SCROLLBAR_THICKNESS;
                    let thumb_w = hthumb_len(htrack, content_w, avail_w);
                    let travel = (htrack.width - thumb_w).max(1.0);
                    let max_off = (content_w - avail_w).max(1.0);
                    let dx = p.x - state.drag_start_cursor_x;
                    let nsx = state.drag_start_offset_x + dx * (max_off / travel);
                    state.scroll_x.set(clamp_scroll_x(nsx, content_w, avail_w));
                    shell.request_redraw();
                    shell.capture_event();
                    return;
                }
                if state.dragging_scrollbar {
                    let Some(p) = cursor.position() else { return };
                    let scrollbar = scrollbar_track(content_bounds, viewport_h);
                    let thumb_h = thumb_height(scrollbar, total_h);
                    let travel = (scrollbar.height - thumb_h).max(1.0);
                    let max_off = (total_h - viewport_h).max(1.0);
                    let dy = p.y - state.drag_start_cursor_y;
                    let new = state.drag_start_offset + dy * (max_off / travel);
                    state
                        .scroll_offset
                        .set(clamp_scroll(new, total_h, content_bounds.height));
                    shell.request_redraw();
                    shell.capture_event();
                    return;
                }
                if state.dragging_minimap {
                    let Some(p) = cursor.position() else { return };
                    let mm_rect = minimap::minimap_rect(content_bounds, viewport_h, MINIMAP_WIDTH, SCROLLBAR_THICKNESS);
                    let dy = p.y - state.drag_start_minimap_y;
                    let new = state.drag_start_minimap_scroll
                        + minimap::minimap_pixel_to_scroll(dy, mm_rect, total_h, viewport_h);
                    state
                        .scroll_offset
                        .set(clamp_scroll(new, total_h, content_bounds.height));
                    shell.request_redraw();
                    shell.capture_event();
                    return;
                }
                if state.selecting {
                    let Some(p) = cursor.position() else { return };
                    // Do not extend selection into the column header.
                    if p.y < bounds.y + HEADER_HEIGHT {
                        return;
                    }
                    if let Some(addr) = addr_at(
                        p,
                        content_bounds,
                        state.scroll_offset.get(),
                        state.scroll_x.get(),
                        self.bytes_per_row,
                        total_len,
                        self.addr_col_width(),
                    ) {
                        if let Some(cb) = &self.on_extend_to {
                            shell.publish(cb(addr));
                        }
                        shell.request_redraw();
                        shell.capture_event();
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let mut consumed = false;
                if state.dragging_scrollbar {
                    state.dragging_scrollbar = false;
                    consumed = true;
                }
                if state.dragging_scrollbar_x {
                    state.dragging_scrollbar_x = false;
                    consumed = true;
                }
                if state.dragging_minimap {
                    state.dragging_minimap = false;
                    consumed = true;
                }
                if state.selecting {
                    state.selecting = false;
                    consumed = true;
                }
                if consumed {
                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.shift_pressed.set(modifiers.shift());
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modifiers,
                text,
                ..
            }) => {
                if !cursor.is_over(bounds) {
                    return;
                }

                // ── Edit-mode keys take priority ─────────────────────────
                if self.edit.is_some() {
                    match key {
                        keyboard::Key::Named(key::Named::Escape) => {
                            if let Some(cb) = &self.on_edit_cancel {
                                shell.publish(cb());
                                shell.capture_event();
                                return;
                            }
                        }
                        keyboard::Key::Named(key::Named::Enter | key::Named::Tab) => {
                            if let Some(cb) = &self.on_edit_commit {
                                shell.publish(cb(true));
                                shell.capture_event();
                                return;
                            }
                        }
                        keyboard::Key::Named(key::Named::Backspace) => {
                            if let Some(cb) = &self.on_edit_backspace {
                                shell.publish(cb());
                                shell.capture_event();
                                return;
                            }
                        }
                        _ => {}
                    }
                }

                // F2 starts a hex-digit edit at the current cursor (hex mode
                // only — in text mode each character encodes immediately).
                if self.write_mode == WriteMode::Hex
                    && matches!(key, keyboard::Key::Named(key::Named::F2))
                    && self.edit.is_none()
                {
                    if let Some(cb) = &self.on_begin_edit {
                        shell.publish(cb(self.selection.cursor));
                        shell.capture_event();
                        return;
                    }
                }

                // CTRL+E creates a pattern from the current selection.
                if (modifiers.control() || modifiers.command())
                    && matches!(key, keyboard::Key::Character(c) if c.to_lowercase() == "e")
                {
                    if let Some(cb) = &self.on_create_pattern {
                        shell.publish(cb());
                        shell.capture_event();
                        return;
                    }
                }

                // Ctrl+G opens the goto dialog.
                if (modifiers.control() || modifiers.command())
                    && matches!(key, keyboard::Key::Character(c) if c.to_lowercase() == "g")
                {
                    if let Some(cb) = &self.on_open_goto {
                        shell.publish(cb());
                        shell.capture_event();
                        return;
                    }
                }

                // Ctrl+F opens the search overlay.
                if (modifiers.control() || modifiers.command())
                    && matches!(key, keyboard::Key::Character(c) if c.to_lowercase() == "f")
                {
                    if let Some(cb) = &self.on_open_search {
                        shell.publish(cb());
                        shell.capture_event();
                        return;
                    }
                }

                // Ctrl+C copies the selected byte range as hex text.
                if (modifiers.control() || modifiers.command())
                    && matches!(key, keyboard::Key::Character(c) if c.to_lowercase() == "c")
                {
                    if let Some(cb) = &self.on_copy_selection {
                        shell.publish(cb());
                        shell.capture_event();
                        return;
                    }
                }

                // Ctrl+V pastes hex bytes from the clipboard.
                if (modifiers.control() || modifiers.command())
                    && matches!(key, keyboard::Key::Character(c) if c.to_lowercase() == "v")
                {
                    if let Some(cb) = &self.on_paste {
                        shell.publish(cb());
                        shell.capture_event();
                        return;
                    }
                }

                // Character typing: in hex mode only hex-digits are accepted;
                // in text mode any printable character is forwarded so the
                // update handler can encode it.
                //
                // Control and Command are blocked in both modes (they are used
                // for shortcuts such as Ctrl+C/V).  Alt / Option is *only*
                // blocked in hex mode — on macOS, Option is how non-ASCII
                // characters like `ł` or `€` are typed and must be allowed in
                // text mode.
                let mods_blocked = if self.write_mode == WriteMode::Hex {
                    modifiers.control() || modifiers.command() || modifiers.alt()
                } else {
                    modifiers.control() || modifiers.command()
                };
                if !mods_blocked {
                    if let Some(t) = text {
                        let c = if self.write_mode == WriteMode::Hex {
                            first_hex_char(t)
                        } else {
                            first_printable_char(t)
                        };
                        if let Some(c) = c {
                            if self.write_mode == WriteMode::Hex {
                                // Hex mode: existing draft-based editing.
                                if self.edit.is_some() {
                                    if let Some(cb) = &self.on_edit_type {
                                        shell.publish(cb(c));
                                        shell.capture_event();
                                        return;
                                    }
                                } else if !self.bytes.is_empty() {
                                    // Auto-start: behave like F2 then type.
                                    if let Some(begin) = &self.on_begin_edit {
                                        shell.publish(begin(self.selection.cursor));
                                    }
                                    if let Some(typ) = &self.on_edit_type {
                                        shell.publish(typ(c));
                                    }
                                    shell.capture_event();
                                    return;
                                }
                            } else {
                                // Text mode: encode & write immediately.
                                // Only when the buffer is non-empty (hex mode
                                // also guards with the same check above).
                                if !self.bytes.is_empty() {
                                    if let Some(cb) = &self.on_edit_type {
                                        shell.publish(cb(c));
                                        shell.capture_event();
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }

                // ── Text-mode Backspace / Delete ───────────────────────────
                // In hex mode Backspace pops a nibble from the edit draft (see
                // the edit-mode priority block above).  In text mode there is no
                // draft, so Backspace simply moves the cursor back one byte and
                // Delete writes 0x00 at the current position (advancing by one).
                if self.write_mode != WriteMode::Hex {
                    if matches!(key, keyboard::Key::Named(key::Named::Backspace)) {
                        if let Some(cb) = &self.on_edit_backspace {
                            shell.publish(cb());
                            shell.capture_event();
                            return;
                        }
                    } else if matches!(key, keyboard::Key::Named(key::Named::Delete)) {
                        if let Some(cb) = &self.on_delete_byte {
                            shell.publish(cb());
                            shell.capture_event();
                            return;
                        }
                    }
                }

                // ── Navigation ───────────────────────────────────────────
                if modifiers.control() || modifiers.command() {
                    let dir = match key {
                        keyboard::Key::Named(key::Named::Home) => Some(NavDir::DocumentStart),
                        keyboard::Key::Named(key::Named::End) => Some(NavDir::DocumentEnd),
                        _ => None,
                    };
                    if let Some(dir) = dir {
                        self.publish_nav(state, dir, modifiers.shift(), bounds, shell);
                        shell.capture_event();
                    }
                    return;
                }
                let dir = match key {
                    keyboard::Key::Named(key::Named::ArrowLeft) => Some(NavDir::Left),
                    keyboard::Key::Named(key::Named::ArrowRight) => Some(NavDir::Right),
                    keyboard::Key::Named(key::Named::ArrowUp) => Some(NavDir::Up),
                    keyboard::Key::Named(key::Named::ArrowDown) => Some(NavDir::Down),
                    keyboard::Key::Named(key::Named::Home) => Some(NavDir::LineStart),
                    keyboard::Key::Named(key::Named::End) => Some(NavDir::LineEnd),
                    keyboard::Key::Named(key::Named::PageUp) => Some(NavDir::PageUp),
                    keyboard::Key::Named(key::Named::PageDown) => Some(NavDir::PageDown),
                    _ => None,
                };
                if let Some(dir) = dir {
                    self.publish_nav(state, dir, modifiers.shift(), bounds, shell);
                    shell.capture_event();
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Idle
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
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let full_clip = bounds.intersection(viewport).unwrap_or(bounds);

        // Background (full bounds — fill everything).
        renderer.fill_quad(
            renderer::Quad {
                bounds: full_clip,
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(color!(0x14110f)),
        );

        // ── Geometry for the content area below the column header ──────
        let viewport_h = self.content_viewport_h(bounds.height, bounds.width);
        let content_top = bounds.y + HEADER_HEIGHT;
        let content_bounds = Rectangle {
            x: bounds.x,
            y: content_top,
            width: bounds.width,
            height: viewport_h,
        };

        // Clip the content rows to the visible portion of the content area
        // (below header, excluding the vertical scrollbar and minimap strip).
        let content_clip_y = content_top.max(full_clip.y);
        let content_clip_bottom = (content_top + viewport_h).min(full_clip.y + full_clip.height);
        let content_clip = Rectangle {
            x: full_clip.x,
            y: content_clip_y,
            width: (full_clip.width - self.right_strip()).max(0.0),
            height: (content_clip_bottom - content_clip_y).max(0.0),
        };

        // Further clip hex/ASCII cells to exclude the address gutter.
        let cell_clip = Rectangle {
            x: content_clip.x.max(bounds.x + self.addr_col_width()),
            y: content_clip.y,
            width: (content_clip.x + content_clip.width
                - content_clip.x.max(bounds.x + self.addr_col_width()))
            .max(0.0),
            height: content_clip.height,
        };

        let total_rows = self.total_rows();
        let bpr = self.bytes_per_row as usize;
        let total_h = self.total_height();
        let bpr64 = bpr as u64;

        let scroll = if total_h <= viewport_h || total_rows == 0 {
            0.0
        } else {
            let cursor = self.selection.cursor;
            let cursor_row = cursor / bpr64;
            let last = state.last_cursor_row.get();

            if last != Some(cursor_row) {
                state.last_cursor_row.set(Some(cursor_row));
                ensure_visible(
                    state.scroll_offset.get(),
                    cursor,
                    bpr64,
                    viewport_h,
                    total_h,
                )
            } else {
                state.scroll_offset.get()
            }
        };
        state.scroll_offset.set(scroll);

        let scroll_x = state.scroll_x.get();

        let visible = visible_row_range(scroll, viewport_h, ROW_HEIGHT, total_rows, OVERSCAN);

        let font = Font::MONOSPACE;
        let addr_color = color!(0x7a6f64);
        let hex_color = color!(0xd4cabd);
        let ascii_color = color!(0xb8a898);
        let header_color = color!(0x8a7a6a);
        let header_bg = color!(0x1a1614);
        let header_separator = color!(0x2a2218);
        let group_separator_color = color!(0x251f1a);
        let selection_bg = color!(0x3b2a18);
        let cursor_bg = color!(0x6a4a26);
        let selection_text = color!(0xfff4e0);
        let dirty_bg = color!(0x4a1f1a);
        let dirty_text = color!(0xff9d6e);
        let diff_bg = color!(0x232f1f);
        let diff_text = color!(0x9bd07a);
        let edit_bg = color!(0xc25e1c);
        let edit_text = color!(0xfff8ee);
        let caret_color = color!(0xfff4e0);

        let hex_start_x = content_bounds.x + self.addr_col_width() - scroll_x;
        let ascii_start_x = self.ascii_start_x(content_bounds.x) - scroll_x;
        let sel_range = self.selection.range();
        let cursor_addr = self.selection.cursor;
        let edit_addr = self.edit.map(|e| e.addr);

        // ── Address-gutter background (covers header + content rows) ────
        // Drawn early so both the header row and content rows sit on top.
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: bounds.y,
                    width: self.addr_col_width(),
                    height: HEADER_HEIGHT + viewport_h,
                },
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(color!(0x14110f)),
        );

        // ── Column header ─────────────────────────────────────────────────
        // Background for the header row (right of the address gutter).
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x + self.addr_col_width(),
                    y: bounds.y,
                    width: (bounds.width - self.addr_col_width()).max(0.0),
                    height: HEADER_HEIGHT,
                },
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(header_bg),
        );

        // Hex column numbers (e.g. "0 1 2 3 4 5 6 7  8 9 A B C D E F").
        for col in 0..self.bytes_per_row as usize {
            let group = col / 8;
            let cell_x =
                hex_start_x + col as f32 * HEX_CELL_WIDTH + group as f32 * GROUP_GAP;
            let label = if self.show_decimal {
                format!("{}", col)
            } else {
                // One hex digit per column (0–F).
                let c = match col {
                    0..=9 => (b'0' + col as u8) as char,
                    10..=15 => (b'A' + col as u8 - 10) as char,
                    _ => '?',
                };
                c.to_string()
            };
            // Center the label in its HEX_CELL_WIDTH column.
            let label_w = label.len() as f32 * 9.0;
            let text_x = cell_x + (HEX_CELL_WIDTH - label_w) / 2.0;
            draw_glyph_string(
                renderer,
                &self.cache,
                &label,
                font,
                Rectangle {
                    x: text_x,
                    y: bounds.y,
                    width: label_w,
                    height: HEADER_HEIGHT,
                },
                header_color,
                full_clip,
            );
        }

        // Thin separator line below the header.
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: bounds.y + HEADER_HEIGHT - 1.0,
                    width: bounds.width,
                    height: 1.0,
                },
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(header_separator),
        );

        // ── Data rows ─────────────────────────────────────────────────────
        for row_idx in visible {
            let base_addr = row_idx * bpr as u64;
            let y = content_bounds.y + (row_idx as f32 * ROW_HEIGHT) - scroll;

            // ── Entropy colour band in the address gutter ────────────────
            if let Some(bands) = self.entropy_bands {
                if let Some(&(_, entropy)) = bands.get(row_idx as usize) {
                    let (r, g, b) = entropy_to_color(entropy);
                    let band_color = Color::from_rgb(r, g, b);
                    let band_rect = Rectangle {
                        x: bounds.x,
                        y,
                        width: 4.0,
                        height: ROW_HEIGHT,
                    };
                    if let Some(clipped) = content_clip.intersection(&band_rect) {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: clipped,
                                border: Border::default(),
                                shadow: Shadow::default(),
                                snap: true,
                            },
                            Background::Color(band_color),
                        );
                    }
                }
            }

            // Address gutter — right-aligned.
            let addr_str = if self.show_decimal {
                format!("{}", base_addr)
            } else {
                format!("{:08X}", base_addr)
            };
            let text_w = addr_str.len() as f32 * 9.0;
            draw_glyph_string(
                renderer,
                &self.cache,
                &addr_str,
                font,
                Rectangle {
                    x: bounds.x + self.addr_col_width() - 8.0 - text_w,
                    y,
                    width: text_w,
                    height: ROW_HEIGHT,
                },
                addr_color,
                content_clip,
            );

            // Hex + ASCII columns.
            let row_end = (base_addr as usize + bpr).min(self.bytes.len());
            let row_bytes = &self.bytes[base_addr as usize..row_end];

            for (col, &b) in row_bytes.iter().enumerate() {
                let addr = base_addr + col as u64;
                let group = col / 8;
                let cell_x = hex_start_x + col as f32 * HEX_CELL_WIDTH + group as f32 * GROUP_GAP;
                let ax = ascii_start_x + col as f32 * ASCII_CELL_WIDTH;

                let in_sel = sel_range.contains(&addr);
                let is_dirty = self.dirty.contains(&addr);
                let is_diff = self.vanilla_diff.contains(&addr);
                let pat_entry = self.patterns.get(&addr).copied();
                let is_editing = edit_addr == Some(addr);

                // Background priority: edit > selection-cursor > selection >
                // pattern > dirty (this session) > diff (cumulative vs vanilla).
                let base_bg = if is_editing {
                    Some(edit_bg)
                } else if in_sel {
                    Some(if addr == cursor_addr {
                        cursor_bg
                    } else {
                        selection_bg
                    })
                } else if let Some((pid, color_idx)) = pat_entry {
                    let mut bg = pattern_bg(color_idx);
                    if self.alternate_patterns.contains(&pid) {
                        bg.r *= 0.5;
                        bg.g *= 0.5;
                        bg.b *= 0.5;
                    }
                    Some(bg)
                } else if is_dirty {
                    Some(dirty_bg)
                } else if is_diff {
                    Some(diff_bg)
                } else {
                    None
                };

                // Default foreground via the shared provider chain.
                let (default_fg, _) = default_byte_colors(self.color_scheme, b, self.dim_nulls);
                let default_fg = default_fg.unwrap_or(hex_color);

                let text_color = if is_editing {
                    edit_text
                } else if in_sel {
                    selection_text
                } else if let Some((_, color_idx)) = pat_entry {
                    pattern_fg(color_idx)
                } else if is_dirty {
                    dirty_text
                } else if is_diff {
                    diff_text
                } else {
                    default_fg
                };
                let ascii_col = if is_editing {
                    edit_text
                } else if in_sel {
                    selection_text
                } else if let Some((_, color_idx)) = pat_entry {
                    pattern_fg(color_idx)
                } else if is_dirty {
                    dirty_text
                } else if is_diff {
                    diff_text
                } else if self.color_scheme != ColorScheme::Monochrome {
                    default_fg
                } else {
                    ascii_color
                };

                // Search-match overlay (overrides bg/fg when applicable).
                let in_search = self.search_match_set.contains(&addr);
                let in_current_match = self
                    .search_current_addr
                    .map(|cur| addr >= cur && addr < cur + self.search_query_len)
                    .unwrap_or(false);
                let bg = if in_current_match {
                    Some(color!(0x4a6a2a))
                } else if in_search {
                    Some(color!(0x2a4a2a))
                } else {
                    base_bg
                };
                let text_color = if in_current_match {
                    color!(0xfff8ee)
                } else if in_search {
                    color!(0xfff4e0)
                } else {
                    text_color
                };

                if let Some(c) = bg {
                    fill_cell(renderer, cell_x, y, HEX_CELL_WIDTH, c, cell_clip);
                    fill_cell(renderer, ax, y, ASCII_CELL_WIDTH, c, cell_clip);
                }

                if is_editing {
                    // Render the in-flight draft instead of the underlying
                    // byte. Empty draft → show a thin caret block where the
                    // first nibble would land.
                    let draft = self.edit.map(|e| e.draft).unwrap_or("");
                    let chars: Vec<char> = draft.chars().collect();
                    let hi = chars
                        .first()
                        .map(|c| char_to_glyph(*c))
                        .unwrap_or(HEX_DIGITS[(b >> 4) as usize]);
                    let lo = chars
                        .get(1)
                        .map(|c| char_to_glyph(*c))
                        .unwrap_or(HEX_DIGITS[(b & 0x0F) as usize]);
                    let hi_p = shape_glyph(&self.cache, hi, font);
                    let lo_p = shape_glyph(&self.cache, lo, font);
                    paint_glyph(renderer, &hi_p, cell_x, y, text_color, cell_clip);
                    paint_glyph(renderer, &lo_p, cell_x + 8.0, y, text_color, cell_clip);

                    // Caret over the next nibble slot.
                    let caret_off = match chars.len() {
                        0 => 0.0,
                        1 => 8.0,
                        _ => 16.0,
                    };
                    fill_cell(
                        renderer,
                        cell_x + caret_off,
                        y + ROW_HEIGHT - 2.0,
                        7.0,
                        caret_color,
                        cell_clip,
                    );

                    // ASCII column shows the would-be byte.
                    let ascii_glyph = match chars.len() {
                        2 => {
                            let v = u8::from_str_radix(draft, 16).unwrap_or(b);
                            ascii_repr(v)
                        }
                        _ => "·",
                    };
                    let ascii = shape_glyph(&self.cache, ascii_glyph, font);
                    paint_glyph(renderer, &ascii, ax, y, ascii_col, cell_clip);
                } else {
                    let hi = shape_glyph(&self.cache, HEX_DIGITS[(b >> 4) as usize], font);
                    let lo = shape_glyph(&self.cache, HEX_DIGITS[(b & 0x0F) as usize], font);
                    paint_glyph(renderer, &hi, cell_x, y, text_color, cell_clip);
                    paint_glyph(renderer, &lo, cell_x + 8.0, y, text_color, cell_clip);

                    let ascii = shape_glyph(&self.cache, ascii_repr(b), font);
                    paint_glyph(renderer, &ascii, ax, y, ascii_col, cell_clip);
                }
            }

            // ── Annotation column (per-segment colour) ──────────────────
            if let Some(segments) = self.row_annotations.get(&base_addr) {
                let ann_x0 = self.annotation_start_x(bounds.x) - scroll_x;
                let mut seg_x = ann_x0;
                // Shared separator paragraph (shaped once).
                let sep_para = shape_glyph(&self.cache, " │ ", font);
                let sep_w = sep_para.min_bounds().width;
                for (i, (pat_id, text)) in segments.iter().enumerate() {
                    let is_active = self.active_patterns.contains(pat_id);
                    let color = if is_active {
                        color!(0xd4cabd)
                    } else {
                        color!(0x6a6050)
                    };
                    if i > 0 {
                        let cell = Rectangle {
                            x: seg_x,
                            y,
                            width: sep_w,
                            height: ROW_HEIGHT,
                        };
                        let pos = cell.anchor(
                            sep_para.min_bounds(),
                            alignment::Horizontal::Left,
                            alignment::Vertical::Center,
                        );
                        if let Some(cell_clip) = content_clip.intersection(&cell) {
                            <iced::Renderer as text::Renderer>::fill_paragraph(
                                renderer,
                                &sep_para,
                                pos,
                                color!(0x6a6050),
                                cell_clip,
                            );
                        }
                        seg_x += sep_w;
                    }
                    let para = shape_glyph(&self.cache, text, font);
                    let text_w = para.min_bounds().width;
                    let cell = Rectangle {
                        x: seg_x,
                        y,
                        width: text_w,
                        height: ROW_HEIGHT,
                    };
                    let pos = cell.anchor(
                        para.min_bounds(),
                        alignment::Horizontal::Left,
                        alignment::Vertical::Center,
                    );
                    if let Some(cell_clip) = content_clip.intersection(&cell) {
                        <iced::Renderer as text::Renderer>::fill_paragraph(
                            renderer, &para, pos, color, cell_clip,
                        );
                    }
                    seg_x += text_w;
                }
            }
        }

        // Subtle vertical separator between every 8-byte group.
        for g in 1..group_count(bpr) {
            let x =
                hex_start_x + (g * 8) as f32 * HEX_CELL_WIDTH + (g - 1) as f32 * GROUP_GAP + 4.0;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x,
                        y: content_bounds.y,
                        width: 1.0,
                        height: viewport_h,
                    },
                    border: Border::default(),
                    shadow: Shadow::default(),
                    snap: true,
                },
                Background::Color(group_separator_color),
            );
        }

        // Minimap overview strip (between content and scrollbar).
        let total_len = self.bytes.len() as u64;
        let needs_vscroll = total_h > viewport_h;
        if needs_vscroll && self.show_minimap {
            let hovering = cursor
                .position_over(content_bounds)
                .map(|p| minimap::minimap_rect(content_bounds, viewport_h, MINIMAP_WIDTH, SCROLLBAR_THICKNESS).contains(p))
                .unwrap_or(false);

            // Compute or reuse the minimap pixel cache.
            let h_px = viewport_h.max(1.0) as u32;
            let mut cache = state.minimap_cache.borrow_mut();
            let needs_recompute = match &*cache {
                Some(c) => !minimap::minimap_cache_valid(
                    c,
                    total_len,
                    h_px,
                    self.color_scheme,
                    self.dim_nulls,
                    self.patterns,
                    self.dirty,
                    self.vanilla_diff,
                ),
                None => true,
            };
            if needs_recompute {
                let cols = minimap::compute_block_pixels(
                    self.bytes,
                    total_len,
                    h_px,
                    self.patterns,
                    &self.alternate_patterns,
                    self.dirty,
                    self.vanilla_diff,
                    self.color_scheme,
                    self.dim_nulls,
                );
                *cache = Some(minimap::MinimapCache {
                    columns: cols,
                    total_len,
                    h_px,
                    color_scheme: self.color_scheme,
                    dim_nulls: self.dim_nulls,
                    pattern_hash: minimap::pattern_hash(self.patterns),
                    dirty_count: self.dirty.len(),
                    diff_count: self.vanilla_diff.len(),
                });
            }
            // Safety: cache is always Some here — either it was already
            // valid or we just recomputed it above.
            let columns = &cache.as_ref().unwrap().columns;

            minimap::draw_minimap(
                renderer,
                content_bounds,
                scroll,
                total_h,
                viewport_h,
                columns,
                self.selection.start(),
                self.selection.end(),
                self.selection.cursor,
                total_len,
                state.dragging_minimap || hovering,
            );
        }

        // Scrollbar with search-match and cursor-position markers.
        if needs_vscroll {
            let hovering = cursor
                .position_over(content_bounds)
                .map(|p| scrollbar_track(content_bounds, viewport_h).contains(p))
                .unwrap_or(false);
            draw_vscrollbar(
                renderer,
                content_bounds,
                scroll,
                total_h,
                viewport_h,
                state.dragging_scrollbar || hovering,
                self.search_match_starts,
                self.selection.cursor,
                total_len,
            );
        }

        // Horizontal scrollbar at the bottom.
        let content_w = self.total_content_width();
        let avail_w = bounds.width - SCROLLBAR_THICKNESS;
        let needs_hscroll = content_w > avail_w;
        if needs_hscroll {
            let htrack = hscrollbar_track(bounds);
            let hovering = cursor
                .position_over(bounds)
                .map(|p| htrack.contains(p))
                .unwrap_or(false);
            draw_hscrollbar(
                renderer,
                htrack,
                scroll_x,
                content_w,
                avail_w,
                state.dragging_scrollbar_x || hovering,
            );
        }
    }
}

/// Lift a hex character to its rendered glyph. Falls back to a blank for
/// non-hex input (which the message handler also rejects).
fn char_to_glyph(c: char) -> &'static str {
    match c.to_ascii_uppercase() {
        '0' => "0",
        '1' => "1",
        '2' => "2",
        '3' => "3",
        '4' => "4",
        '5' => "5",
        '6' => "6",
        '7' => "7",
        '8' => "8",
        '9' => "9",
        'A' => "A",
        'B' => "B",
        'C' => "C",
        'D' => "D",
        'E' => "E",
        'F' => "F",
        _ => " ",
    }
}

/// First hex character in a typed `text` field, if any. Used so paste of
/// "FF aa" only registers the first digit per keypress.
pub fn first_hex_char(t: &str) -> Option<char> {
    t.chars().find(|c| c.is_ascii_hexdigit())
}

/// First printable (non-control) character in a typed `text` field, if any.
/// Used by text write-modes so that any meaningful character is forwarded to
/// the encoding pipeline.  Control characters like Tab/Enter are excluded.
pub fn first_printable_char(t: &str) -> Option<char> {
    // Allow any character that is not a control character.
    // This includes spaces (U+0020) — they are NOT control characters but
    // `char::is_whitespace` returns true for ' '.  Spaces MUST be allowed so
    // the user can type them in text write modes.
    t.chars().find(|c| !c.is_control())
}

impl<'a, Message> HexMatrix<'a, Message> {
    fn publish_nav(
        &self,
        state: &mut State,
        dir: NavDir,
        extend: bool,
        bounds: Rectangle,
        shell: &mut Shell<'_, Message>,
    ) {
        if self.bytes.is_empty() {
            return;
        }
        if let Some(cb) = &self.on_nav {
            shell.publish(cb(dir, extend));
        }
        // Optimistically mirror nav_target so we can scroll-into-view this
        // frame instead of waiting for the message handler.
        let bpr = self.bytes_per_row as u64;
        let max_addr = (self.bytes.len() as u64).saturating_sub(1);
        let viewport_h = self.content_viewport_h(bounds.height, bounds.width);
        let target = crate::selection::nav_target(
            self.selection.cursor,
            dir,
            bpr,
            page_rows(viewport_h),
            max_addr,
        );
        let new_scroll = ensure_visible(
            state.scroll_offset.get(),
            target,
            bpr,
            viewport_h,
            self.total_height(),
        );
        if (new_scroll - state.scroll_offset.get()).abs() > f32::EPSILON {
            state.scroll_offset.set(new_scroll);
        }
        shell.request_redraw();
    }
}

fn group_count(bpr: usize) -> usize {
    bpr.div_ceil(8).saturating_sub(1)
}

fn fill_cell(
    renderer: &mut iced::Renderer,
    x: f32,
    y: f32,
    width: f32,
    color: Color,
    clip: Rectangle,
) {
    let cell = Rectangle {
        x,
        y,
        width,
        height: ROW_HEIGHT,
    };
    let Some(rect) = clip.intersection(&cell) else {
        return;
    };
    renderer.fill_quad(
        renderer::Quad {
            bounds: rect,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(color),
    );
}

fn paint_glyph(
    renderer: &mut iced::Renderer,
    paragraph: &Paragraph,
    x: f32,
    y: f32,
    color: Color,
    clip: Rectangle,
) {
    let cell = Rectangle {
        x,
        y,
        width: 16.0,
        height: ROW_HEIGHT,
    };
    let pos = cell.anchor(
        paragraph.min_bounds(),
        alignment::Horizontal::Left,
        alignment::Vertical::Center,
    );
    let cell_clip = clip.intersection(&cell).unwrap_or(Rectangle {
        x,
        y,
        width: 0.0,
        height: 0.0,
    });
    <iced::Renderer as text::Renderer>::fill_paragraph(renderer, paragraph, pos, color, cell_clip);
}

fn draw_glyph_string(
    renderer: &mut iced::Renderer,
    cache: &ParagraphCache,
    text_str: &str,
    font: Font,
    bounds: Rectangle,
    color: Color,
    clip: Rectangle,
) {
    let key = ParagraphKey::new(text_str, TEXT_SIZE, bounds.width, font);
    let para = cache.get_or_insert(key, || {
        Paragraph::with_text(text::Text {
            content: text_str,
            bounds: Size::new(bounds.width, bounds.height),
            size: Pixels(TEXT_SIZE),
            line_height: text::LineHeight::default(),
            font,
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Top,
            shaping: text::Shaping::Basic,
            wrapping: text::Wrapping::None,
        })
    });
    let pos = bounds.anchor(
        para.min_bounds(),
        alignment::Horizontal::Left,
        alignment::Vertical::Center,
    );
    let Some(cell_clip) = clip.intersection(&bounds) else {
        return;
    };
    <iced::Renderer as text::Renderer>::fill_paragraph(renderer, &para, pos, color, cell_clip);
}

fn scrollbar_track(bounds: Rectangle, viewport_h: f32) -> Rectangle {
    Rectangle {
        x: bounds.x + bounds.width - SCROLLBAR_THICKNESS,
        y: bounds.y,
        width: SCROLLBAR_THICKNESS,
        height: viewport_h,
    }
}

fn hscrollbar_track(bounds: Rectangle) -> Rectangle {
    Rectangle {
        x: bounds.x,
        y: bounds.y + bounds.height - SCROLLBAR_THICKNESS,
        width: bounds.width - SCROLLBAR_THICKNESS,
        height: SCROLLBAR_THICKNESS,
    }
}

fn thumb_height(track: Rectangle, total_h: f32) -> f32 {
    (track.height / total_h * track.height).max(20.0)
}

/// Y position of a file address on the scrollbar track, as a fraction 0..1.
fn scrollbar_y_frac(addr: u64, total_len: u64, track: Rectangle) -> f32 {
    if total_len <= 1 {
        return track.y;
    }
    track.y + (addr as f32 / (total_len - 1) as f32) * track.height
}

fn scrollbar_thumb(track: Rectangle, scroll: f32, total_h: f32) -> Rectangle {
    let h = thumb_height(track, total_h);
    let max_off = (total_h - track.height).max(1.0);
    let y = track.y + (scroll / max_off) * (track.height - h);
    Rectangle {
        x: track.x + 1.0,
        y,
        width: track.width - 2.0,
        height: h,
    }
}

fn hthumb_len(track: Rectangle, content_w: f32, _avail_w: f32) -> f32 {
    (track.width / content_w * track.width).max(20.0)
}

fn hscrollbar_thumb(track: Rectangle, scroll_x: f32, content_w: f32, avail_w: f32) -> Rectangle {
    let w = hthumb_len(track, content_w, avail_w);
    let max_off = (content_w - avail_w).max(1.0);
    let x = track.x + (scroll_x / max_off) * (track.width - w);
    Rectangle {
        x,
        y: track.y + 1.0,
        width: w,
        height: track.height - 2.0,
    }
}

/// Marker dot size in pixels.
const MARKER_SIZE: f32 = 4.0;

/// Draw the vertical scrollbar (no fattening — simple color change on hover).
#[allow(clippy::too_many_arguments)]
fn draw_vscrollbar(
    renderer: &mut iced::Renderer,
    bounds: Rectangle,
    scroll: f32,
    total_h: f32,
    viewport_h: f32,
    active: bool,
    search_match_starts: &[u64],
    cursor_addr: u64,
    total_len: u64,
) {
    let track = scrollbar_track(bounds, viewport_h);
    let thumb = scrollbar_thumb(track, scroll, total_h);

    // Track background.
    renderer.fill_quad(
        renderer::Quad {
            bounds: track,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(color!(0x141210)),
    );

    let thumb_color = if active {
        color!(0xB97024)
    } else {
        color!(0x5d4037)
    };

    // Search-match markers (small green dots).
    for &match_start in search_match_starts {
        let my = scrollbar_y_frac(match_start, total_len, track);
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: track.x + (track.width - MARKER_SIZE) / 2.0,
                    y: my - MARKER_SIZE / 2.0,
                    width: MARKER_SIZE,
                    height: MARKER_SIZE,
                },
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(color!(0x4a7a2a)),
        );
    }

    // Cursor-position marker (amber dot).
    let cy = scrollbar_y_frac(cursor_addr, total_len, track);
    renderer.fill_quad(
        renderer::Quad {
            bounds: Rectangle {
                x: track.x + (track.width - MARKER_SIZE) / 2.0,
                y: cy - MARKER_SIZE / 2.0,
                width: MARKER_SIZE,
                height: MARKER_SIZE,
            },
            border: Border::default(),
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(color!(0xB97024)),
    );

    // Thumb.
    renderer.fill_quad(
        renderer::Quad {
            bounds: thumb,
            border: Border {
                color: thumb_color,
                width: 0.5,
                radius: 0.into(),
            },
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(thumb_color),
    );
}

/// Draw the horizontal scrollbar.
fn draw_hscrollbar(
    renderer: &mut iced::Renderer,
    track: Rectangle,
    scroll_x: f32,
    content_w: f32,
    avail_w: f32,
    active: bool,
) {
    let thumb = hscrollbar_thumb(track, scroll_x, content_w, avail_w);

    renderer.fill_quad(
        renderer::Quad {
            bounds: track,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(color!(0x141210)),
    );

    let thumb_color = if active {
        color!(0xB97024)
    } else {
        color!(0x5d4037)
    };

    renderer.fill_quad(
        renderer::Quad {
            bounds: thumb,
            border: Border {
                color: thumb_color,
                width: 0.5,
                radius: 0.into(),
            },
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(thumb_color),
    );
}

impl<'a, Message, Theme> From<HexMatrix<'a, Message>>
    for Element<'a, Message, Theme, iced::Renderer>
where
    Theme: 'a,
    Message: 'a,
{
    fn from(w: HexMatrix<'a, Message>) -> Self {
        Element::new(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ADDR_COL_WIDTH: f32 = 88.0;

    fn make_bounds() -> Rectangle {
        Rectangle {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 320.0,
        }
    }

    #[test]
    fn empty_provider_yields_empty_range() {
        assert_eq!(visible_row_range(0.0, 200.0, 16.0, 0, 2), 0..0);
    }

    #[test]
    fn visible_range_with_zero_scroll() {
        let r = visible_row_range(0.0, 200.0, 16.0, 100, 2);
        assert_eq!(r.start, 0);
        assert_eq!(r.end, 18);
    }

    #[test]
    fn visible_range_scrolled_into_middle() {
        let r = visible_row_range(320.0, 200.0, 16.0, 100, 2);
        assert_eq!(r.start, 18);
        assert_eq!(r.end, 36);
    }

    #[test]
    fn visible_range_clamps_at_total() {
        let r = visible_row_range(2_000.0, 200.0, 16.0, 100, 2);
        assert!(r.end <= 100);
        assert!(r.start <= r.end);
    }

    #[test]
    fn visible_range_negative_scroll_clamped_to_zero() {
        let r = visible_row_range(-50.0, 200.0, 16.0, 100, 2);
        assert_eq!(r.start, 0);
    }

    #[test]
    fn clamp_scroll_keeps_within_bounds() {
        assert_eq!(clamp_scroll(-10.0, 1000.0, 200.0), 0.0);
        assert_eq!(clamp_scroll(2_000.0, 1000.0, 200.0), 800.0);
        assert_eq!(clamp_scroll(500.0, 1000.0, 200.0), 500.0);
    }

    #[test]
    fn clamp_scroll_when_content_smaller_than_viewport() {
        assert_eq!(clamp_scroll(500.0, 100.0, 1000.0), 0.0);
    }

    #[test]
    fn ascii_repr_handles_printable_and_non_printable() {
        assert_eq!(ascii_repr(b'A'), "A");
        assert_eq!(ascii_repr(b' '), " ");
        assert_eq!(ascii_repr(0x00), "·");
        assert_eq!(ascii_repr(0xFF), "·");
        assert_eq!(ascii_repr(0x7F), "·");
    }

    #[test]
    fn group_count_handles_typical_widths() {
        assert_eq!(group_count(8), 0);
        assert_eq!(group_count(16), 1);
        assert_eq!(group_count(32), 3);
    }

    #[test]
    fn ensure_visible_no_op_when_already_visible() {
        let scroll = ensure_visible(0.0, 5 * 16, 16, 320.0, 1000.0);
        assert_eq!(scroll, 0.0);
    }

    #[test]
    fn ensure_visible_scrolls_down_when_target_below() {
        let scroll = ensure_visible(0.0, 100 * 16, 16, 320.0, 100_000.0);
        assert_eq!(scroll, 1448.0);
    }

    #[test]
    fn ensure_visible_scrolls_up_when_target_above() {
        let scroll = ensure_visible(1000.0, 5 * 16, 16, 320.0, 100_000.0);
        assert_eq!(scroll, 0.0);
    }

    #[test]
    fn page_rows_at_least_one() {
        assert_eq!(page_rows(0.0), 1);
        assert_eq!(page_rows(160.0), 10);
    }

    #[test]
    fn addr_at_hex_column_first_byte() {
        let bounds = make_bounds();
        let p = Point::new(89.0, 4.0);
        let addr = addr_at(p, bounds, 0.0, 0.0, 16, 1024, TEST_ADDR_COL_WIDTH).unwrap();
        assert_eq!(addr, 0);
    }

    #[test]
    fn addr_at_hex_column_with_scroll() {
        let bounds = make_bounds();
        let p = Point::new(89.0, 4.0);
        let addr = addr_at(p, bounds, 32.0, 0.0, 16, 1024, TEST_ADDR_COL_WIDTH).unwrap();
        assert_eq!(addr, 32);
    }

    #[test]
    fn addr_at_ascii_column() {
        let bounds = make_bounds();
        let ascii_start = TEST_ADDR_COL_WIDTH
            + 16.0 * HEX_CELL_WIDTH
            + group_count(16) as f32 * GROUP_GAP
            + COLUMN_GAP;
        let p = Point::new(ascii_start + 2.0 * ASCII_CELL_WIDTH + 1.0, 4.0);
        let addr = addr_at(p, bounds, 0.0, 0.0, 16, 1024, TEST_ADDR_COL_WIDTH).unwrap();
        assert_eq!(addr, 2);
    }

    #[test]
    fn addr_at_outside_columns_returns_none() {
        let bounds = make_bounds();
        assert!(addr_at(
            Point::new(20.0, 4.0),
            bounds,
            0.0,
            0.0,
            16,
            1024,
            TEST_ADDR_COL_WIDTH
        )
        .is_none());
    }

    #[test]
    fn addr_at_clamps_past_end_of_file() {
        let bounds = make_bounds();
        let p = Point::new(TEST_ADDR_COL_WIDTH + 15.0 * HEX_CELL_WIDTH + 5.0, 4.0);
        let addr = addr_at(p, bounds, 0.0, 0.0, 16, 5, TEST_ADDR_COL_WIDTH).unwrap();
        assert_eq!(addr, 4);
    }

    #[test]
    fn addr_at_empty_file_returns_none() {
        let bounds = make_bounds();
        assert!(addr_at(
            Point::new(100.0, 4.0),
            bounds,
            0.0,
            0.0,
            16,
            0,
            TEST_ADDR_COL_WIDTH
        )
        .is_none());
    }

    #[test]
    fn first_hex_char_picks_first_match() {
        assert_eq!(first_hex_char("a"), Some('a'));
        assert_eq!(first_hex_char("F"), Some('F'));
        assert_eq!(first_hex_char(" 9"), Some('9'));
        assert_eq!(first_hex_char("xyz"), None);
        assert_eq!(first_hex_char(""), None);
    }

    #[test]
    fn char_to_glyph_normalizes_case() {
        assert_eq!(char_to_glyph('a'), "A");
        assert_eq!(char_to_glyph('F'), "F");
        assert_eq!(char_to_glyph('0'), "0");
        assert_eq!(char_to_glyph('z'), " ");
    }

    #[test]
    fn first_printable_char_allows_space() {
        // Regression: space is whitespace (not control) and MUST be allowed.
        assert_eq!(first_printable_char(" "), Some(' '));
        assert_eq!(first_printable_char("  "), Some(' '));
    }

    #[test]
    fn first_printable_char_standard() {
        assert_eq!(first_printable_char("Hello"), Some('H'));
        assert_eq!(first_printable_char("ł"), Some('ł'));
        assert_eq!(first_printable_char("€"), Some('€'));
    }

    #[test]
    fn first_printable_char_rejects_control() {
        // Tab, newline, null etc. are control characters — correctly rejected.
        assert_eq!(first_printable_char("\t"), None);
        assert_eq!(first_printable_char("\n"), None);
        assert_eq!(first_printable_char("\0"), None);
    }

    #[test]
    fn first_printable_char_empty_string() {
        assert_eq!(first_printable_char(""), None);
    }

    // ── Column header tests ─────────────────────────────────────────────

    #[test]
    fn header_constants_are_reasonable() {
        assert_eq!(HEADER_HEIGHT, 16.0);
        assert_eq!(HEADER_HEIGHT, ROW_HEIGHT);
    }

    #[test]
    fn addr_at_returns_none_for_clicks_in_header_area() {
        // With bounds adjusted to start after the header, a click at the top
        // of the widget (in the header area) should not match any address.
        let bounds = Rectangle {
            x: 0.0,
            y: HEADER_HEIGHT, // content starts after the header
            width: 800.0,
            height: 300.0,
        };
        // A click at (100, 0) in widget coordinates — this is inside the
        // header area (y < HEADER_HEIGHT), but *outside* the content bounds
        // (content starts at y=HEADER_HEIGHT). addr_at should reject it.
        let p = Point::new(100.0, 0.0);
        assert!(
            addr_at(p, bounds, 0.0, 0.0, 16, 1024, TEST_ADDR_COL_WIDTH).is_none(),
            "click in header area should not resolve to a byte address"
        );
    }

    #[test]
    fn header_does_not_affect_addr_at_after_content_bounds() {
        // Verify that when the content bounds start after HEADER_HEIGHT,
        // addr_at requires the click Y to be within those bounds to resolve
        // an address. A click in the header area (y < HEADER_HEIGHT) is
        // rejected because bounds.contains() returns false.
        let bpr: u8 = 16;
        let aw = bpr as f32 * HEX_CELL_WIDTH + group_count(bpr as usize) as f32 * GROUP_GAP
            + TEST_ADDR_COL_WIDTH
            + COLUMN_GAP
            + bpr as f32 * ASCII_CELL_WIDTH;
        let bounds = Rectangle {
            x: 0.0,
            y: HEADER_HEIGHT, // content starts here
            width: aw + 100.0,
            height: 300.0,
        };
        // Click at (in hex area, y=0) — this is BEFORE content_bounds.y so
        // bounds.contains() is false → addr_at returns None.
        let hex_x = TEST_ADDR_COL_WIDTH + 4.0;
        assert!(
            addr_at(Point::new(hex_x, 0.0), bounds, 0.0, 0.0, bpr, 1024, TEST_ADDR_COL_WIDTH)
                .is_none(),
            "click at y=0 (in header) should be rejected by content bounds"
        );
        // Click at the same x but within the content area → should resolve.
        assert!(
            addr_at(Point::new(hex_x, HEADER_HEIGHT + 4.0), bounds, 0.0, 0.0, bpr, 1024, TEST_ADDR_COL_WIDTH)
                .is_some(),
            "click within content bounds should resolve"
        );
    }
}
