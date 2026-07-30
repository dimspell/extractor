//! Custom Iced widget rendering a side-by-side binary diff view.
//!
//! Renders two byte buffers (baseline and comparison) side-by-side within a
//! single pane, sharing an address column and a single scrollbar. Bytes that
//! differ are colour-tinted (reddish on baseline, greenish on comparison).
//! Pattern overlays and annotation columns are rendered on both sides.
//!
//! Layout (left → right):
//! ```text
//! [addr_col] [hex_A] [ascii_A] [mid_gap] [hex_B] [ascii_B] [ann_col]
//! ```
//!
//! The diff view is **read-only** — no inline editing. The user can navigate
//! with mouse/keyboard and select byte addresses, but edits go through the
//! normal hex matrix pane instead.

mod draw;
mod event;
mod layout;
mod state;

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

use iced::advanced::layout::{Limits, Node};
use iced::advanced::renderer;
use iced::advanced::widget::{tree, Tree, Widget};
use iced::mouse;
use iced::{Element, Length, Rectangle, Size};

use crate::coloring::ColorScheme;
use crate::domain::selection::Selection;
use crate::ui::theme::HexEditorTheme;
use gui_widgets::components::paragraph_cache::ParagraphCache;

use self::layout::{HEADER_HEIGHT, ROW_HEIGHT, SCROLLBAR_THICKNESS};

/// A virtualized side-by-side binary diff widget.
///
/// Two byte buffers are rendered within the same pane, sharing a single
/// address column and scroll offset. Diff-detection marks bytes that differ
/// between the two buffers with tinted backgrounds.
pub struct DiffView<'a, Message> {
    // ── Data sources ───────────────────────────────────────────────────
    /// Baseline file bytes (left side).
    pub(super) baseline_bytes: &'a [u8],
    /// Comparison file bytes (right side).
    pub(super) comparison_bytes: &'a [u8],
    /// How many bytes per row.
    pub(super) bytes_per_row: u8,

    // ── Selection ──────────────────────────────────────────────────────
    /// Shared cursor / selection across both sides.
    pub(super) selection: Selection,
    /// Addresses that differ between baseline and comparison.
    pub(super) diff: &'a BTreeSet<u64>,

    // ── Pattern overlays ───────────────────────────────────────────────
    pub(super) patterns: &'a BTreeMap<u64, (usize, u8)>,
    pub(super) row_annotations: &'a BTreeMap<u64, Vec<(usize, String)>>,
    pub(super) active_patterns: &'a BTreeSet<usize>,
    pub(super) alternate_patterns: BTreeSet<usize>,

    // ── Search ─────────────────────────────────────────────────────────
    pub(super) search_match_set: &'a BTreeSet<u64>,
    pub(super) search_query_len: u64,
    pub(super) search_current_addr: Option<u64>,
    pub(super) search_match_starts: &'a [u64],

    // ── Display options ────────────────────────────────────────────────
    pub(super) color_scheme: ColorScheme,
    pub(super) dim_nulls: bool,
    pub(super) show_decimal: bool,
    pub(super) diff_review: bool,
    pub(super) theme: &'static HexEditorTheme,

    // ── Rendering ──────────────────────────────────────────────────────
    pub(super) cache: ParagraphCache,
    pub(super) width: Length,
    pub(super) height: Length,

    /// One-shot center-on request consumed by the draw method.
    pub(super) pending_center_on: Cell<Option<u64>>,

    // ── Callbacks ──────────────────────────────────────────────────────
    /// Called when the user clicks/navigates to a byte address.
    pub(super) on_select_at: Option<Box<dyn Fn(u64) -> Message + 'a>>,
    /// Called when the user right-clicks a byte address (for context menus).
    pub(super) on_right_click: Option<Box<dyn Fn(u64) -> Message + 'a>>,
    /// Called when the user extends selection to an address (shift-click / drag).
    pub(super) on_extend_to: Option<Box<dyn Fn(u64) -> Message + 'a>>,
    /// Called on arrow/navigation key (dir, extend).
    pub(super) on_nav: Option<Box<dyn Fn(crate::domain::selection::NavDir, bool) -> Message + 'a>>,
    /// Called on Ctrl+Down → jump to next diff chunk.
    pub(super) on_diff_nav_next: Option<Box<dyn Fn() -> Message + 'a>>,
    /// Called on Ctrl+Up → jump to previous diff chunk.
    pub(super) on_diff_nav_prev: Option<Box<dyn Fn() -> Message + 'a>>,
}

// ── Constructor ─────────────────────────────────────────────────────────

impl<'a, Message> DiffView<'a, Message> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        baseline_bytes: &'a [u8],
        comparison_bytes: &'a [u8],
        bytes_per_row: u8,
        selection: Selection,
        diff: &'a BTreeSet<u64>,
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
        theme: &'static HexEditorTheme,
    ) -> Self {
        DiffView {
            baseline_bytes,
            comparison_bytes,
            bytes_per_row: bytes_per_row.max(1),
            selection,
            diff,
            patterns,
            search_match_set,
            search_query_len,
            search_current_addr,
            search_match_starts,
            row_annotations,
            active_patterns,
            alternate_patterns,
            color_scheme,
            dim_nulls,
            show_decimal: false,
            diff_review: false,
            cache,
            width: Length::Fill,
            height: Length::Fill,
            pending_center_on: Cell::new(None),
            on_select_at: None,
            on_right_click: None,
            on_extend_to: None,
            on_nav: None,
            on_diff_nav_next: None,
            on_diff_nav_prev: None,
            theme,
        }
    }
}

// ── Builder methods ─────────────────────────────────────────────────────

impl<'a, Message> DiffView<'a, Message> {
    pub fn on_select_at(mut self, f: impl Fn(u64) -> Message + 'a) -> Self {
        self.on_select_at = Some(Box::new(f));
        self
    }

    pub fn on_right_click(mut self, f: impl Fn(u64) -> Message + 'a) -> Self {
        self.on_right_click = Some(Box::new(f));
        self
    }

    pub fn on_extend_to(mut self, f: impl Fn(u64) -> Message + 'a) -> Self {
        self.on_extend_to = Some(Box::new(f));
        self
    }

    pub fn on_nav(mut self, f: impl Fn(crate::domain::selection::NavDir, bool) -> Message + 'a) -> Self {
        self.on_nav = Some(Box::new(f));
        self
    }

    pub fn show_decimal(mut self, v: bool) -> Self {
        self.show_decimal = v;
        self
    }

    /// Enable "Show Diffs Only" mode — hides rows that have zero differing
    /// bytes between the two buffers.
    pub fn diff_review(mut self, v: bool) -> Self {
        self.diff_review = v;
        self
    }

    /// Ctrl+Down → jump to next diff chunk.
    pub fn on_diff_nav_next(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_diff_nav_next = Some(Box::new(f));
        self
    }

    /// Ctrl+Up → jump to previous diff chunk.
    pub fn on_diff_nav_prev(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_diff_nav_prev = Some(Box::new(f));
        self
    }

    pub fn center_on(self, addr: Option<u64>) -> Self {
        self.pending_center_on.set(addr);
        self
    }
}

// ── Geometry helper methods ────────────────────────────────────────────

impl<'a, Message> DiffView<'a, Message> {
    pub(super) fn total_rows(&self) -> u64 {
        let bpr = self.bytes_per_row as usize;
        let total = self
            .baseline_bytes
            .len()
            .max(self.comparison_bytes.len());
        if total == 0 {
            0
        } else {
            total.div_ceil(bpr) as u64
        }
    }

    #[allow(dead_code)]
    fn total_height(&self) -> f32 {
        self.total_rows() as f32 * ROW_HEIGHT
    }

    pub(super) fn content_viewport_h(&self, bounds_h: f32, bounds_w: f32) -> f32 {
        let bpr = self.bytes_per_row as usize;
        let right_reserved = self.right_strip();
        let needs_hscroll =
            layout::total_content_width(bpr, !self.row_annotations.is_empty()) > bounds_w - right_reserved;
        if needs_hscroll {
            (bounds_h - HEADER_HEIGHT - SCROLLBAR_THICKNESS).max(0.0)
        } else {
            (bounds_h - HEADER_HEIGHT).max(0.0)
        }
    }

    pub(super) fn right_strip(&self) -> f32 {
        // No minimap in diff view, just scrollbar.
        SCROLLBAR_THICKNESS
    }

    /// Whether the row starting at `base_addr` contains any differing bytes.
    pub(super) fn row_has_diff(&self, base_addr: u64) -> bool {
        let bpr = self.bytes_per_row as u64;
        self.diff
            .range(base_addr..base_addr + bpr)
            .next()
            .is_some()
    }
}

// ── Widget trait implementation ───────────────────────────────────────

impl<'a, Message, Theme> Widget<Message, Theme, iced::Renderer> for DiffView<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<state::State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(state::State::default())
    }

    fn diff(&mut self, _tree: &mut Tree) {
        // Default diff is fine — state is simple.
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &Limits,
    ) -> Node {
        Node::new(limits.resolve(self.width, self.height, Size::new(800.0, 320.0)))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: iced::advanced::layout::Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        shell: &mut iced::advanced::Shell<'_, Message>,
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
        layout: iced::advanced::layout::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<state::State>();
        draw::draw_diff_view(self, state, renderer, layout, cursor, viewport);
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: iced::advanced::layout::Layout<'_>,
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

// ── From<DiffView> for Element ─────────────────────────────────────────

impl<'a, Message, Theme> From<DiffView<'a, Message>>
    for Element<'a, Message, Theme, iced::Renderer>
where
    Theme: 'a,
    Message: 'a,
{
    fn from(w: DiffView<'a, Message>) -> Self {
        Element::new(w)
    }
}

// ── View function (called from panel.rs) ───────────────────────────────

use std::collections::BTreeSet as BSet;

/// Build the diff view element from the hex editor state.
///
/// Called by the pane dispatcher in `panel.rs`. Returns either a
/// placeholder message or a fully constructed `DiffView` widget.
pub fn view<'a>(
    state: &'a crate::HexEditorState,
    _config: &crate::config::HexEditorConfig,
) -> iced::Element<'a, crate::HexEditorMessage> {
    use crate::domain::provider::HexProvider;
    use iced::widget::{button, column, container, row, text};
    use iced::{Fill, Font};

    let Some(ref cf) = state.comparison_file else {
        return container(
            text("No comparison file loaded. Right-click to select one.")
                .size(11),
        )
        .width(Fill)
        .height(Fill)
        .center_x(Fill)
        .center_y(Fill)
        .into();
    };

    // ── Diff header bar with close button ──
    let close_btn = button(
        row![
            text("✕").size(12).font(Font::MONOSPACE),
            text(" Close Diff").size(11).font(Font::MONOSPACE),
        ]
        .spacing(2)
        .align_y(iced::Alignment::Center),
    )
    .padding([2, 8])
    .on_press(crate::HexEditorMessage::CloseComparison);

    let comparison_name = text(&cf.name).size(11).font(Font::MONOSPACE);
    let header = container(
        row![
            comparison_name,
            container(close_btn).width(Fill).align_x(iced::alignment::Horizontal::Right),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    )
    .padding([4, 12]);

    // Compute zebra-striping for patterns (same logic as matrix_content).
    let mut alternate_patterns = BSet::new();
    for group in &state.groups {
        let mut group_patterns: Vec<&crate::domain::pattern::Pattern> = state
            .patterns
            .iter()
            .filter(|p| p.group_id == Some(group.id))
            .collect();
        group_patterns.sort_by_key(|p| (p.start, p.id));
        for (i, pat) in group_patterns.iter().enumerate() {
            if i % 2 == 1 {
                alternate_patterns.insert(pat.id);
            }
        }
    }

    let total_bytes = state.provider.len().max(cf.data.len() as u64);
    let max_addr = total_bytes.saturating_sub(1);

    // Clamp selection cursor to within the longer buffer.
    let clamped_sel = if state.selection.cursor > max_addr {
        crate::domain::selection::Selection::single(max_addr)
    } else {
        state.selection
    };

    let diff_view: iced::Element<'a, crate::HexEditorMessage> = DiffView::new(
        state.provider.as_slice(),
        &cf.data,
        state.bytes_per_row,
        clamped_sel,
        &cf.diff,
        &state.pattern_by_addr,
        &state.search.match_set,
        state.search.query_len,
        state.search.current_addr(),
        &state.search.results,
        &state.row_annotations,
        &state.active_patterns,
        alternate_patterns,
        state.cache.clone(),
        state.color_scheme,
        state.dim_nulls,
        state.theme,
    )
    .on_select_at(crate::HexEditorMessage::DiffAddrSelected)
    .on_extend_to(crate::HexEditorMessage::ExtendTo)
    .on_right_click(crate::HexEditorMessage::RightClickAt)
    .on_nav(|dir, extend| crate::HexEditorMessage::Nav { dir, extend })
    .on_diff_nav_next(|| crate::HexEditorMessage::DiffNavNext)
    .on_diff_nav_prev(|| crate::HexEditorMessage::DiffNavPrev)
    .center_on(state.pending_center_on.take())
    .diff_review(state.diff_review)
    .into();

    column![header, diff_view].spacing(0).width(Fill).height(Fill).into()
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use gui_widgets::components::paragraph_cache::ParagraphCache;

    use super::layout::*;
    use super::draw::col_at_x;

    fn sel() -> crate::domain::selection::Selection {
        crate::domain::selection::Selection::default()
    }

    fn empty_set() -> BTreeSet<u64> { BTreeSet::new() }
    fn empty_map() -> BTreeMap<u64, (usize, u8)> { BTreeMap::new() }
    fn empty_ann() -> BTreeMap<u64, Vec<(usize, String)>> { BTreeMap::new() }
    fn empty_active() -> BTreeSet<usize> { BTreeSet::new() }

    /// Construct a minimal DiffView for tests. All non-essential buffers
    /// reference static empty collections.
    fn minimal_dv<'a>(a: &'a [u8], b: &'a [u8], bpr: u8) -> super::DiffView<'a, ()> {
        static EMPTY_SET: BTreeSet<u64> = BTreeSet::new();
        static EMPTY_MAP: BTreeMap<u64, (usize, u8)> = BTreeMap::new();
        static EMPTY_ANN: BTreeMap<u64, Vec<(usize, String)>> = BTreeMap::new();
        static EMPTY_ACTIVE: BTreeSet<usize> = BTreeSet::new();
        super::DiffView::new(
            a, b, bpr, sel(),
            &EMPTY_SET, &EMPTY_MAP, &EMPTY_SET,
            0, None, &[], &EMPTY_ANN, &EMPTY_ACTIVE,
            BTreeSet::new(), ParagraphCache::default(),
            crate::coloring::ColorScheme::Monochrome, false,
            &crate::ui::theme::DARK_THEME,
        )
    }

    // ── Constructor / geometry ─────────────────────────────────────────

    #[test]
    fn empty_buffers_yield_zero_rows() {
        let dv = minimal_dv(&[], &[], 16);
        assert_eq!(dv.total_rows(), 0);
    }

    #[test]
    fn total_rows_computed_from_longer_buffer() {
        let dv = minimal_dv(&[0u8; 32], &[0u8; 48], 16);
        assert_eq!(dv.total_rows(), 3);
    }

    #[test]
    fn right_strip_is_scrollbar_only() {
        let dv = minimal_dv(&[0u8; 16], &[0u8; 16], 16);
        assert_eq!(dv.right_strip(), SCROLLBAR_THICKNESS);
    }

    // ── Builder methods ────────────────────────────────────────────────

    #[test]
    fn builder_on_right_click_sets_callback() {
        let called = std::cell::Cell::new(None);
        let dv = minimal_dv(&[0], &[0], 16)
            .on_right_click(|addr| { called.set(Some(addr)); () });
        let cb = dv.on_right_click.as_ref().unwrap();
        cb(42);
        assert_eq!(called.get(), Some(42));
    }

    #[test]
    fn builder_on_extend_to_sets_callback() {
        let called = std::cell::Cell::new(None);
        let dv = minimal_dv(&[0], &[0], 16)
            .on_extend_to(|addr| { called.set(Some(addr)); () });
        let cb = dv.on_extend_to.as_ref().unwrap();
        cb(99);
        assert_eq!(called.get(), Some(99));
    }

    #[test]
    fn builder_on_nav_sets_callback() {
        // NavDir does not implement PartialEq, so verify invocation by flag
        let called = std::cell::Cell::new(false);
        let dv = minimal_dv(&[0], &[0], 16)
            .on_nav(|_, _| { called.set(true); () });
        let cb = dv.on_nav.as_ref().unwrap();
        cb(crate::domain::selection::NavDir::Right, true);
        assert!(called.get(), "on_nav callback should have been invoked");
    }

    #[test]
    fn builder_show_decimal_sets_flag() {
        let dv = minimal_dv(&[0], &[0], 16).show_decimal(true);
        assert!(dv.show_decimal);

        let dv = minimal_dv(&[0], &[0], 16).show_decimal(false);
        assert!(!dv.show_decimal);
    }

    // ── col_at_x coordinate mapping ─────────────────────────────────────
    //
    // Layout for bpr=16:
    //   [ADDR=88] [hex_A (16*18 + gaps)] [ascii_A (16*9)] [MID_GAP=18]
    //   [hex_B (16*18 + gaps)] [ascii_B (16*9)] [ANN gap]

    fn hex_a_start() -> f32 {
        baseline_hex_start(ADDR_COL_WIDTH)
    }

    fn ascii_a_start() -> f32 {
        baseline_ascii_start(ADDR_COL_WIDTH, 16)
    }

    fn comp_hex_start() -> f32 {
        comparison_hex_start(ADDR_COL_WIDTH, 16)
    }

    fn comp_ascii_start() -> f32 {
        comparison_ascii_start(ADDR_COL_WIDTH, 16)
    }

    #[test]
    fn col_at_x_baseline_hex_first_byte() {
        let x = hex_a_start() + 2.0;
        let (col, is_baseline) = col_at_x(x, 16).unwrap();
        assert_eq!(col, 0);
        assert!(is_baseline);
    }

    #[test]
    fn col_at_x_baseline_hex_tenth_byte() {
        let x = hex_a_start() + 10.0 * HEX_CELL_WIDTH + GROUP_GAP;
        let (col, is_baseline) = col_at_x(x, 16).unwrap();
        assert_eq!(col, 10);
        assert!(is_baseline);
    }

    #[test]
    fn col_at_x_baseline_ascii_first_byte() {
        let x = ascii_a_start() + 2.0;
        let (col, is_baseline) = col_at_x(x, 16).unwrap();
        assert_eq!(col, 0);
        assert!(is_baseline);
    }

    #[test]
    fn col_at_x_baseline_ascii_last_byte() {
        let x = ascii_a_start() + 15.0 * ASCII_CELL_WIDTH + 1.0;
        let (col, is_baseline) = col_at_x(x, 16).unwrap();
        assert_eq!(col, 15);
        assert!(is_baseline);
    }

    #[test]
    fn col_at_x_comparison_hex_first_byte() {
        let x = comp_hex_start() + 2.0;
        let (col, is_baseline) = col_at_x(x, 16).unwrap();
        assert_eq!(col, 0);
        assert!(!is_baseline);
    }

    #[test]
    fn col_at_x_comparison_ascii_middle_byte() {
        let x = comp_ascii_start() + 7.0 * ASCII_CELL_WIDTH + 1.0;
        let (col, is_baseline) = col_at_x(x, 16).unwrap();
        assert_eq!(col, 7);
        assert!(!is_baseline);
    }

    #[test]
    fn col_at_x_address_gutter_returns_none() {
        let x = ADDR_COL_WIDTH - 4.0;
        assert!(col_at_x(x, 16).is_none());
    }

    #[test]
    fn col_at_x_mid_gap_returns_none() {
        let x = ascii_a_start() + 16.0 * ASCII_CELL_WIDTH + 2.0;
        assert!(col_at_x(x, 16).is_none());
    }

    #[test]
    fn col_at_x_beyond_ascii_b_returns_none() {
        let x = comp_ascii_start() + 16.0 * ASCII_CELL_WIDTH + 20.0;
        assert!(col_at_x(x, 16).is_none());
    }

    #[test]
    fn col_at_x_bpr_8_baseline() {
        let x = baseline_hex_start(ADDR_COL_WIDTH) + 3.0 * HEX_CELL_WIDTH;
        let (col, is_baseline) = col_at_x(x, 8).unwrap();
        assert_eq!(col, 3);
        assert!(is_baseline);
    }

    #[test]
    fn col_at_x_bpr_32_comparison() {
        let x = comparison_hex_start(ADDR_COL_WIDTH, 32) + 25.0 * HEX_CELL_WIDTH + 3.0 * GROUP_GAP;
        let (col, is_baseline) = col_at_x(x, 32).unwrap();
        assert_eq!(col, 25);
        assert!(!is_baseline);
    }

    // ── Diff-specific layout functions ──────────────────────────────────

    #[test]
    fn diff_total_content_width_no_annotations() {
        let side_a = ADDR_COL_WIDTH
            + 16.0 * HEX_CELL_WIDTH + 1.0 * GROUP_GAP + COLUMN_GAP + 16.0 * ASCII_CELL_WIDTH;
        let side_b = 16.0 * HEX_CELL_WIDTH + 1.0 * GROUP_GAP + COLUMN_GAP + 16.0 * ASCII_CELL_WIDTH;
        let expected = side_a + MID_GAP + side_b;
        assert_eq!(total_content_width(16, false), expected);
    }

    #[test]
    fn diff_total_content_width_with_annotations() {
        let without_ann = total_content_width(16, false);
        let with_ann = total_content_width(16, true);
        assert_eq!(with_ann - without_ann, ANN_COL_GAP + MAX_ANN_COL_WIDTH);
    }

    #[test]
    fn diff_layout_column_starts_are_monotonic() {
        let bpr = 16;
        let hex_a = baseline_hex_start(ADDR_COL_WIDTH);
        let ascii_a = baseline_ascii_start(ADDR_COL_WIDTH, bpr);
        let comp_hex = comparison_hex_start(ADDR_COL_WIDTH, bpr);
        let comp_ascii = comparison_ascii_start(ADDR_COL_WIDTH, bpr);
        assert!(hex_a < ascii_a, "hex_a before ascii_a");
        assert!(ascii_a < comp_hex, "ascii_a before comp_hex");
        assert!(comp_hex < comp_ascii, "comp_hex before comp_ascii");
    }

    // ── State defaults ──────────────────────────────────────────────────

    #[test]
    fn state_dragging_cursor_starts_false() {
        use super::state::State;
        let s = State::default();
        assert!(!s.dragging_cursor);
    }
}
