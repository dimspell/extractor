//! Custom Iced widget rendering the virtualized hex matrix.
//!
//! Layout (left → right): address gutter, hex bytes (grouped 8 with a small
//! gap), ASCII gutter, scrollbar. Only rows in the viewport are touched per
//! frame; everything else is virtual.

use std::cell::Cell;
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

use crate::editors::hex_editor::pattern::{pattern_bg, pattern_fg};
use crate::editors::hex_editor::selection::{NavDir, Selection};
use crate::view::editor::paragraph_cache::{ParagraphCache, ParagraphKey};

type Paragraph = GraphicsParagraph;

/// Default cell metrics. Tuned for 11px monospace.
const TEXT_SIZE: f32 = 11.0;
const ROW_HEIGHT: f32 = 16.0;
const HEX_CELL_WIDTH: f32 = 20.0;
const ASCII_CELL_WIDTH: f32 = 9.0;
const GROUP_GAP: f32 = 8.0;
const COLUMN_GAP: f32 = 12.0;
const ADDR_COL_WIDTH: f32 = 88.0;
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
    pub dragging_scrollbar: bool,
    pub drag_start_cursor_y: f32,
    pub drag_start_offset: f32,
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
    /// Fast lookup: byte address → pattern id.
    patterns: &'a BTreeMap<u64, usize>,
    /// Search results: all byte addresses covered by any match.
    search_match_set: &'a BTreeSet<u64>,
    /// Length (in bytes) of the current search query.
    search_query_len: u64,
    /// Start address of the current (navigated-to) match, if any.
    search_current_addr: Option<u64>,
    /// Start addresses of all search matches, for scrollbar markers.
    search_match_starts: &'a [u64],
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
    on_edit_commit: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    on_right_click: Option<Box<dyn Fn(u64) -> Message + 'a>>,
    on_create_pattern: Option<Box<dyn Fn() -> Message + 'a>>,
    on_open_goto: Option<Box<dyn Fn() -> Message + 'a>>,
    on_open_search: Option<Box<dyn Fn() -> Message + 'a>>,
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
        patterns: &'a BTreeMap<u64, usize>,
        search_match_set: &'a BTreeSet<u64>,
        search_query_len: u64,
        search_current_addr: Option<u64>,
        search_match_starts: &'a [u64],
        cache: ParagraphCache,
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
            cache,
            width: Length::Fill,
            height: Length::Fill,
            on_select_at: None,
            on_extend_to: None,
            on_nav: None,
            on_begin_edit: None,
            on_edit_type: None,
            on_edit_backspace: None,
            on_edit_cancel: None,
            on_edit_commit: None,
            on_right_click: None,
            on_create_pattern: None,
            on_open_goto: None,
            on_open_search: None,
        }
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
            + ADDR_COL_WIDTH
            + (bpr as f32) * HEX_CELL_WIDTH
            + group_count(bpr) as f32 * GROUP_GAP
            + COLUMN_GAP
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
    bytes_per_row: u8,
    total_len: u64,
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

    let hex_start = bounds.x + ADDR_COL_WIDTH;
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

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if !cursor.is_over(bounds) {
                    return;
                }
                let dy = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => -y * ROW_HEIGHT * 3.0,
                    mouse::ScrollDelta::Pixels { y, .. } => -y,
                };
                let so = state.scroll_offset.get();
                let new = clamp_scroll(so + dy, total_h, bounds.height);
                if (new - so).abs() > f32::EPSILON {
                    state.scroll_offset.set(new);
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(p) = cursor.position_over(bounds) else {
                    return;
                };
                // Scrollbar takes precedence.
                let scrollbar = scrollbar_track(bounds);
                if scrollbar.contains(p) && total_h > bounds.height {
                    let thumb = scrollbar_thumb(scrollbar, state.scroll_offset.get(), total_h);
                    if thumb.contains(p) {
                        state.dragging_scrollbar = true;
                        state.drag_start_cursor_y = p.y;
                        state.drag_start_offset = state.scroll_offset.get();
                    } else {
                        let dir = if p.y < thumb.y { -1.0 } else { 1.0 };
                        let new = clamp_scroll(
                            state.scroll_offset.get() + dir * bounds.height,
                            total_h,
                            bounds.height,
                        );
                        state.scroll_offset.set(new);
                        shell.request_redraw();
                    }
                    shell.capture_event();
                    return;
                }

                // Cell click → selection (and maybe edit on double-click).
                if let Some(addr) = addr_at(
                    p,
                    bounds,
                    state.scroll_offset.get(),
                    self.bytes_per_row,
                    total_len,
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
                if let Some(addr) = addr_at(
                    p,
                    bounds,
                    state.scroll_offset.get(),
                    self.bytes_per_row,
                    total_len,
                ) {
                    if let Some(cb) = &self.on_right_click {
                        shell.publish(cb(addr));
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.dragging_scrollbar {
                    let Some(p) = cursor.position() else { return };
                    let scrollbar = scrollbar_track(bounds);
                    let thumb_h = thumb_height(scrollbar, total_h);
                    let travel = (scrollbar.height - thumb_h).max(1.0);
                    let max_off = (total_h - bounds.height).max(1.0);
                    let dy = p.y - state.drag_start_cursor_y;
                    let new = state.drag_start_offset + dy * (max_off / travel);
                    state
                        .scroll_offset
                        .set(clamp_scroll(new, total_h, bounds.height));
                    shell.request_redraw();
                    shell.capture_event();
                    return;
                }
                if state.selecting {
                    let Some(p) = cursor.position() else { return };
                    if let Some(addr) = addr_at(
                        p,
                        bounds,
                        state.scroll_offset.get(),
                        self.bytes_per_row,
                        total_len,
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
                if state.selecting {
                    state.selecting = false;
                    consumed = true;
                }
                if consumed {
                    shell.capture_event();
                }
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

                // F2 starts an edit at the current cursor.
                if matches!(key, keyboard::Key::Named(key::Named::F2)) && self.edit.is_none() {
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

                // Hex-digit typing: append in edit mode, or auto-start one.
                if !modifiers.control() && !modifiers.command() && !modifiers.alt() {
                    if let Some(t) = text {
                        if let Some(c) = first_hex_char(t) {
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
        let clip = bounds.intersection(viewport).unwrap_or(bounds);

        // Background.
        renderer.fill_quad(
            renderer::Quad {
                bounds: clip,
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(color!(0x14110f)),
        );

        let total_rows = self.total_rows();
        if total_rows == 0 {
            return;
        }

        let bpr = self.bytes_per_row as usize;
        let total_h = self.total_height();
        let bpr64 = bpr as u64;

        let scroll = if total_h <= bounds.height {
            // Content fits — no scrolling needed.
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
                    bounds.height,
                    total_h,
                )
            } else {
                state.scroll_offset.get()
            }
        };
        state.scroll_offset.set(scroll);

        let visible = visible_row_range(scroll, bounds.height, ROW_HEIGHT, total_rows, OVERSCAN);

        let font = Font::MONOSPACE;
        let addr_color = color!(0x7a6f64);
        let hex_color = color!(0xd4cabd);
        let zero_color = color!(0x4a4339);
        let ascii_color = color!(0xb8a898);
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

        let hex_start_x = bounds.x + ADDR_COL_WIDTH;
        let ascii_start_x = self.ascii_start_x(bounds.x);
        let sel_range = self.selection.range();
        let cursor_addr = self.selection.cursor;
        let edit_addr = self.edit.map(|e| e.addr);

        for row_idx in visible {
            let base_addr = row_idx * bpr as u64;
            let y = bounds.y + (row_idx as f32 * ROW_HEIGHT) - scroll;

            // Address gutter.
            let addr_str = format!("{:08X}", base_addr);
            draw_glyph_string(
                renderer,
                &self.cache,
                &addr_str,
                font,
                Rectangle {
                    x: bounds.x + 8.0,
                    y,
                    width: ADDR_COL_WIDTH - 16.0,
                    height: ROW_HEIGHT,
                },
                addr_color,
                clip,
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
                let pattern_id = self.patterns.get(&addr).copied();
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
                } else if let Some(pid) = pattern_id {
                    Some(pattern_bg(pid as u8))
                } else if is_dirty {
                    Some(dirty_bg)
                } else if is_diff {
                    Some(diff_bg)
                } else {
                    None
                };

                let text_color = if is_editing {
                    edit_text
                } else if in_sel {
                    selection_text
                } else if let Some(pid) = pattern_id {
                    pattern_fg(pid as u8)
                } else if is_dirty {
                    dirty_text
                } else if is_diff {
                    diff_text
                } else if b == 0 {
                    zero_color
                } else {
                    hex_color
                };
                let ascii_col = if is_editing {
                    edit_text
                } else if in_sel {
                    selection_text
                } else if let Some(pid) = pattern_id {
                    pattern_fg(pid as u8)
                } else if is_dirty {
                    dirty_text
                } else if is_diff {
                    diff_text
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
                    fill_cell(renderer, cell_x, y, HEX_CELL_WIDTH, c, clip);
                    fill_cell(renderer, ax, y, ASCII_CELL_WIDTH, c, clip);
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
                    paint_glyph(renderer, &hi_p, cell_x, y, text_color, clip);
                    paint_glyph(renderer, &lo_p, cell_x + 8.0, y, text_color, clip);

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
                        clip,
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
                    paint_glyph(renderer, &ascii, ax, y, ascii_col, clip);
                } else {
                    let hi = shape_glyph(&self.cache, HEX_DIGITS[(b >> 4) as usize], font);
                    let lo = shape_glyph(&self.cache, HEX_DIGITS[(b & 0x0F) as usize], font);
                    paint_glyph(renderer, &hi, cell_x, y, text_color, clip);
                    paint_glyph(renderer, &lo, cell_x + 8.0, y, text_color, clip);

                    let ascii = shape_glyph(&self.cache, ascii_repr(b), font);
                    paint_glyph(renderer, &ascii, ax, y, ascii_col, clip);
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
                        y: bounds.y,
                        width: 1.0,
                        height: bounds.height,
                    },
                    border: Border::default(),
                    shadow: Shadow::default(),
                    snap: true,
                },
                Background::Color(group_separator_color),
            );
        }

        // Scrollbar with search-match and cursor-position markers.
        let total_len = self.bytes.len() as u64;
        if total_h > bounds.height {
            let hovering = cursor
                .position_over(bounds)
                .map(|p| scrollbar_track(bounds).contains(p))
                .unwrap_or(false);
            draw_scrollbar(
                renderer,
                bounds,
                scroll,
                total_h,
                state.dragging_scrollbar || hovering,
                self.search_match_starts,
                self.selection.cursor,
                total_len,
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
        // frame instead of waiting for the next message round-trip.
        let bpr = self.bytes_per_row as u64;
        let max_addr = (self.bytes.len() as u64).saturating_sub(1);
        let target = crate::editors::hex_editor::selection::nav_target(
            self.selection.cursor,
            dir,
            bpr,
            page_rows(bounds.height),
            max_addr,
        );
        let new_scroll = ensure_visible(
            state.scroll_offset.get(),
            target,
            bpr,
            bounds.height,
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

fn scrollbar_track(bounds: Rectangle) -> Rectangle {
     Rectangle {
         x: bounds.x + bounds.width - SCROLLBAR_THICKNESS,
         y: bounds.y,
         width: SCROLLBAR_THICKNESS,
         height: bounds.height,
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

 /// Marker dot size in pixels.
 const MARKER_SIZE: f32 = 4.0;

 #[allow(clippy::too_many_arguments)]
 fn draw_scrollbar(
     renderer: &mut iced::Renderer,
     bounds: Rectangle,
     scroll: f32,
     total_h: f32,
     active: bool,
     search_match_starts: &[u64],
     cursor_addr: u64,
     total_len: u64,
 ) {
     let track = scrollbar_track(bounds);

     // Thicken on hover/drag (like table widget).
     let (track, thumb) = if active {
         let fat = SCROLLBAR_THICKNESS + 5.0;
         let fat_track = Rectangle {
             x: bounds.x + bounds.width - fat,
             y: bounds.y,
             width: fat,
             height: bounds.height,
         };
         let thumb = scrollbar_thumb(fat_track, scroll, total_h);
         (fat_track, thumb)
     } else {
         let t = track;
         let thumb = scrollbar_thumb(t, scroll, total_h);
         (t, thumb)
     };

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
        let addr = addr_at(p, bounds, 0.0, 16, 1024).unwrap();
        assert_eq!(addr, 0);
    }

    #[test]
    fn addr_at_hex_column_with_scroll() {
        let bounds = make_bounds();
        let p = Point::new(89.0, 4.0);
        let addr = addr_at(p, bounds, 32.0, 16, 1024).unwrap();
        assert_eq!(addr, 32);
    }

    #[test]
    fn addr_at_ascii_column() {
        let bounds = make_bounds();
        let ascii_start = ADDR_COL_WIDTH
            + 16.0 * HEX_CELL_WIDTH
            + group_count(16) as f32 * GROUP_GAP
            + COLUMN_GAP;
        let p = Point::new(ascii_start + 2.0 * ASCII_CELL_WIDTH + 1.0, 4.0);
        let addr = addr_at(p, bounds, 0.0, 16, 1024).unwrap();
        assert_eq!(addr, 2);
    }

    #[test]
    fn addr_at_outside_columns_returns_none() {
        let bounds = make_bounds();
        assert!(addr_at(Point::new(20.0, 4.0), bounds, 0.0, 16, 1024).is_none());
    }

    #[test]
    fn addr_at_clamps_past_end_of_file() {
        let bounds = make_bounds();
        let p = Point::new(ADDR_COL_WIDTH + 15.0 * HEX_CELL_WIDTH + 5.0, 4.0);
        let addr = addr_at(p, bounds, 0.0, 16, 5).unwrap();
        assert_eq!(addr, 4);
    }

    #[test]
    fn addr_at_empty_file_returns_none() {
        let bounds = make_bounds();
        assert!(addr_at(Point::new(100.0, 4.0), bounds, 0.0, 16, 0).is_none());
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
}
