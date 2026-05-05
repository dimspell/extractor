//! Spreadsheet UI state and the methods that mutate it.
//!
//! The view layer never touches these fields directly — every change goes
//! through one of the methods on [`SpreadsheetState`] so the invariants
//! (sort/filter consistency, selection survival across filter changes,
//! pane-grid bookkeeping) stay in one place.

use super::caches::{compute_caches, ComputedCaches};
use super::constants::{COL_WIDTH, COL_WIDTH_MAX, COL_WIDTH_MIN, ID_COL_WIDTH_PX, ROW_HEIGHT};
use crate::components::editor::editable::EditableRecord;
use crate::components::textarea::TextAreaContent;
use crate::view::editor::paragraph_cache::ParagraphCache;
use iced::widget::pane_grid::{self, Pane};
use std::collections::{HashMap, HashSet};

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
    /// Absolute horizontal scroll offset. The widget rehydrates from this on
    /// each layout, so programmatic navigation just writes here.
    pub horizontal_scroll_offset: f32,
    /// Absolute vertical scroll offset.
    pub vertical_scroll_offset: f32,
    /// Height of the visible body. Updated from `BodyScrolled` so
    /// `scroll_y_for_row` and `ensure_row_visible_y` use the real viewport.
    pub viewport_height: f32,

    // ── Catalog-derived caches ─────────────────────────────────────────────
    /// Bumped whenever any column width changes. Threaded into the table
    /// widget's paragraph-cache key so reshape happens at most once per
    /// (text, width).
    pub col_widths_gen: u32,
    /// Cached row hashes parallel to the catalog.
    pub row_hashes: Vec<u64>,
    /// Pre-computed display strings for every row, indexed by `orig_idx`.
    /// Each inner `Vec` has one entry per column.
    pub display_cache: Vec<Vec<String>>,
    /// Per-cell validation flags (`true` = invalid), indexed by `orig_idx`.
    pub validation_cache: Vec<Vec<bool>>,

    // ── Column quick-filter ────────────────────────────────────────────────
    /// Per-column exact-match filters. Key = column index, value = set of
    /// selected values.
    pub column_filters: HashMap<usize, HashSet<String>>,
    /// Which column's quick-filter dropdown is currently open
    /// (`None` = closed).
    pub active_column_filter: Option<usize>,
    /// Pre-computed unique values with counts for the open column filter.
    pub column_filter_options: Vec<ColumnFilterOption>,
    /// Search query within the column filter dropdown.
    pub column_filter_search: String,

    // ── Inspector textarea state ───────────────────────────────────────────
    /// One `text_editor::Content` per TextArea field of the currently-inspected
    /// record, keyed by field name. Populated on `SelectRow`. Allows the
    /// cursor / selection to survive re-renders.
    pub inspector_textarea_contents: HashMap<String, TextAreaContent>,

    /// Process-wide cache of pre-shaped paragraphs. Shared by every cell
    /// the table widget paints; cheap to clone (Arc).
    pub paragraph_cache: ParagraphCache,
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
            paragraph_cache: ParagraphCache::default(),
        }
    }
}

impl SpreadsheetState {
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

    /// Move the current-highlight cursor to the next match, with wrap-around.
    /// Safe to call when no highlights are present.
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
    /// visibility. Used by keyboard navigation and highlight-step messages.
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
    /// press on the same column within 400 ms): the caller should auto-size
    /// the column and skip starting a drag. Returns `false` for a normal
    /// single press, in which case the drag is started and the caller does
    /// nothing extra.
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

    /// Double-click auto-size: measure the longest cell value among the
    /// filtered rows plus the header label and set the column width
    /// accordingly, clamped to `[COL_WIDTH_MIN, COL_WIDTH_MAX]`.
    pub fn auto_size_column<R: EditableRecord>(&mut self, col: usize, catalog: &[R]) {
        let descriptors = R::field_descriptors();
        let Some(desc) = descriptors.get(col) else {
            return;
        };

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

    /// Body-scrollable Y offset that centers `filtered_idx` in the viewport.
    /// Used by jump-style navigation (highlight cursor, bottom).
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

    /// Record a programmatic scroll target. The custom `TableWidget` reads
    /// these fields on its next layout to snap its internal offset, so this
    /// mutation is the only way programmatic navigation moves the viewport.
    pub fn record_target_offset(&mut self, x: f32, y: f32) {
        self.horizontal_scroll_offset = x;
        self.vertical_scroll_offset = y;
    }

    /// Record a scroll event published by the table widget. Mirrors the
    /// widget's offset and viewport into state so programmatic navigation
    /// (`record_target_offset`) computes against an up-to-date viewport.
    pub fn record_scroll(&mut self, offset_x: f32, offset_y: f32, viewport_height: f32) {
        self.horizontal_scroll_offset = offset_x;
        self.vertical_scroll_offset = offset_y;
        self.viewport_height = viewport_height;
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
    /// large catalogs.
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

    /// Get the raw display string for a cell, cloned. The table widget paints
    /// truncated glyphs directly, so callers receive the untruncated value.
    pub fn get_display(&self, orig_idx: usize, col: usize) -> String {
        self.display_cache
            .get(orig_idx)
            .and_then(|row| row.get(col))
            .cloned()
            .unwrap_or_default()
    }

    /// Borrowed equivalent of [`get_display`](Self::get_display).
    pub fn get_display_ref(&self, orig_idx: usize, col: usize) -> Option<&String> {
        self.display_cache
            .get(orig_idx)
            .and_then(|row| row.get(col))
    }

    /// Cached hash for a row by original index.
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

    /// Create the pane state with only the Table pane. The Inspector pane is
    /// added on demand when a row is first selected.
    pub fn init_pane_state(&mut self) {
        let (state, _first) = pane_grid::State::new(SpreadsheetPaneContent::Table);
        self.pane_state = Some(state);
    }

    /// Add or remove the inspector pane to match `show_inspector`. When
    /// adding, the inspector gets ~30 % of the width.
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
    if haystack.is_ascii() {
        return hb
            .windows(nb.len())
            .any(|w| w.iter().zip(nb).all(|(&h, &n)| h.to_ascii_lowercase() == n));
    }
    haystack.to_lowercase().contains(needle_lower)
}
