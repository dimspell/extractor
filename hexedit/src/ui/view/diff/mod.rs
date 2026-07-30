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
    pub(super) theme: &'static HexEditorTheme,

    // ── Rendering ──────────────────────────────────────────────────────
    pub(super) cache: ParagraphCache,
    pub(super) width: Length,
    pub(super) height: Length,

    // ── Callbacks ──────────────────────────────────────────────────────
    /// Called when the user clicks/navigates to a byte address.
    pub(super) on_select_at: Option<Box<dyn Fn(u64) -> Message + 'a>>,
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
            cache,
            width: Length::Fill,
            height: Length::Fill,
            on_select_at: None,
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

    pub fn show_decimal(mut self, v: bool) -> Self {
        self.show_decimal = v;
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
    use iced::widget::container;
    use iced::Fill;

    let Some(ref cf) = state.comparison_file else {
        return container(
            iced::widget::text("No comparison file loaded. Right-click to select one.")
                .size(11),
        )
        .width(Fill)
        .height(Fill)
        .center_x(Fill)
        .center_y(Fill)
        .into();
    };

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
    let clamped_sel = if state.selection.cursor > max_addr as u64 {
        crate::domain::selection::Selection::single(max_addr as u64)
    } else {
        state.selection
    };

    DiffView::new(
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
    .on_select_at(move |addr| crate::HexEditorMessage::DiffAddrSelected(addr))
    .into()
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use gui_widgets::components::paragraph_cache::ParagraphCache;

    use super::layout::*;

    fn sel() -> crate::domain::selection::Selection {
        crate::domain::selection::Selection::default()
    }

    #[test]
    fn empty_buffers_yield_zero_rows() {
        let pats: BTreeMap<u64, (usize, u8)> = BTreeMap::new();
        let ann: BTreeMap<u64, Vec<(usize, String)>> = BTreeMap::new();
        let diff = BTreeSet::new();
        let search = BTreeSet::new();
        let active = BTreeSet::new();
        let dv = super::DiffView::<()>::new(
            &[], &[], 16, sel(), &diff, &pats, &search, 0, None, &[], &ann, &active,
            BTreeSet::new(), ParagraphCache::default(),
            crate::coloring::ColorScheme::Monochrome, false,
            &crate::ui::theme::DARK_THEME,
        );
        assert_eq!(dv.total_rows(), 0);
    }

    #[test]
    fn total_rows_computed_from_longer_buffer() {
        let pats: BTreeMap<u64, (usize, u8)> = BTreeMap::new();
        let ann: BTreeMap<u64, Vec<(usize, String)>> = BTreeMap::new();
        let diff = BTreeSet::new();
        let search = BTreeSet::new();
        let active = BTreeSet::new();
        let dv = super::DiffView::<()>::new(
            &[0u8; 32], &[0u8; 48], 16, sel(), &diff, &pats, &search, 0, None, &[], &ann, &active,
            BTreeSet::new(), ParagraphCache::default(),
            crate::coloring::ColorScheme::Monochrome, false,
            &crate::ui::theme::DARK_THEME,
        );
        assert_eq!(dv.total_rows(), 3);
    }

    #[test]
    fn right_strip_is_scrollbar_only() {
        let pats: BTreeMap<u64, (usize, u8)> = BTreeMap::new();
        let ann: BTreeMap<u64, Vec<(usize, String)>> = BTreeMap::new();
        let diff = BTreeSet::new();
        let search = BTreeSet::new();
        let active = BTreeSet::new();
        let dv = super::DiffView::<()>::new(
            &[0u8; 16], &[0u8; 16], 16, sel(), &diff, &pats, &search, 0, None, &[], &ann, &active,
            BTreeSet::new(), ParagraphCache::default(),
            crate::coloring::ColorScheme::Monochrome, false,
            &crate::ui::theme::DARK_THEME,
        );
        assert_eq!(dv.right_strip(), SCROLLBAR_THICKNESS);
    }
}
