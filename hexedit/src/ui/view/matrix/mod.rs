//! Custom Iced widget rendering the virtualized hex matrix.
//!
//! Layout (left → right): address gutter, hex bytes (grouped 8 with a small
//! gap), ASCII gutter, scrollbar. Only rows in the viewport are touched per
//! frame; everything else is virtual.
//!
//! The widget implementation is split across submodules:
//!
//! * [`state`] — per-instance widget state (scroll, drag, double-click, etc.)
//! * [`layout`] — pure layout helpers and geometry constants
//! * [`event`]  — mouse and keyboard event handling
//! * [`draw`]   — rendering helpers and the `Widget::draw` body
//!
//! This root module owns the [`HexMatrix`] struct definition, the `Widget`
//! trait implementation (with thin delegation to submodules), and unit tests.

mod state;
mod layout;
mod draw;
mod event;

pub use state::{EditView, State};
pub use draw::{first_hex_char, first_printable_char};

use std::collections::{BTreeMap, BTreeSet};

use iced::advanced::layout::{Layout, Limits, Node};
use iced::advanced::renderer;
use iced::advanced::widget::{tree, Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Element, Event, Length, Rectangle, Size};

use crate::coloring::ColorScheme;
use crate::domain::write_mode::WriteMode;
use crate::selection::{NavDir, Selection};
use crate::ui::view::minimap::MINIMAP_WIDTH;
use gui_widgets::components::paragraph_cache::ParagraphCache;

// ── The HexMatrix widget ──────────────────────────────────────────────

/// A virtualized hex-matrix widget.
///
/// Renders a file as a scrollable grid of hex bytes with an ASCII gutter
/// on the right, an address gutter on the left, a minimap overview strip,
/// search-match highlights, inline editing support, and both vertical and
/// horizontal scrollbars.
pub struct HexMatrix<'a, Message> {
    pub(super) bytes: &'a [u8],
    pub(super) bytes_per_row: u8,
    pub(super) selection: Selection,
    pub(super) edit: Option<EditView<'a>>,
    pub(super) dirty: &'a BTreeSet<u64>,
    pub(super) vanilla_diff: &'a BTreeSet<u64>,
    pub(super) patterns: &'a BTreeMap<u64, (usize, u8)>,
    pub(super) search_match_set: &'a BTreeSet<u64>,
    pub(super) search_query_len: u64,
    pub(super) search_current_addr: Option<u64>,
    pub(super) search_match_starts: &'a [u64],
    pub(super) row_annotations: &'a BTreeMap<u64, Vec<(usize, String)>>,
    pub(super) active_patterns: &'a BTreeSet<usize>,
    pub(super) alternate_patterns: BTreeSet<usize>,
    pub(super) cache: ParagraphCache,
    pub(super) width: Length,
    pub(super) height: Length,
    pub(super) on_select_at: Option<Box<dyn Fn(u64) -> Message + 'a>>,
    pub(super) on_extend_to: Option<Box<dyn Fn(u64) -> Message + 'a>>,
    pub(super) on_nav: Option<Box<dyn Fn(NavDir, bool) -> Message + 'a>>,
    pub(super) on_begin_edit: Option<Box<dyn Fn(u64) -> Message + 'a>>,
    pub(super) on_edit_type: Option<Box<dyn Fn(char) -> Message + 'a>>,
    pub(super) on_edit_backspace: Option<Box<dyn Fn() -> Message + 'a>>,
    pub(super) on_edit_cancel: Option<Box<dyn Fn() -> Message + 'a>>,
    pub(super) on_delete_byte: Option<Box<dyn Fn() -> Message + 'a>>,
    pub(super) on_edit_commit: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    pub(super) on_right_click: Option<Box<dyn Fn(u64) -> Message + 'a>>,
    pub(super) on_create_pattern: Option<Box<dyn Fn() -> Message + 'a>>,
    pub(super) on_open_goto: Option<Box<dyn Fn() -> Message + 'a>>,
    pub(super) on_open_search: Option<Box<dyn Fn() -> Message + 'a>>,
    pub(super) on_copy_selection: Option<Box<dyn Fn() -> Message + 'a>>,
    pub(super) on_paste: Option<Box<dyn Fn() -> Message + 'a>>,
    pub(super) show_decimal: bool,
    pub(super) on_toggle_addr_format: Option<Box<dyn Fn() -> Message + 'a>>,
    pub(super) color_scheme: ColorScheme,
    pub(super) dim_nulls: bool,
    pub(super) write_mode: WriteMode,
    pub(super) entropy_bands: Option<&'a [(u64, f64)]>,
    pub(super) show_minimap: bool,
}

// ── Constructor ───────────────────────────────────────────────────────

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
        HexMatrix {
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
}

// ── Builder methods ───────────────────────────────────────────────────

impl<'a, Message> HexMatrix<'a, Message> {
    pub fn show_minimap(mut self, v: bool) -> Self {
        self.show_minimap = v;
        self
    }

    pub fn show_decimal(mut self, v: bool) -> Self {
        self.show_decimal = v;
        self
    }

    pub fn dim_nulls(mut self, v: bool) -> Self {
        self.dim_nulls = v;
        self
    }

    pub fn color_scheme(mut self, v: ColorScheme) -> Self {
        self.color_scheme = v;
        self
    }

    pub fn write_mode(mut self, v: WriteMode) -> Self {
        self.write_mode = v;
        self
    }

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

    pub fn on_toggle_addr_format(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_toggle_addr_format = Some(Box::new(f));
        self
    }
}

// ── Geometry helper methods ───────────────────────────────────────────

impl<'a, Message> HexMatrix<'a, Message> {
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
        self.total_rows() as f32 * layout::ROW_HEIGHT
    }

    fn ascii_start_x(&self, bounds_x: f32) -> f32 {
        let bpr = self.bytes_per_row as usize;
        bounds_x
            + self.addr_col_width()
            + (bpr as f32) * layout::HEX_CELL_WIDTH
            + layout::group_count(bpr) as f32 * layout::GROUP_GAP
            + layout::COLUMN_GAP
    }

    fn annotation_col_width(&self) -> f32 {
        if self.row_annotations.is_empty() {
            return 0.0;
        }
        let max_chars = self
            .row_annotations
            .values()
            .map(|segments| {
                let text_len: usize = segments.iter().map(|(_, t)| t.len()).sum();
                let separators = segments.len().saturating_sub(1) * 3;
                text_len + separators
            })
            .max()
            .unwrap_or(0);
        let estimated = max_chars as f32 * layout::ASCII_CELL_WIDTH;
        estimated.clamp(layout::MIN_ANN_COL_WIDTH, layout::MAX_ANN_COL_WIDTH)
    }

    fn annotation_start_x(&self, bounds_x: f32) -> f32 {
        self.ascii_start_x(bounds_x)
            + (self.bytes_per_row as f32) * layout::ASCII_CELL_WIDTH
            + layout::ANN_COL_GAP
    }

    fn total_content_width(&self) -> f32 {
        let bpr = self.bytes_per_row as usize;
        let mut w = self.addr_col_width()
            + (bpr as f32) * layout::HEX_CELL_WIDTH
            + layout::group_count(bpr) as f32 * layout::GROUP_GAP
            + layout::COLUMN_GAP
            + (bpr as f32) * layout::ASCII_CELL_WIDTH;
        if !self.row_annotations.is_empty() {
            w += layout::ANN_COL_GAP + self.annotation_col_width();
        }
        w
    }

    fn content_viewport_h(&self, bounds_h: f32, bounds_w: f32) -> f32 {
        let right_reserved = self.right_strip();
        let needs_hscroll = self.total_content_width() > bounds_w - right_reserved;
        if needs_hscroll {
            (bounds_h - layout::HEADER_HEIGHT - layout::SCROLLBAR_THICKNESS).max(0.0)
        } else {
            (bounds_h - layout::HEADER_HEIGHT).max(0.0)
        }
    }

    fn right_strip(&self) -> f32 {
        if self.show_minimap {
            layout::SCROLLBAR_THICKNESS + MINIMAP_WIDTH
        } else {
            layout::SCROLLBAR_THICKNESS
        }
    }
}

// ── publish_nav — navigation helper ───────────────────────────────────

impl<'a, Message> HexMatrix<'a, Message> {
    fn publish_nav(
        &self,
        state: &mut state::State,
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
        let bpr = self.bytes_per_row as u64;
        let max_addr = (self.bytes.len() as u64).saturating_sub(1);
        let viewport_h = self.content_viewport_h(bounds.height, bounds.width);
        let target = crate::selection::nav_target(
            self.selection.cursor,
            dir,
            bpr,
            layout::page_rows(viewport_h),
            max_addr,
        );
        let new_scroll = layout::ensure_visible(
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

// ── Widget trait implementation ───────────────────────────────────────

impl<'a, Message, Theme> Widget<Message, Theme, iced::Renderer> for HexMatrix<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<state::State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(state::State::default())
    }

    fn diff(&self, _tree: &mut Tree) {
        // State is simple enough that the default diff is fine.
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &iced::Renderer, limits: &Limits) -> Node {
        Node::new(limits.resolve(self.width, self.height, Size::new(800.0, 320.0)))
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
        let state = tree.state.downcast_mut::<state::State>();
        event::handle_event(self, state, event, layout, cursor, shell);
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
        let state = tree.state.downcast_ref::<state::State>();
        draw::draw_matrix(self, state, renderer, layout, cursor, viewport);
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
}

// ── From<HexMatrix> for Element ───────────────────────────────────────

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

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::layout::*;
    use super::draw::*;
    use iced::{Point, Rectangle};

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
        assert_eq!(first_printable_char("\t"), None);
        assert_eq!(first_printable_char("\n"), None);
        assert_eq!(first_printable_char("\0"), None);
    }

    #[test]
    fn first_printable_char_empty_string() {
        assert_eq!(first_printable_char(""), None);
    }

    #[test]
    fn header_constants_are_reasonable() {
        assert_eq!(HEADER_HEIGHT, 16.0);
        assert_eq!(HEADER_HEIGHT, ROW_HEIGHT);
    }

    #[test]
    fn addr_at_returns_none_for_clicks_in_header_area() {
        let bounds = Rectangle {
            x: 0.0,
            y: HEADER_HEIGHT,
            width: 800.0,
            height: 300.0,
        };
        let p = Point::new(100.0, 0.0);
        assert!(
            addr_at(p, bounds, 0.0, 0.0, 16, 1024, TEST_ADDR_COL_WIDTH).is_none(),
            "click in header area should not resolve to a byte address"
        );
    }

    #[test]
    fn header_does_not_affect_addr_at_after_content_bounds() {
        let bpr: u8 = 16;
        let aw = bpr as f32 * HEX_CELL_WIDTH + group_count(bpr as usize) as f32 * GROUP_GAP
            + TEST_ADDR_COL_WIDTH
            + COLUMN_GAP
            + bpr as f32 * ASCII_CELL_WIDTH;
        let bounds = Rectangle {
            x: 0.0,
            y: HEADER_HEIGHT,
            width: aw + 100.0,
            height: 300.0,
        };
        let hex_x = TEST_ADDR_COL_WIDTH + 4.0;
        assert!(
            addr_at(Point::new(hex_x, 0.0), bounds, 0.0, 0.0, bpr, 1024, TEST_ADDR_COL_WIDTH)
                .is_none(),
            "click at y=0 (in header) should be rejected by content bounds"
        );
        assert!(
            addr_at(Point::new(hex_x, HEADER_HEIGHT + 4.0), bounds, 0.0, 0.0, bpr, 1024, TEST_ADDR_COL_WIDTH)
                .is_some(),
            "click within content bounds should resolve"
        );
    }
}
