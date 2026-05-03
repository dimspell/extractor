//! Spreadsheet-style editor view.
//!
//! A richly-styled data grid inspired by Excel and the plan in
//! `dispel-gui/docs/spreadsheet-redesign-plan.md`.  Highlights:
//!
//! * Sticky header row with clickable sort arrows (`▲` / `▼`).
//! * Zebra striping, hover affordance, gold selection accent, red invalid cells.
//! * Frozen "#" column matching the row-number column of Excel.
//! * Dual-mode global filter (`Filter` / `Highlight`) with prev/next navigation
//!   through highlighted matches.
//! * Single-click-to-select, click-again-to-edit cell UX.
//! * VIM-style `NORMAL` / `EDIT` mode indicator in the status bar.
//! * One-click CSV export of the currently-filtered view.

use crate::components::editor::editable::{EditableRecord, FieldDescriptor, FieldKind};
use crate::components::textarea::{self, TextAreaContent};
use crate::generic_editor::GenericEditorState;
use crate::message::{Message, SystemMessage};
use crate::style;
use crate::utils::{horizontal_rule, horizontal_space};
#[cfg(not(feature = "table_widget"))]
use crate::view::editor::cached_text::cached_text;
use crate::view::editor::cached_text::ParagraphCache;
use iced::widget::pane_grid::{self, Pane};
#[cfg(not(feature = "table_widget"))]
use iced::widget::scrollable::Direction as ScrollDir;
use iced::widget::text_editor;
use iced::widget::{
    button, column, container, pick_list, progress_bar, row, scrollable, text, text_input, Column,
};
#[cfg(not(feature = "table_widget"))]
use iced::Font;
use iced::{Element, Fill, Length};
use std::collections::HashMap;

const ROW_HEIGHT: f32 = 24.0;
/// Baseline overscan applied even when the user is idle. Small enough to keep
/// the rendered widget tree shallow during slow exploration.
const OVERSCAN_ROWS_BASE: usize = 32;
/// Hard ceiling for adaptive overscan during the fastest fling-scrolls.
/// Past this point the per-frame rebuild starts to dominate any benefit from
/// pre-warming additional rows.
const OVERSCAN_ROWS_MAX: usize = 512;
/// Translates absolute scroll velocity (in px/s) into extra overscan rows:
/// `extra = |velocity_px_per_s| / ROW_HEIGHT / OVERSCAN_VELOCITY_DIVISOR`.
/// `1` means "1 second of look-ahead at the current velocity"; lower numbers
/// scale up faster.
const OVERSCAN_VELOCITY_DIVISOR: f32 = 2.0;
/// The virtual window boundary only shifts when `first_row` crosses a multiple
/// of this value.  Snapping prevents a widget-tree rebuild (and costly
/// cosmic-text Korean glyph layout) on every single scroll pixel.
#[cfg(not(feature = "table_widget"))]
const WINDOW_STEP: usize = 64;
const ID_COL_WIDTH_PX: f32 = 42.0;
#[cfg(not(feature = "table_widget"))]
const ID_COL_WIDTH: Length = Length::Fixed(ID_COL_WIDTH_PX);
/// Default pixel width for each data column; overridden per-column via
/// `SpreadsheetState::column_widths`. Using a fixed width (rather than
/// FillPortion) lets the table overflow its container and become scrollable.
const COL_WIDTH: f32 = 140.0;
const COL_WIDTH_MIN: f32 = 40.0;
const COL_WIDTH_MAX: f32 = 600.0;
/// Pixel width of the draggable separator between column headers.
#[cfg(not(feature = "table_widget"))]
const RESIZE_HANDLE_WIDTH: f32 = 5.0;

/// How a global (filter-bar) query affects the row listing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GlobalFilterMode {
    /// Hide rows that do not match — classic Excel AutoFilter behaviour.
    #[default]
    FilterOut,
    /// Show every row, but tint the matching ones and let the user step
    /// through them with prev/next (Ctrl+G style).
    Highlight,
}

/// A single option in the column filter dropdown with value and row count.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColumnFilterOption {
    pub value: String,
    pub count: usize,
}

#[derive(Debug, Clone)]
pub struct SpreadsheetState {
    pub active: bool,
    pub sort_column: Option<usize>,
    pub sort_ascending: bool,

    // ── Global filter ──────────────────────────────────────────────────────
    pub filter_query: String,
    pub filter_mode: GlobalFilterMode,
    /// `orig_idx` for every row that matches the current query (populated
    /// only in `Highlight` mode). Stored in *catalog* order so prev/next
    /// navigation is stable.
    pub highlighted_indices: Vec<usize>,
    /// Position within `highlighted_indices` of the currently-focused match.
    pub current_highlight_pos: Option<usize>,
    /// Indices (into the catalog, i.e. `orig_idx`) that survive the filter.
    /// In `FilterOut` mode these are the only visible rows; in `Highlight`
    /// mode this always contains every row.
    pub filtered_indices: Vec<usize>,

    // ── Selection ─────────────────────────────────────────────────────────
    /// Index into the *catalog* (not into `filtered_indices`) of the row the
    /// user has selected. Storing the original index keeps the selection
    /// stable across filter and sort changes — the same record stays selected
    /// even if its position in the visible list moves, or it is filtered out
    /// and later filtered back in. Use `selected_filtered_idx()` to translate
    /// to a visible-row index for scrolling and styling.
    pub selected_orig: Option<usize>,

    // ── Panes / chrome ─────────────────────────────────────────────────────
    pub show_inspector: bool,
    pub pane_state: Option<pane_grid::State<SpreadsheetPaneContent>>,
    pub is_loading: bool,

    /// id of the filter `text_input`, exposed so future keyboard bindings can
    /// focus it via `text_input::focus(...)`.
    pub filter_input_id: iced::advanced::widget::Id,
    /// Ids of the two scrollables used to implement the sticky header: the
    /// body's horizontal offset is mirrored onto the header every time the
    /// body scrolls.
    pub body_scroll_id: iced::advanced::widget::Id,
    pub header_scroll_id: iced::advanced::widget::Id,

    // ── Column resizing ────────────────────────────────────────────────────
    /// Per-column width overrides. Columns absent from the map use
    /// `COL_WIDTH` as their default width.
    pub column_widths: HashMap<usize, f32>,
    /// Active drag state for the column the user is currently resizing.
    pub resizing_column: Option<ColumnDragState>,
    /// Tracks the last `StartResizeColumn` press `(col, time)` so that a
    /// second rapid press on the same column is promoted to an auto-size
    /// instead of starting a drag.  Required because the outer drag-tracking
    /// `mouse_area` intercepts the second click before it can reach the inner
    /// handle's `on_double_click`.
    pub last_resize_press: Option<(usize, std::time::Instant)>,

    // ── Scroll / viewport ─────────────────────────────────────────────────
    /// Absolute horizontal scroll offset, preserved when issuing scroll-to-row
    /// commands so the horizontal position isn't accidentally reset.
    pub horizontal_scroll_offset: f32,
    /// Absolute vertical scroll offset. Updated from `BodyScrolled` and used
    /// by the virtual-row renderer to determine which rows are in the viewport.
    pub vertical_scroll_offset: f32,
    /// Height of the body scrollable's visible area. Updated from `BodyScrolled`.
    pub viewport_height: f32,

    // ── Lazy-row cache invalidation ────────────────────────────────────────
    /// Bumped whenever any column width changes. Included in each row's lazy
    /// key so all rows rebuild on resize without an expensive full hash.
    pub col_widths_gen: u32,
    /// Pre-computed hash for each row's field values. Updated when catalog loads
    /// so the lazy key doesn't recompute on every render.
    pub row_hashes: Vec<u64>,
    /// Pre-computed display strings for all rows. Each entry is a `Vec<String>` of
    /// raw field values for display (one per column). Computed once on catalog load.
    /// Per-cell width-aware truncation is applied lazily inside the row renderer
    /// — the lazy widget caches the result by row key, so a string is shaped at
    /// most once per (row, col_widths_gen).
    pub display_cache: Vec<Vec<String>>,
    /// Pre-computed validation status for all rows. Each entry is a `Vec<bool>` where
    /// true = validation error. Computed once on catalog load.
    pub validation_cache: Vec<Vec<bool>>,

    // ── Scroll throttling ─────────────────────────────────────────────────
    /// Timestamp of the last scroll update we processed. Used to throttle
    /// frequent BodyScrolled messages during fast scrolling.
    pub last_scroll_update: Option<std::time::Instant>,
    /// Smoothed vertical scroll velocity in pixels per second (signed —
    /// positive when scrolling down). Updated from `record_scroll`. Used by
    /// the virtual-row renderer to widen the rendered range during fling
    /// scrolls so chunk boundaries are crossed with more rows pre-warmed.
    pub scroll_velocity_y: f32,

    // ── Column quick-filter ────────────────────────────────────────────────
    /// Per-column exact-match filters. Key = column index, value = set of selected values.
    pub column_filters: HashMap<usize, std::collections::HashSet<String>>,
    /// Which column's quick-filter dropdown is currently open (`None` = closed).
    pub active_column_filter: Option<usize>,
    /// Pre-computed unique values with counts for the open column filter dropdown.
    pub column_filter_options: Vec<ColumnFilterOption>,
    /// Search query for filtering the column filter dropdown.
    pub column_filter_search: String,

    // ── Inspector textarea state ───────────────────────────────────────────

    // ── Inspector textarea state ───────────────────────────────────────────
    /// One `text_editor::Content` per TextArea field of the currently-inspected
    /// record, keyed by field name. Populated on `SelectRow`, cleared on
    /// row change. Allows the cursor / selection to survive re-renders.
    pub inspector_textarea_contents: HashMap<String, TextAreaContent>,

    /// Process-wide cache of pre-shaped paragraphs, keyed by
    /// `(text, size, max_width, font)`. Cloned into each visible cell's
    /// renderer to skip cosmic-text shaping after the first hit. Cheap to
    /// clone (Arc).
    pub paragraph_cache: ParagraphCache,
}

/// Transient state for an in-progress column-resize drag.
///
/// We track the cursor x in *table-relative* coordinates (i.e. the x reported
/// by the outer wrapper `MouseArea`). Because the wrapper's bounds don't
/// shift while the user drags, the x value is stable and lets us compute an
/// accurate delta-from-start even when the drag spans hundreds of pixels.
#[derive(Debug, Clone, Copy)]
pub struct ColumnDragState {
    /// Index (into `R::field_descriptors()`) of the column being resized.
    pub col: usize,
    /// Width of the column at drag start — the baseline we add the drag
    /// delta to when computing the new width.
    pub anchor_width: f32,
    /// Table-relative x of the cursor at the first `on_move` after
    /// `on_press`. `None` until the first movement event arrives.
    pub anchor_cursor_x: Option<f32>,
}

#[derive(Debug, Clone)]
pub enum SpreadsheetPaneContent {
    Table,
    Inspector,
}

impl Default for SpreadsheetState {
    fn default() -> Self {
        Self {
            active: false,
            sort_column: None,
            sort_ascending: true,
            filter_query: String::new(),
            filter_mode: GlobalFilterMode::default(),
            highlighted_indices: Vec::new(),
            current_highlight_pos: None,
            filtered_indices: Vec::new(),
            selected_orig: None,
            show_inspector: false,
            pane_state: None,
            is_loading: false,
            filter_input_id: iced::advanced::widget::Id::unique(),
            body_scroll_id: iced::advanced::widget::Id::unique(),
            header_scroll_id: iced::advanced::widget::Id::unique(),
            column_widths: HashMap::new(),
            resizing_column: None,
            last_resize_press: None,
            horizontal_scroll_offset: 0.0,
            vertical_scroll_offset: 0.0,
            viewport_height: 400.0,
            col_widths_gen: 0,
            row_hashes: Vec::new(),
            display_cache: Vec::new(),
            validation_cache: Vec::new(),
            column_filters: HashMap::new(),
            active_column_filter: None,
            column_filter_options: Vec::new(),
            column_filter_search: String::new(),
            inspector_textarea_contents: HashMap::new(),
            last_scroll_update: None,
            scroll_velocity_y: 0.0,
            paragraph_cache: ParagraphCache::default(),
        }
    }
}

impl SpreadsheetState {
    pub const SCROLL_THROTTLE_MS: u64 = 16;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_sort(&mut self, col: usize) {
        if self.sort_column == Some(col) {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = Some(col);
            self.sort_ascending = true;
        }
    }

    /// Rebuild `filtered_indices` / `highlighted_indices` from the current
    /// `filter_query`, `filter_mode`, and `column_filters`.
    pub fn apply_filter<R: EditableRecord>(&mut self, catalog: &[R]) {
        self.highlighted_indices.clear();
        self.current_highlight_pos = None;

        let descriptors = R::field_descriptors();
        let has_query = !self.filter_query.is_empty();
        let has_col = !self.column_filters.is_empty();

        // Column filters always hard-filter rows regardless of mode.
        let col_matches = |record: &R| -> bool {
            for (&col, selected_values) in &self.column_filters {
                if let Some(desc) = descriptors.get(col) {
                    let value = record.get_field(desc.name);
                    if !selected_values.is_empty() && !selected_values.contains(&value) {
                        return false;
                    }
                }
            }
            true
        };

        if !has_query && !has_col {
            self.filtered_indices = (0..catalog.len()).collect();
            return;
        }

        let query = self.filter_query.to_lowercase();
        let matches_query = |record: &R| Self::record_matches(record, &query);

        match self.filter_mode {
            GlobalFilterMode::FilterOut => {
                self.filtered_indices.clear();
                for (idx, record) in catalog.iter().enumerate() {
                    let col_ok = !has_col || col_matches(record);
                    let q_ok = !has_query || matches_query(record);
                    if col_ok && q_ok {
                        self.filtered_indices.push(idx);
                    }
                }
            }
            GlobalFilterMode::Highlight => {
                // Column filters hard-filter; global query only highlights.
                self.filtered_indices.clear();
                for (idx, record) in catalog.iter().enumerate() {
                    if !has_col || col_matches(record) {
                        self.filtered_indices.push(idx);
                        if has_query && matches_query(record) {
                            self.highlighted_indices.push(idx);
                        }
                    }
                }
                if !self.highlighted_indices.is_empty() {
                    self.current_highlight_pos = Some(0);
                }
            }
        }
    }

    /// Returns `true` if any field or list label contains `query_lower`
    /// (case-insensitive ASCII comparison, zero allocations for ASCII data).
    fn record_matches<R: EditableRecord>(record: &R, query_lower: &str) -> bool {
        if query_lower.is_empty() {
            return true;
        }
        for desc in R::field_descriptors() {
            let value = record.get_field(desc.name);
            if contains_ignore_ascii_case(&value, query_lower) {
                return true;
            }
        }
        contains_ignore_ascii_case(&record.list_label(), query_lower)
    }

    /// Clear the text query but keep any active column filters.
    pub fn clear_filter<R: EditableRecord>(&mut self, catalog: &[R]) {
        self.filter_query.clear();
        self.apply_filter(catalog);
    }

    /// Remove the quick-filter for a single column and re-apply filters.
    pub fn clear_column_filter<R: EditableRecord>(&mut self, col: usize, catalog: &[R]) {
        self.column_filters.remove(&col);
        self.active_column_filter = None;
        self.apply_filter(catalog);
    }

    pub fn set_filter_mode<R: EditableRecord>(&mut self, mode: GlobalFilterMode, catalog: &[R]) {
        if self.filter_mode == mode {
            return;
        }
        self.filter_mode = mode;
        self.apply_filter(catalog);
    }

    /// Move the current-highlight cursor to the next / previous match, with
    /// wrap-around. Safe to call when no highlights are present.
    pub fn navigate_next_highlight(&mut self) {
        if self.highlighted_indices.is_empty() {
            self.current_highlight_pos = None;
            return;
        }
        let len = self.highlighted_indices.len();
        self.current_highlight_pos = Some(match self.current_highlight_pos {
            Some(pos) => (pos + 1) % len,
            None => 0,
        });
    }

    pub fn navigate_prev_highlight(&mut self) {
        if self.highlighted_indices.is_empty() {
            self.current_highlight_pos = None;
            return;
        }
        let len = self.highlighted_indices.len();
        self.current_highlight_pos = Some(match self.current_highlight_pos {
            Some(0) | None => len - 1,
            Some(pos) => pos - 1,
        });
    }

    /// `orig_idx` of the row the user is currently navigated to via
    /// `Navigate{Next,Prev}Highlight`. Returns `None` when not in highlight
    /// mode or when there are no matches.
    pub fn current_highlight_orig_idx(&self) -> Option<usize> {
        self.current_highlight_pos
            .and_then(|p| self.highlighted_indices.get(p).copied())
    }

    pub fn apply_sort<R: EditableRecord>(&mut self, catalog: &[R]) {
        let Some(col) = self.sort_column else { return };
        let descriptors = R::field_descriptors();
        let Some(desc) = descriptors.get(col) else {
            return;
        };
        let field = desc.name;
        let ascending = self.sort_ascending;

        // Try a numeric comparison first; fall back to lexicographic so the
        // sort behaves intuitively for mixed text/integer columns.
        self.filtered_indices.sort_by(|&a, &b| {
            let va = catalog[a].get_field(field);
            let vb = catalog[b].get_field(field);
            let cmp = match (va.parse::<f64>(), vb.parse::<f64>()) {
                (Ok(na), Ok(nb)) => na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal),
                _ => va.cmp(&vb),
            };
            if ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });
    }

    /// Translate the stored `selected_orig` to a position in the current
    /// `filtered_indices`. Returns `None` when nothing is selected, or when
    /// the selected row is currently filtered out.
    pub fn selected_filtered_idx(&self) -> Option<usize> {
        let orig = self.selected_orig?;
        self.filtered_indices.iter().position(|&i| i == orig)
    }

    /// Set the selection from a *filtered* index without changing inspector
    /// visibility. Used by keyboard navigation and highlight-step messages
    /// where the inspector should stay in whatever state the user left it.
    pub fn set_selection(&mut self, filtered_idx: usize) {
        self.selected_orig = self.filtered_indices.get(filtered_idx).copied();
    }

    /// Set the selection from a *filtered* index AND open the inspector. Used
    /// by explicit row clicks where the user is asking to inspect the row.
    pub fn select_row(&mut self, filtered_idx: usize) {
        self.set_selection(filtered_idx);
        self.show_inspector = true;
    }

    /// Current display width (in px) of a data column, honouring any
    /// per-column override stored in `column_widths`.
    pub fn column_width(&self, col: usize) -> f32 {
        self.column_widths.get(&col).copied().unwrap_or(COL_WIDTH)
    }

    /// Total content width of the table (ID column + all data columns).
    pub fn total_table_width(&self, n_cols: usize) -> f32 {
        let mut w = ID_COL_WIDTH_PX;
        for col in 0..n_cols {
            w += self.column_width(col);
        }
        w
    }

    /// Called on mouse-down on a column's resize handle.
    ///
    /// Returns `true` when the press is recognised as a double-press (second
    /// press on the same column within 400 ms): the caller should auto-size the
    /// column and skip starting a drag.  Returns `false` for a normal single
    /// press, in which case the drag is started and the caller does nothing
    /// extra.
    ///
    /// We detect double-press here rather than relying on `on_double_click`
    /// because the first press makes `resizing_column = Some(…)` which causes
    /// iced to wrap the table in an outer `mouse_area`; that outer layer
    /// intercepts the second click before it can reach the inner handle's
    /// `on_double_click`.
    pub fn try_begin_column_resize(&mut self, col: usize) -> bool {
        let double_press_threshold = std::time::Duration::from_millis(400);
        if let Some((last_col, last_time)) = self.last_resize_press {
            if last_col == col && last_time.elapsed() < double_press_threshold {
                self.last_resize_press = None;
                return true; // caller should auto-size
            }
        }
        self.last_resize_press = Some((col, std::time::Instant::now()));
        let anchor_width = self.column_width(col);
        self.resizing_column = Some(ColumnDragState {
            col,
            anchor_width,
            anchor_cursor_x: None,
        });
        false
    }

    /// Called on every mouse move while a resize handle is pressed. Uses the
    /// cursor delta since drag start to compute a new width, clamped to
    /// `[COL_WIDTH_MIN, COL_WIDTH_MAX]`.
    ///
    /// `col_widths_gen` is intentionally not bumped here — doing so would
    /// invalidate the lazy cache for every visible row on every mousemove,
    /// which is the dominant cost of a column drag. The header reads
    /// `column_widths` directly so it still resizes live; data rows stay
    /// frozen to their cached widths until `end_column_resize` bumps the gen.
    pub fn update_column_resize(&mut self, cursor_x: f32) {
        let Some(ref mut drag) = self.resizing_column else {
            return;
        };
        let anchor_x = match drag.anchor_cursor_x {
            Some(x) => x,
            None => {
                drag.anchor_cursor_x = Some(cursor_x);
                return;
            }
        };
        let delta = cursor_x - anchor_x;
        let new_width = (drag.anchor_width + delta).clamp(COL_WIDTH_MIN, COL_WIDTH_MAX);
        self.column_widths.insert(drag.col, new_width);
    }

    pub fn end_column_resize(&mut self) {
        self.resizing_column = None;
        self.col_widths_gen = self.col_widths_gen.wrapping_add(1);
    }

    /// Double-click auto-size: measure the longest cell value in the visible
    /// rows plus the header label and set the column width accordingly, clamped
    /// to `[COL_WIDTH_MIN, COL_WIDTH_MAX]`.
    ///
    /// Uses the same 7 px/char heuristic already applied for truncation so the
    /// result is consistent with what the user sees in the cells.  A small pad
    /// (16 px) is added for cell padding on both sides.
    pub fn auto_size_column<R: EditableRecord>(&mut self, col: usize, catalog: &[R]) {
        let descriptors = R::field_descriptors();
        let Some(desc) = descriptors.get(col) else {
            return;
        };

        // Seed with the header label length.
        let mut max_chars = desc.label.chars().count();

        for &orig_idx in &self.filtered_indices {
            if let Some(record) = catalog.get(orig_idx) {
                let chars = record.get_field(desc.name).chars().count();
                if chars > max_chars {
                    max_chars = chars;
                }
            }
        }

        let width = ((max_chars as f32) * 7.0 + 16.0).clamp(COL_WIDTH_MIN, COL_WIDTH_MAX);
        self.column_widths.insert(col, width);
        self.col_widths_gen = self.col_widths_gen.wrapping_add(1);
    }

    /// Move selection one row up; returns the new `filtered_idx` or `None`
    /// when there is nothing to navigate.
    pub fn navigate_up(&mut self) -> Option<usize> {
        let total = self.filtered_indices.len();
        if total == 0 {
            return None;
        }
        let new_idx = match self.selected_filtered_idx() {
            Some(idx) if idx > 0 => idx - 1,
            Some(idx) => idx,
            None => total.saturating_sub(1),
        };
        self.select_row(new_idx);
        Some(new_idx)
    }

    /// Move selection one row down; returns the new `filtered_idx`.
    pub fn navigate_down(&mut self) -> Option<usize> {
        let total = self.filtered_indices.len();
        if total == 0 {
            return None;
        }
        let new_idx = match self.selected_filtered_idx() {
            Some(idx) if idx + 1 < total => idx + 1,
            Some(idx) => idx,
            None => 0,
        };
        self.select_row(new_idx);
        Some(new_idx)
    }

    /// Jump to the first visible row.
    pub fn navigate_top(&mut self) -> Option<usize> {
        if self.filtered_indices.is_empty() {
            return None;
        }
        self.select_row(0);
        Some(0)
    }

    /// Jump to the last visible row.
    pub fn navigate_bottom(&mut self) -> Option<usize> {
        let total = self.filtered_indices.len();
        if total == 0 {
            return None;
        }
        let last = total - 1;
        self.select_row(last);
        Some(last)
    }

    /// Compute the body-scrollable Y offset that centers `filtered_idx` in
    /// the visible viewport. Used by jump-style navigation (highlight cursor,
    /// bottom).
    pub fn scroll_y_for_row(&self, filtered_idx: usize) -> f32 {
        ((filtered_idx as f32 + 0.5) * ROW_HEIGHT - self.viewport_height / 2.0).max(0.0)
    }

    /// Minimal scroll offset to keep `filtered_idx` visible. If the row is
    /// already inside the current viewport the existing offset is returned —
    /// arrow-key navigation across already-visible rows must not jerk the
    /// viewport. Otherwise the row is brought just-on-screen at the
    /// nearest edge.
    pub fn ensure_row_visible_y(&self, filtered_idx: usize) -> f32 {
        let row_top = filtered_idx as f32 * ROW_HEIGHT;
        let row_bottom = row_top + ROW_HEIGHT;
        let cur = self.vertical_scroll_offset;
        let cur_bottom = cur + self.viewport_height;
        if row_top < cur {
            row_top
        } else if row_bottom > cur_bottom {
            (row_bottom - self.viewport_height).max(0.0)
        } else {
            cur
        }
    }

    /// Record a programmatic scroll target. Under the lazy/scrollable path
    /// this is redundant — the scrollable will emit `BodyScrolled` once it
    /// applies the operation, which calls `record_scroll` with the same
    /// values. Under the `table_widget` path, however, the custom widget
    /// reads these fields on its next layout to snap its internal offset,
    /// so this mutation is the *only* way programmatic navigation moves the
    /// viewport.
    pub fn record_target_offset(&mut self, x: f32, y: f32) {
        self.horizontal_scroll_offset = x;
        self.vertical_scroll_offset = y;
    }

    /// Record a scroll event from the body scrollable. Updates the cached
    /// offset and viewport plus a smoothed scroll velocity used to size the
    /// virtual-row overscan adaptively.
    ///
    /// Returns the EWMA-smoothed signed velocity in px/s (positive = downward).
    pub fn record_scroll(&mut self, offset_x: f32, offset_y: f32, viewport_height: f32) -> f32 {
        use std::time::Instant;
        let now = Instant::now();
        if let Some(last_t) = self.last_scroll_update {
            let dt = now.duration_since(last_t).as_secs_f32();
            // Ignore stale gaps — if the user paused for more than 250 ms,
            // velocity is no longer meaningful, so reset rather than carry
            // stale momentum into the next fling.
            if dt > 0.0 && dt < 0.25 {
                let inst_v = (offset_y - self.vertical_scroll_offset) / dt;
                self.scroll_velocity_y = 0.4 * inst_v + 0.6 * self.scroll_velocity_y;
            } else {
                self.scroll_velocity_y = 0.0;
            }
        }
        self.last_scroll_update = Some(now);
        self.horizontal_scroll_offset = offset_x;
        self.vertical_scroll_offset = offset_y;
        self.viewport_height = viewport_height;
        self.scroll_velocity_y
    }

    /// Compute the current overscan-row count from `scroll_velocity_y`.
    /// During fast scrolling the rendered range is widened so that chunk
    /// boundaries coincide with rows that have already been laid out, hiding
    /// the cosmic-text shaping cost.
    pub fn overscan_rows(&self) -> usize {
        let velocity_rows = (self.scroll_velocity_y.abs() / ROW_HEIGHT) as usize;
        let extra = (velocity_rows as f32 / OVERSCAN_VELOCITY_DIVISOR) as usize;
        (OVERSCAN_ROWS_BASE + extra).min(OVERSCAN_ROWS_MAX)
    }

    pub fn toggle_inspector(&mut self) {
        self.show_inspector = !self.show_inspector;
    }

    pub fn toggle_active(&mut self) {
        self.active = !self.active;
    }

    pub fn init_filter<R: EditableRecord>(&mut self, catalog: &[R]) {
        self.filtered_indices = (0..catalog.len()).collect();
    }

    /// Compute and cache row hashes, raw field values, and validation status.
    /// Synchronous — blocks the UI thread while running. Prefer
    /// `compute_caches` + `install_caches` from a `spawn_blocking` task for
    /// large catalogs (see `standard::handle::CatalogLoaded`).
    pub fn compute_all_caches<R: EditableRecord>(&mut self, catalog: &[R]) {
        self.install_caches(compute_caches(catalog));
    }

    /// Install pre-computed caches. The expensive work happens in
    /// `compute_caches` — this method is the cheap mutation that integrates
    /// the result back into spreadsheet state on the UI thread.
    pub fn install_caches(&mut self, data: ComputedCaches) {
        self.row_hashes = data.row_hashes;
        self.display_cache = data.display_cache;
        self.validation_cache = data.validation_cache;
        self.col_widths_gen = self.col_widths_gen.wrapping_add(1);
    }

    /// Get raw display string for a cell (cloned). Width-aware truncation
    /// happens at render time via [`truncate_for_width`].
    pub fn get_display(&self, orig_idx: usize, col: usize) -> String {
        self.display_cache
            .get(orig_idx)
            .and_then(|row| row.get(col))
            .cloned()
            .unwrap_or_default()
    }

    /// Get reference to cached display string (avoids clone if caller only needs to borrow).
    pub fn get_display_ref(&self, orig_idx: usize, col: usize) -> Option<&String> {
        self.display_cache
            .get(orig_idx)
            .and_then(|row| row.get(col))
    }

    /// Get the cached hash for a row by original index.
    pub fn row_hash(&self, orig_idx: usize) -> u64 {
        self.row_hashes.get(orig_idx).copied().unwrap_or(0)
    }

    /// Export the *currently visible* rows (respecting filter + sort) to CSV.
    /// Returns the raw bytes ready to be written to disk.
    pub fn to_csv_bytes<R: EditableRecord>(&self, catalog: &[R]) -> Result<Vec<u8>, String> {
        let mut wtr = csv::Writer::from_writer(Vec::new());
        let descriptors = R::field_descriptors();
        let header: Vec<&'static str> = descriptors.iter().map(|d| d.label).collect();
        wtr.write_record(&header).map_err(|e| e.to_string())?;

        for &orig_idx in &self.filtered_indices {
            let Some(record) = catalog.get(orig_idx) else {
                continue;
            };
            let row: Vec<String> = descriptors
                .iter()
                .map(|d| record.get_field(d.name))
                .collect();
            wtr.write_record(&row).map_err(|e| e.to_string())?;
        }

        wtr.into_inner().map_err(|e| e.into_error().to_string())
    }

    /// Creates the pane state with only the Table pane.
    /// Inspector is added on demand when a row is first selected.
    pub fn init_pane_state(&mut self) {
        let (state, _first) = pane_grid::State::new(SpreadsheetPaneContent::Table);
        self.pane_state = Some(state);
    }

    /// Adds or removes the inspector pane to match `show_inspector`.
    /// When adding, the inspector gets ~30% of width. Lazily initialises
    /// `pane_state` so the inspector can be opened without a prior explicit
    /// `ToggleActive` / catalog-load event.
    pub fn ensure_inspector_pane(&mut self) {
        if self.pane_state.is_none() {
            self.init_pane_state();
        }
        let Some(ref mut ps) = self.pane_state else {
            return;
        };

        if self.show_inspector {
            let has_inspector = ps
                .iter()
                .any(|(_, c)| matches!(c, SpreadsheetPaneContent::Inspector));
            if !has_inspector {
                let table_pane = ps
                    .iter()
                    .find_map(|(p, c)| matches!(c, SpreadsheetPaneContent::Table).then_some(*p));
                if let Some(pane) = table_pane {
                    if let Some((_, split)) = ps.split(
                        pane_grid::Axis::Vertical,
                        pane,
                        SpreadsheetPaneContent::Inspector,
                    ) {
                        // Table takes 70%, Inspector takes 30%
                        ps.resize(split, 0.70);
                    }
                }
            }
        } else {
            let inspector_panes: Vec<Pane> = ps
                .iter()
                .filter_map(|(p, c)| matches!(c, SpreadsheetPaneContent::Inspector).then_some(*p))
                .collect();
            for pane in inspector_panes {
                if ps.len() > 1 {
                    ps.close(pane);
                }
            }
        }
    }
}

/// Pre-computed display data for an entire catalog. Produced by
/// `compute_caches` (a pure function safe to run from a worker thread) and
/// consumed by `SpreadsheetState::install_caches` on the UI thread.
///
/// All `Vec`s are `catalog.len()` long and indexed by `orig_idx`. Splitting
/// compute from install lets a slow ~hundred-millisecond build run inside
/// `tokio::task::spawn_blocking` without freezing the UI.
#[derive(Debug, Clone, Default)]
pub struct ComputedCaches {
    pub row_hashes: Vec<u64>,
    pub display_cache: Vec<Vec<String>>,
    pub validation_cache: Vec<Vec<bool>>,
}

/// Pure cache-build function: walks every record, materialises display
/// strings, hashes them, and runs per-field validation. Allocations only —
/// no UI state touched, no global state read — so it is safe to run from a
/// background thread.
pub fn compute_caches<R: EditableRecord>(catalog: &[R]) -> ComputedCaches {
    use std::hash::{Hash, Hasher};
    let descriptors = R::field_descriptors();
    let num_cols = descriptors.len();
    let mut row_hashes = Vec::with_capacity(catalog.len());
    let mut display_cache = Vec::with_capacity(catalog.len());
    let mut validation_cache = Vec::with_capacity(catalog.len());
    for record in catalog.iter() {
        let values: Vec<String> = (0..num_cols)
            .map(|j| record.get_field(descriptors[j].name))
            .collect();
        let mut h = std::collections::hash_map::DefaultHasher::new();
        values.hash(&mut h);
        row_hashes.push(h.finish());
        let validation: Vec<bool> = (0..num_cols)
            .map(|j| {
                record
                    .validate_field(descriptors[j].name, &values[j])
                    .is_some()
            })
            .collect();
        display_cache.push(values);
        validation_cache.push(validation);
    }
    ComputedCaches {
        row_hashes,
        display_cache,
        validation_cache,
    }
}

/// Width-aware truncation used by the row renderer. Approximates an 7 px/char
/// budget — same heuristic that drives `auto_size_column` — and appends `…`
/// when the value overflows. Cheap enough to invoke per visible cell on each
/// lazy rebuild (i.e. when the row's `col_widths_gen` changes).
pub fn truncate_for_width(value: &str, col_width: f32) -> String {
    let char_budget = ((col_width / 7.0) as usize).max(8);
    if value.len() <= char_budget && value.is_ascii() {
        return value.to_string();
    }
    if value.chars().count() <= char_budget {
        return value.to_string();
    }
    let mut truncated = String::with_capacity(char_budget + 3);
    for (i, c) in value.chars().enumerate() {
        if i >= char_budget {
            truncated.push('…');
            break;
        }
        truncated.push(c);
    }
    truncated
}

#[derive(Debug, Clone)]
pub enum SpreadsheetMessage {
    ToggleActive,
    SortColumn(usize),
    FilterChanged(String),
    ClearFilter,
    SetFilterMode(GlobalFilterMode),
    NavigateNextHighlight,
    NavigatePrevHighlight,
    /// Arrow-key row navigation (keyboard shortcuts wired at the app level).
    NavigateUp,
    NavigateDown,
    NavigateTop,
    NavigateBottom,
    SelectRow(usize),
    CancelEdit,
    ToggleInspector,
    CloseInspector,
    ExportCsv,
    /// Emitted after the async CSV save completes; payload is `Ok(path)` or
    /// `Err(msg)`. Per-editor handlers forward it into the status bar.
    CsvExported(Result<std::path::PathBuf, String>),
    /// Fired by the body scrollable on every scroll delta. Carries the
    /// absolute offset (to mirror horizontal scrolling onto the sticky header
    /// and drive virtual rendering) and the visible viewport height.
    BodyScrolled(iced::widget::scrollable::AbsoluteOffset, f32),

    // ── Column quick-filter ────────────────────────────────────────────────
    /// Toggle the quick-filter dropdown for `col` open/closed. The handler
    /// pre-computes the unique values list before opening.
    OpenColumnFilter(usize),
    /// Apply the picked unique-value filter for `col`.
    ApplyColumnFilter(usize, String),
    /// Remove the quick-filter for `col` and re-apply.
    ClearColumnFilter(usize),

    // ── Column resizing ────────────────────────────────────────────────────
    /// Mouse-down on a column's resize handle; `col` is the index into
    /// `R::field_descriptors()`.
    StartResizeColumn(usize),
    /// Fires on every pointer move while a resize handle is pressed.
    /// Carries the absolute cursor x, in logical px.
    ResizeColumnCursor(f32),
    /// Mouse-up — commit the current width and exit drag mode.
    EndResizeColumn,
    /// Double-click on a resize handle: auto-size `col` to its longest cell value.
    ResetColumnWidth(usize),

    // ── Inspector textarea editing ─────────────────────────────────────────
    /// Fired by a `text_editor` widget in the inspector panel.
    /// `(orig_idx, field_name, action)` — the handler performs the action on
    /// the stored `TextAreaContent` and syncs the string value back to the
    /// record via `FieldChanged`.
    TextAreaChanged(usize, String, text_editor::Action),
    /// Fired by `text_input` / `pick_list` widgets in the inspector panel.
    /// Routes through the macro so that `compute_all_caches` is called after.
    InspectorFieldChanged(usize, String, String),

    // ── Async cache loading ────────────────────────────────────────────────
    /// Result of an off-thread `compute_caches` job. The handler installs the
    /// caches on the UI thread and clears `is_loading`. Dispatched only by
    /// flows that schedule the compute through `spawn_blocking`; synchronous
    /// `compute_all_caches` callers do not emit this.
    CachesComputed(ComputedCaches),

    // ── Context menu ─────────────────────────────────────────────────────
    /// Apply a quick filter from right-click context menu.
    QuickFilter(usize, String),

    // ── Enhanced column filter ──────────────────────────────────────────
    /// Update search query within column filter dropdown.
    ColumnFilterSearch(String),
    /// Toggle a value in the column filter (for multi-select).
    ToggleColumnFilterValue(usize, String),
    /// Select all values in column filter.
    SelectAllColumnFilter(usize),
    /// Clear all selected values in column filter.
    ClearAllColumnFilter(usize),
}

/// Allocation-free case-insensitive substring search for pure-ASCII content.
/// Falls back to a `to_lowercase` allocation for non-ASCII haystacks.
fn contains_ignore_ascii_case(haystack: &str, needle_lower: &str) -> bool {
    if needle_lower.is_empty() {
        return true;
    }
    let hb = haystack.as_bytes();
    let nb = needle_lower.as_bytes();
    if hb.len() < nb.len() {
        return false;
    }
    // Fast path: all characters are ASCII — no allocation.
    if haystack.is_ascii() {
        return hb
            .windows(nb.len())
            .any(|w| w.iter().zip(nb).all(|(&h, &n)| h.to_ascii_lowercase() == n));
    }
    // Slow path: Unicode — fall back to allocating a lowercase copy.
    haystack.to_lowercase().contains(needle_lower)
}

// ===========================================================================
// View
// ===========================================================================

#[allow(clippy::too_many_arguments)]
pub fn view_spreadsheet<'a, R: EditableRecord>(
    editor: &'a GenericEditorState<R>,
    spreadsheet: &'a SpreadsheetState,
    scan_msg: Message,
    save_msg: Message,
    _select_msg: fn(usize) -> Message,
    field_changed_msg: fn(usize, String, String) -> Message,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
    lookups: &'a HashMap<String, Vec<(String, String)>>,
    pane_resized_msg: fn(pane_grid::ResizeEvent) -> Message,
    pane_clicked_msg: fn(Pane) -> Message,
) -> Element<'a, Message> {
    let descriptors = R::field_descriptors();

    let status_row = build_status_bar(editor, spreadsheet, save_msg, spreadsheet_msg);
    let filter_bar = build_filter_bar(editor, spreadsheet, scan_msg, spreadsheet_msg);

    let catalog = editor.catalog.as_ref();

    let Some(ref pane_state) = spreadsheet.pane_state else {
        let table = build_table_content(descriptors, catalog, spreadsheet, spreadsheet_msg);
        return column![
            horizontal_rule(1),
            filter_bar,
            horizontal_rule(1),
            table,
            status_row,
        ]
        .spacing(0)
        .height(Fill)
        .into();
    };

    let pane_grid = pane_grid::PaneGrid::new(pane_state, |_id, pane_content, _is_maximized| {
        let content: Element<Message> = match pane_content {
            SpreadsheetPaneContent::Table => {
                build_table_content(descriptors, catalog, spreadsheet, spreadsheet_msg)
            }
            SpreadsheetPaneContent::Inspector => build_inspector_panel(
                editor,
                spreadsheet,
                lookups,
                field_changed_msg,
                spreadsheet_msg,
            ),
        };
        pane_grid::Content::new(content)
    })
    .on_click(pane_clicked_msg)
    .on_resize(4, pane_resized_msg)
    .height(Length::Fill)
    .width(Length::Fill);

    column![
        horizontal_rule(1),
        filter_bar,
        horizontal_rule(1),
        pane_grid,
        status_row,
    ]
    .spacing(0)
    .height(Fill)
    .into()
}

// ───────────────────────────────────────────────────────────────────────────
// Filter bar — mode toggle, query, clear, navigation, export
// ───────────────────────────────────────────────────────────────────────────

fn build_filter_bar<'a, R: EditableRecord>(
    editor: &'a GenericEditorState<R>,
    spreadsheet: &'a SpreadsheetState,
    scan_msg: Message,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> Element<'a, Message> {
    let total = editor.catalog.as_ref().map(|c| c.len()).unwrap_or(0);
    let visible = spreadsheet.filtered_indices.len();
    let highlight_count = spreadsheet.highlighted_indices.len();

    // Filter mode toggle — two mini-buttons that look like a segmented control.
    let mode_btn = |label: &'static str, mode: GlobalFilterMode| {
        let active = spreadsheet.filter_mode == mode;
        button(text(label).size(11))
            .padding([3, 8])
            .on_press(spreadsheet_msg(SpreadsheetMessage::SetFilterMode(mode)))
            .style(if active {
                style::filter_mode_active
            } else {
                style::filter_mode_inactive
            })
    };

    let mode_toggle = row![
        mode_btn("Filter", GlobalFilterMode::FilterOut),
        mode_btn("Highlight", GlobalFilterMode::Highlight),
    ]
    .spacing(2);

    // Filter input + clear button.
    let filter_input = text_input("Search records...", &spreadsheet.filter_query)
        .id(spreadsheet.filter_input_id.clone())
        .on_input(move |q| spreadsheet_msg(SpreadsheetMessage::FilterChanged(q)))
        .on_submit(spreadsheet_msg(SpreadsheetMessage::NavigateNextHighlight))
        .padding(6)
        .width(Length::FillPortion(2))
        .style(style::spreadsheet_filter_input);

    let clear_btn: Element<Message> = if spreadsheet.filter_query.is_empty() {
        Element::from(horizontal_space().width(Length::Fixed(0.0)))
    } else {
        button(text("×").size(14))
            .padding([0, 8])
            .on_press(spreadsheet_msg(SpreadsheetMessage::ClearFilter))
            .style(style::filter_clear_button)
            .into()
    };

    // Mode-dependent readout: counter or navigation pager.
    let status_area: Element<Message> = match spreadsheet.filter_mode {
        GlobalFilterMode::FilterOut => text(format!("{visible} of {total} rows"))
            .size(11)
            .style(style::filter_status_text)
            .into(),
        GlobalFilterMode::Highlight => {
            let current_label = spreadsheet
                .current_highlight_pos
                .map(|p| p + 1)
                .unwrap_or(0);

            let prev_btn = button(text("◀").size(10))
                .padding([2, 6])
                .on_press_maybe(
                    (highlight_count > 0)
                        .then(|| spreadsheet_msg(SpreadsheetMessage::NavigatePrevHighlight)),
                )
                .style(style::nav_button);

            let next_btn = button(text("▶").size(10))
                .padding([2, 6])
                .on_press_maybe(
                    (highlight_count > 0)
                        .then(|| spreadsheet_msg(SpreadsheetMessage::NavigateNextHighlight)),
                )
                .style(style::nav_button);

            let counter = if highlight_count == 0 {
                text("0 matches".to_string())
            } else {
                text(format!("{current_label} / {highlight_count}"))
            }
            .size(11)
            .style(style::filter_status_text);

            row![prev_btn, counter, next_btn]
                .spacing(6)
                .align_y(iced::Alignment::Center)
                .into()
        }
    };

    // Column quick-filter with search and multi-select (visible only when a column filter menu is open).
    let col_filter_area: Element<Message> = if let Some(col) = spreadsheet.active_column_filter {
        // Search input
        let search_input = text_input("Search...", &spreadsheet.column_filter_search)
            .on_input(move |q| spreadsheet_msg(SpreadsheetMessage::ColumnFilterSearch(q)))
            .padding(4)
            .width(Length::Fixed(140.0))
            .style(style::spreadsheet_filter_input);

        // Filter options based on search
        let search_lower = spreadsheet.column_filter_search.to_lowercase();
        let filtered_options: Vec<_> = spreadsheet
            .column_filter_options
            .iter()
            .filter(|opt| opt.value.to_lowercase().contains(&search_lower))
            .collect();

        // Build list of checkboxes with counts
        let current_filter = spreadsheet.column_filters.get(&col);
        let option_list: Vec<Element<Message>> = filtered_options
            .iter()
            .map(|opt| {
                let is_checked = current_filter
                    .map(|s| s.contains(&opt.value))
                    .unwrap_or(false);
                let label = if is_checked {
                    format!("✓ {} ({})", opt.value, opt.count)
                } else {
                    format!("  {} ({})", opt.value, opt.count)
                };
                button(text(label).size(10))
                    .on_press(spreadsheet_msg(
                        SpreadsheetMessage::ToggleColumnFilterValue(col, opt.value.clone()),
                    ))
                    .width(Length::Fill)
                    .style(if is_checked {
                        style::browse_button
                    } else {
                        style::browse_button
                    })
                    .into()
            })
            .collect();

        let scroll = scrollable(column(option_list).spacing(2))
            .height(Length::Fixed(150.0))
            .width(Length::Fixed(180.0));

        // Select All / Clear All buttons
        let select_all_btn = button(text("All").size(10))
            .on_press(spreadsheet_msg(SpreadsheetMessage::SelectAllColumnFilter(
                col,
            )))
            .style(style::browse_button);
        let clear_all_btn = button(text("None").size(10))
            .on_press(spreadsheet_msg(SpreadsheetMessage::ClearAllColumnFilter(
                col,
            )))
            .style(style::browse_button);
        let close_btn = button(text("✕").size(11))
            .on_press(spreadsheet_msg(SpreadsheetMessage::OpenColumnFilter(col)))
            .style(style::browse_button);

        let controls = row![select_all_btn, clear_all_btn, close_btn].spacing(4);

        container(
            column![
                row![
                    text("Filter:").size(11).style(style::subtle_text),
                    search_input,
                ],
                scroll,
                controls,
            ]
            .spacing(4),
        )
        .style(style::status_bar)
        .into()
    } else {
        horizontal_space().width(Length::Fixed(0.0)).into()
    };

    row![
        text("Filter:").size(12).style(style::subtle_text),
        mode_toggle,
        filter_input,
        clear_btn,
        col_filter_area,
        horizontal_space(),
        status_area,
        horizontal_space().width(12),
        button(text("CSV").size(11))
            .on_press(spreadsheet_msg(SpreadsheetMessage::ExportCsv))
            .style(style::export_button),
        button(text("Scan").size(11))
            .on_press(scan_msg)
            .style(style::browse_button),
    ]
    .padding([8, 12])
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

// ───────────────────────────────────────────────────────────────────────────
// Status bar — VIM-style mode indicator + save button + loading
// ───────────────────────────────────────────────────────────────────────────

fn build_status_bar<'a, R: EditableRecord>(
    editor: &'a GenericEditorState<R>,
    spreadsheet: &'a SpreadsheetState,
    save_msg: Message,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> Element<'a, Message> {
    let loading: Element<Message> = if spreadsheet.is_loading {
        container(
            column![
                text("Loading spreadsheet...")
                    .size(11)
                    .style(style::subtle_text),
                container(progress_bar(0.0..=100.0, 50.0))
                    .width(Fill)
                    .height(Length::Fixed(6.0)),
            ]
            .spacing(4)
            .width(Length::Fixed(160.0)),
        )
        .into()
    } else {
        Element::from(text(""))
    };

    container(
        row![
            text(&editor.status_msg).size(13).style(style::subtle_text),
            horizontal_space(),
            loading,
            horizontal_space().width(20),
            button(text("Inspector").size(11))
                .on_press(spreadsheet_msg(SpreadsheetMessage::ToggleInspector))
                .style(style::browse_button),
            button(text(R::save_button_label()).size(11))
                .on_press(save_msg)
                .style(style::commit_button),
        ]
        .padding([8, 20])
        .spacing(4)
        .align_y(iced::Alignment::Center),
    )
    .width(Fill)
    .style(style::status_bar)
    .into()
}

// ───────────────────────────────────────────────────────────────────────────
// Table content — header + data rows, wrapped in a two-axis scroll.
// ───────────────────────────────────────────────────────────────────────────

fn build_table_content<'a, R: EditableRecord>(
    descriptors: &'a [FieldDescriptor],
    catalog: Option<&'a Vec<R>>,
    spreadsheet: &'a SpreadsheetState,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> Element<'a, Message> {
    let Some(catalog) = catalog else {
        return container(
            text("No data loaded. Click Scan to load records.")
                .size(13)
                .style(style::subtle_text),
        )
        .width(Fill)
        .padding(20)
        .into();
    };

    // Total content width — the header and body scrollables both stretch to
    // this, which keeps column boundaries aligned between them.
    let total_width = spreadsheet.total_table_width(descriptors.len());

    let current_highlight_orig = spreadsheet.current_highlight_orig_idx();
    let is_highlight_mode = spreadsheet.filter_mode == GlobalFilterMode::Highlight;

    // Approach B: custom virtualised table widget. Behind a feature flag for
    // A/B comparison until we delete the lazy/column path.
    #[cfg(feature = "table_widget")]
    {
        let _ = (
            current_highlight_orig,
            is_highlight_mode,
            catalog,
            total_width,
        );
        return build_table_content_widget(descriptors, spreadsheet, spreadsheet_msg);
    }

    #[cfg(not(feature = "table_widget"))]
    {
        let header_row = build_header_row(descriptors, spreadsheet, spreadsheet_msg);
        build_table_content_lazy(
            descriptors,
            catalog,
            header_row,
            total_width,
            spreadsheet,
            current_highlight_orig,
            is_highlight_mode,
            spreadsheet_msg,
        )
    }
}

#[cfg(not(feature = "table_widget"))]
fn build_table_content_lazy<'a, R: EditableRecord>(
    descriptors: &'a [FieldDescriptor],
    catalog: &'a Vec<R>,
    header_row: Element<'a, Message>,
    total_width: f32,
    spreadsheet: &'a SpreadsheetState,
    current_highlight_orig: Option<usize>,
    is_highlight_mode: bool,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> Element<'a, Message> {
    let total_visible = spreadsheet.filtered_indices.len();

    // Virtual-scroll window: only render rows that are in (or near) the viewport.
    // Snap first_row to WINDOW_STEP multiples so the widget tree only rebuilds
    // every WINDOW_STEP rows of scroll rather than on every scroll pixel —
    // this keeps cosmic-text Korean layout from blocking the scroll thread.
    let overscan_rows = spreadsheet.overscan_rows();
    let raw_first =
        ((spreadsheet.vertical_scroll_offset / ROW_HEIGHT) as usize).saturating_sub(overscan_rows);
    let first_row = (raw_first / WINDOW_STEP) * WINDOW_STEP;
    let last_row = (((spreadsheet.vertical_scroll_offset + spreadsheet.viewport_height)
        / ROW_HEIGHT) as usize
        + overscan_rows
        + WINDOW_STEP)
        .min(total_visible);

    // Top spacer fills the height of all rows above the render window so the
    // scrollbar thumb and total content height stay correct.
    let top_spacer_height = first_row as f32 * ROW_HEIGHT;
    // Bottom spacer fills the rows below the render window.
    let bottom_spacer_height = (total_visible.saturating_sub(last_row)) as f32 * ROW_HEIGHT;

    let render_count = last_row.saturating_sub(first_row);
    let mut data_rows: Vec<Element<Message>> = Vec::with_capacity(render_count + 2);

    if top_spacer_height > 0.0 {
        data_rows.push(
            iced::widget::Space::new()
                .width(Length::Fill)
                .height(Length::Fixed(top_spacer_height))
                .into(),
        );
    }

    let col_widths: Vec<f32> = (0..descriptors.len())
        .map(|c| spreadsheet.column_width(c))
        .collect();
    let col_widths_static = col_widths.clone();

    for (filtered_idx, &orig_idx) in spreadsheet
        .filtered_indices
        .iter()
        .enumerate()
        .skip(first_row)
        .take(render_count)
    {
        if catalog.get(orig_idx).is_none() {
            continue;
        }
        let is_selected = spreadsheet.selected_orig == Some(orig_idx);
        let is_highlighted =
            is_highlight_mode && spreadsheet.highlighted_indices.contains(&orig_idx);
        let is_current_highlight = Some(orig_idx) == current_highlight_orig;

        if !spreadsheet.display_cache.is_empty() {
            // Key: orig_idx (stable identity), is_selected/is_highlighted/current (change on user action),
            // row_hash (changes on data edit), col_widths_gen (changes on resize)
            let row_key = (
                orig_idx,
                is_selected as u8,
                is_highlighted as u8,
                is_current_highlight as u8,
                spreadsheet.row_hash(orig_idx),
                spreadsheet.col_widths_gen,
            );
            let col_widths_row = col_widths_static.clone();
            let row_values: Vec<String> = spreadsheet
                .display_cache
                .get(orig_idx)
                .cloned()
                .unwrap_or_default();
            let para_cache = spreadsheet.paragraph_cache.clone();

            data_rows.push(
                iced::widget::lazy(row_key, move |_| -> Element<'static, Message> {
                    let row_style = style::spreadsheet_row(
                        is_selected,
                        filtered_idx,
                        is_highlighted,
                        is_current_highlight,
                    );
                    let id_cell = container(
                        text(format!("{}", orig_idx + 1))
                            .size(10)
                            .font(Font::MONOSPACE),
                    )
                    .width(ID_COL_WIDTH)
                    .padding([0, 6])
                    .height(ROW_HEIGHT)
                    .align_y(iced::Alignment::Center)
                    .style(if is_selected || is_current_highlight {
                        style::spreadsheet_id_cell_selected
                    } else {
                        style::spreadsheet_id_cell
                    })
                    .into();

                    let mut cells: Vec<_> = vec![id_cell];
                    cells.reserve(col_widths_row.len());
                    for (col, col_width) in col_widths_row.iter().enumerate() {
                        let raw = row_values.get(col).map(String::as_str).unwrap_or("");
                        let display = truncate_for_width(raw, *col_width);
                        cells.push(flattened_cell(display, *col_width, para_cache.clone()));
                    }

                    button(row(cells).spacing(0))
                        .on_press(spreadsheet_msg(SpreadsheetMessage::SelectRow(filtered_idx)))
                        .padding(0)
                        .style(row_style)
                        .into()
                })
                .into(),
            );
        }
    }

    if bottom_spacer_height > 0.0 {
        data_rows.push(
            iced::widget::Space::new()
                .width(Length::Fill)
                .height(Length::Fixed(bottom_spacer_height))
                .into(),
        );
    }

    // ── Sticky header ─────────────────────────────────────────────────────
    // The header lives in its own horizontal scrollable whose scrollbar is
    // hidden; a body `on_scroll` handler mirrors the body's horizontal
    // offset onto this scrollable, which keeps columns aligned while the
    // user scrolls the table.
    let header_scroll = scrollable(container(header_row).width(Length::Fixed(total_width)))
        .id(spreadsheet.header_scroll_id.clone())
        .direction(ScrollDir::Horizontal(
            iced::widget::scrollable::Scrollbar::new()
                .width(0)
                .scroller_width(0),
        ))
        .width(Length::Fill);

    // ── Body (two-axis scroll, drives the header) ─────────────────────────
    let body =
        scrollable(container(column(data_rows).spacing(0)).width(Length::Fixed(total_width)))
            .id(spreadsheet.body_scroll_id.clone())
            .direction(ScrollDir::Both {
                vertical: Default::default(),
                horizontal: Default::default(),
            })
            .on_scroll(move |vp| {
                spreadsheet_msg(SpreadsheetMessage::BodyScrolled(
                    vp.absolute_offset(),
                    vp.bounds().height,
                ))
            })
            .height(Length::Fill)
            .width(Length::Fill);

    let table: Element<Message> = column![header_scroll, body].spacing(0).into();

    // When a resize is in progress, wrap the entire table in a `MouseArea`.
    // This wrapper's bounds are the whole table (fixed for the duration of
    // the drag), so `on_move` fires on every pointer movement anywhere over
    // the table — not just the 5px divider strip — giving the user the
    // full viewport of travel. On release we exit drag mode.
    if spreadsheet.resizing_column.is_some() {
        iced::widget::mouse_area(table)
            .on_move(move |p| spreadsheet_msg(SpreadsheetMessage::ResizeColumnCursor(p.x)))
            .on_release(spreadsheet_msg(SpreadsheetMessage::EndResizeColumn))
            .interaction(iced::mouse::Interaction::ResizingHorizontally)
            .into()
    } else {
        table
    }
}

#[cfg(feature = "table_widget")]
fn build_table_content_widget<'a>(
    descriptors: &'a [FieldDescriptor],
    spreadsheet: &'a SpreadsheetState,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> Element<'a, Message> {
    use crate::view::editor::table_widget::{RowFlags, TableColumn, TableWidget};

    // Header is rendered inside the TableWidget itself — labels, sort
    // indicators, filter icons, and resize handles are painted directly,
    // so there is no separate header `scrollable` and no horizontal-scroll
    // mirror to keep in sync.

    let columns: Vec<TableColumn> = (0..descriptors.len())
        .map(|c| TableColumn {
            width_px: spreadsheet.column_width(c),
            label: descriptors[c].label.to_string(),
            sort: if spreadsheet.sort_column == Some(c) {
                Some(spreadsheet.sort_ascending)
            } else {
                None
            },
            has_filter: spreadsheet.column_filters.contains_key(&c),
        })
        .collect();

    let current_highlight_orig = spreadsheet.current_highlight_orig_idx();
    let is_highlight_mode = spreadsheet.filter_mode == GlobalFilterMode::Highlight;
    let selected_orig = spreadsheet.selected_orig;
    let highlighted = &spreadsheet.highlighted_indices;

    let row_flags = move |visible_idx: usize| -> RowFlags {
        let Some(&orig_idx) = spreadsheet.filtered_indices.get(visible_idx) else {
            return RowFlags::default();
        };
        RowFlags {
            selected: selected_orig == Some(orig_idx),
            highlighted: is_highlight_mode && highlighted.contains(&orig_idx),
            current_highlight: Some(orig_idx) == current_highlight_orig,
        }
    };

    let body: Element<Message> = TableWidget::new(
        &spreadsheet.display_cache,
        &spreadsheet.filtered_indices,
        columns,
        ID_COL_WIDTH_PX,
        row_flags,
        ROW_HEIGHT,
        spreadsheet.paragraph_cache.clone(),
    )
    .external_offset(
        spreadsheet.horizontal_scroll_offset,
        spreadsheet.vertical_scroll_offset,
    )
    .on_select(move |visible_idx| spreadsheet_msg(SpreadsheetMessage::SelectRow(visible_idx)))
    .on_scroll(move |x, y, vh| {
        spreadsheet_msg(SpreadsheetMessage::BodyScrolled(
            iced::widget::scrollable::AbsoluteOffset { x, y },
            vh,
        ))
    })
    .on_sort(move |c| spreadsheet_msg(SpreadsheetMessage::SortColumn(c)))
    .on_open_filter(move |c| spreadsheet_msg(SpreadsheetMessage::OpenColumnFilter(c)))
    .on_clear_filter(move |c| spreadsheet_msg(SpreadsheetMessage::ClearColumnFilter(c)))
    .on_start_resize(move |c| spreadsheet_msg(SpreadsheetMessage::StartResizeColumn(c)))
    .on_reset_column_width(move |c| spreadsheet_msg(SpreadsheetMessage::ResetColumnWidth(c)))
    .on_next_highlight(move || spreadsheet_msg(SpreadsheetMessage::NavigateNextHighlight))
    .on_prev_highlight(move || spreadsheet_msg(SpreadsheetMessage::NavigatePrevHighlight))
    .on_escape(move || spreadsheet_msg(SpreadsheetMessage::ClearFilter))
    .on_quick_filter(move |col, value| spreadsheet_msg(SpreadsheetMessage::QuickFilter(col, value)))
    .into();

    let table: Element<Message> = body;

    if spreadsheet.resizing_column.is_some() {
        iced::widget::mouse_area(table)
            .on_move(move |p| spreadsheet_msg(SpreadsheetMessage::ResizeColumnCursor(p.x)))
            .on_release(spreadsheet_msg(SpreadsheetMessage::EndResizeColumn))
            .interaction(iced::mouse::Interaction::ResizingHorizontally)
            .into()
    } else {
        table
    }
}

#[cfg(not(feature = "table_widget"))]
fn build_header_row<'a>(
    descriptors: &'a [FieldDescriptor],
    spreadsheet: &'a SpreadsheetState,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> Element<'a, Message> {
    let id_cell: Element<Message> = container(text("#").size(10).style(style::subtle_text))
        .width(ID_COL_WIDTH)
        .padding([0, 6])
        .height(ROW_HEIGHT)
        .align_y(iced::Alignment::Center)
        .style(style::spreadsheet_id_cell)
        .into();
    let data_header = build_data_header_row(descriptors, spreadsheet, spreadsheet_msg);
    container(row![id_cell, data_header].spacing(0))
        .style(style::spreadsheet_header)
        .into()
}

/// Build the column-header row *without* the leading `#` cell. The widget
/// path renders the `#` cell as a frozen header outside the horizontal
/// scrollable so it stays put while the data column headers scroll with
/// the body.
#[cfg(not(feature = "table_widget"))]
fn build_data_header_row<'a>(
    descriptors: &'a [FieldDescriptor],
    spreadsheet: &'a SpreadsheetState,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> Element<'a, Message> {
    use iced::widget::mouse_area;

    let mut header_cells: Vec<Element<Message>> = Vec::with_capacity(descriptors.len());

    let is_resizing = spreadsheet.resizing_column.is_some();

    for (col, desc) in descriptors.iter().enumerate() {
        let sort_indicator = match spreadsheet.sort_column {
            Some(c) if c == col => {
                if spreadsheet.sort_ascending {
                    " ▲"
                } else {
                    " ▼"
                }
            }
            _ => "",
        };

        let col_width = spreadsheet.column_width(col);
        let has_col_filter = spreadsheet.column_filters.contains_key(&col);
        // Reserve room for the filter badge when a filter is active (14 px).
        let badge_width = if has_col_filter { 14.0 } else { 0.0 };
        let label_width = (col_width - RESIZE_HANDLE_WIDTH - badge_width).max(0.0);

        // ─ Filter badge: small "◼ ×" shown when a column filter is active ─
        let filter_badge: Element<Message> = if has_col_filter {
            button(text("◼").size(8))
                .padding([0, 2])
                .on_press(spreadsheet_msg(SpreadsheetMessage::ClearColumnFilter(col)))
                .style(style::spreadsheet_header_button)
                .into()
        } else {
            horizontal_space().width(Length::Fixed(0.0)).into()
        };

        // The sort+label button also doubles as the quick-filter trigger on
        // right-click. For now, left-click sorts and a separate "▾" affordance
        // opens the value list — added here as a tiny icon after the label.
        let filter_btn: Element<Message> = button(text("▾").size(8))
            .padding([0, 2])
            .on_press(spreadsheet_msg(SpreadsheetMessage::OpenColumnFilter(col)))
            .style(style::spreadsheet_header_button)
            .into();

        let label_cell = container(
            row![
                button(
                    text(format!("{}{}", desc.label, sort_indicator))
                        .size(10)
                        .font(Font::MONOSPACE),
                )
                .on_press(spreadsheet_msg(SpreadsheetMessage::SortColumn(col)))
                .style(style::spreadsheet_header_button)
                .padding([0, 6])
                .width(Length::Fill),
                filter_btn,
                filter_badge,
            ]
            .spacing(0)
            .align_y(iced::Alignment::Center),
        )
        .width(Length::Fixed(label_width + badge_width))
        .height(ROW_HEIGHT)
        .align_y(iced::Alignment::Center);

        // Thin, clickable strip at the right edge of each header cell. It
        // captures the mouse-down (to record *which* column is being
        // resized) and double-click-to-reset; the actual drag tracking is
        // done by the outer `MouseArea` wrapping the whole table, so the
        // cursor has the full viewport to roam in.
        let _ = is_resizing;
        let resize_handle = mouse_area(
            container(text(""))
                .width(Length::Fixed(RESIZE_HANDLE_WIDTH))
                .height(ROW_HEIGHT)
                .style(style::resize_handle),
        )
        .interaction(iced::mouse::Interaction::ResizingHorizontally)
        .on_press(spreadsheet_msg(SpreadsheetMessage::StartResizeColumn(col)))
        .on_double_click(spreadsheet_msg(SpreadsheetMessage::ResetColumnWidth(col)));

        header_cells.push(row![label_cell, resize_handle].spacing(0).into());
    }

    row(header_cells).spacing(0).into()
}

/// A single cell inside a data row. Rendered as a plain `container` so that
/// clicks bubble up to the surrounding row-level `button`, the row's
/// background paints through, and the row's `text_color` cascades into the
/// cell text (so selected rows show the gold accent without per-cell styling).
///
/// Previously each cell was its own `button`. With ~18 columns × 64 rendered
/// rows that meant ~1150 button widgets per frame, each with its own state,
/// hover handling, and text-shaping wrapper — the dominant per-frame cost
/// during steady scrolling.
#[cfg(not(feature = "table_widget"))]
fn flattened_cell(
    display: String,
    col_width: f32,
    cache: ParagraphCache,
) -> Element<'static, Message> {
    container(cached_text(display, cache).size(10).font(Font::MONOSPACE))
        .padding([3, 8])
        .width(Length::Fixed(col_width))
        .height(ROW_HEIGHT)
        .align_y(iced::Alignment::Center)
        .into()
}

// ───────────────────────────────────────────────────────────────────────────
// Inspector panel (right-hand pane when a row is selected)
// ───────────────────────────────────────────────────────────────────────────

fn build_inspector_panel<'a, R: EditableRecord>(
    editor: &'a GenericEditorState<R>,
    spreadsheet: &'a SpreadsheetState,
    lookups: &'a HashMap<String, Vec<(String, String)>>,
    field_changed_msg: fn(usize, String, String) -> Message,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> Element<'a, Message> {
    let descriptors = R::field_descriptors();

    let header = container(
        row![
            text("Inspector").size(13),
            horizontal_space(),
            button(text("✕").size(11))
                .on_press(spreadsheet_msg(SpreadsheetMessage::CloseInspector))
                .style(style::browse_button)
                .padding([2, 6]),
        ]
        .align_y(iced::Alignment::Center)
        .padding([6, 10]),
    )
    .width(Fill)
    .style(style::spreadsheet_header);

    let mut fields: Column<Message> = column![].spacing(6).padding([8, 12]);

    if let Some(orig_idx) = spreadsheet.selected_orig {
        if let Some(record) = editor.catalog.as_ref().and_then(|c| c.get(orig_idx)) {
            for desc in descriptors.iter() {
                let value = record.get_field(desc.name);
                let lookup_data = match &desc.kind {
                    FieldKind::Lookup(key) => lookups.get(*key).cloned(),
                    _ => None,
                };
                let validation_error = record.validate_field(desc.name, &value);
                fields = fields.push(build_inspector_field(
                    desc,
                    value,
                    orig_idx,
                    lookup_data,
                    validation_error,
                    field_changed_msg,
                    &spreadsheet.inspector_textarea_contents,
                    spreadsheet_msg,
                ));
            }
        }
    } else {
        fields = fields.push(
            text("Click a row to inspect")
                .size(12)
                .style(style::subtle_text),
        );
    }

    let scroll = scrollable(fields).height(Length::Fill);

    container(column![header, scroll])
        .width(Fill)
        .height(Fill)
        .style(style::sidebar_container)
        .into()
}

#[allow(clippy::too_many_arguments)]
fn build_inspector_field<'a>(
    descriptor: &'a FieldDescriptor,
    value: String,
    orig_idx: usize,
    lookups: Option<Vec<(String, String)>>,
    validation_error: Option<String>,
    field_changed_msg: fn(usize, String, String) -> Message,
    textarea_contents: &'a HashMap<String, TextAreaContent>,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> Element<'a, Message> {
    let label = text(descriptor.label).size(11).style(style::subtle_text);

    let body: Element<'a, Message> = match &descriptor.kind {
        FieldKind::TextArea => {
            let field_name = descriptor.name.to_string();
            if let Some(tc) = textarea_contents.get(descriptor.name) {
                textarea::textarea(&tc.0, move |action| {
                    spreadsheet_msg(SpreadsheetMessage::TextAreaChanged(
                        orig_idx,
                        field_name.clone(),
                        action,
                    ))
                })
            } else {
                // Fallback before the first SelectRow initialises the content.
                text_input("", &value)
                    .on_input(move |v| field_changed_msg(orig_idx, field_name.clone(), v))
                    .padding(4)
                    .size(11)
                    .width(Length::Fill)
                    .into()
            }
        }
        FieldKind::Lookup(_) => {
            if let Some(options) = lookups {
                let field_name = descriptor.name.to_string();
                let selected = options
                    .iter()
                    .find(|(id, _)| id == &value)
                    .map(|(_, name)| name.clone());

                let options_vec: Vec<String> =
                    options.iter().map(|(_, name)| name.clone()).collect();

                pick_list(options_vec, selected, move |selected_name| {
                    let selected_id = options
                        .iter()
                        .find(|(_, name)| name == &selected_name)
                        .map(|(id, _)| id.clone())
                        .unwrap_or_default();
                    spreadsheet_msg(SpreadsheetMessage::InspectorFieldChanged(
                        orig_idx,
                        field_name.clone(),
                        selected_id,
                    ))
                })
                .width(Length::Fill)
                .into()
            } else {
                text_input("", &value)
                    .padding(4)
                    .size(11)
                    .width(Length::Fill)
                    .into()
            }
        }
        FieldKind::Enum { variants } => {
            let field_name = descriptor.name.to_string();
            let selected = variants.iter().find(|&&v| v == value).copied();
            pick_list(*variants, selected, move |selected_variant| {
                spreadsheet_msg(SpreadsheetMessage::InspectorFieldChanged(
                    orig_idx,
                    field_name.clone(),
                    selected_variant.to_string(),
                ))
            })
            .width(Length::Fill)
            .into()
        }
        _ => {
            let field_name = descriptor.name.to_string();
            text_input("", &value)
                .on_input(move |v| {
                    spreadsheet_msg(SpreadsheetMessage::InspectorFieldChanged(
                        orig_idx,
                        field_name.clone(),
                        v,
                    ))
                })
                .padding(4)
                .size(11)
                .width(Length::Fill)
                .into()
        }
    };

    let mut field_column = column![label, body].spacing(4);
    if let Some(err) = validation_error {
        field_column =
            field_column.push(
                text(err)
                    .size(10)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(iced::color!(0xff5252)),
                    }),
            );
    }
    field_column.into()
}

// ===========================================================================
// CSV export — async save dialog
// ===========================================================================

/// Build the async CSV-export task. Editors call this from their own message
/// handler so the returned `Task` can be flattened into the editor's update
/// function.
pub fn export_csv_task<R: EditableRecord>(
    spreadsheet: &SpreadsheetState,
    catalog: &[R],
    default_file_name: &str,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> iced::Task<Message> {
    use iced::Task;

    let bytes = match spreadsheet.to_csv_bytes(catalog) {
        Ok(b) => b,
        Err(e) => {
            return Task::done(Message::System(SystemMessage::ShowError(format!(
                "CSV export failed: {e}"
            ))));
        }
    };
    let default_file_name = default_file_name.to_string();

    Task::perform(
        async move {
            let Some(handle) = rfd::AsyncFileDialog::new()
                .set_file_name(&default_file_name)
                .add_filter("CSV", &["csv"])
                .save_file()
                .await
            else {
                return Err("cancelled".to_string());
            };
            let path = handle.path().to_path_buf();
            tokio::fs::write(&path, &bytes)
                .await
                .map(|_| path)
                .map_err(|e| e.to_string())
        },
        move |result| spreadsheet_msg(SpreadsheetMessage::CsvExported(result)),
    )
}
