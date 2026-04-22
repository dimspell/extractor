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
use iced::widget::pane_grid::{self, Pane};
use iced::widget::scrollable::Direction as ScrollDir;
use iced::widget::text_editor;
use iced::widget::{
    button, column, container, pick_list, progress_bar, row, scrollable, text, text_input, Column,
};
use iced::{Element, Fill, Font, Length};
use std::collections::HashMap;

const ROW_HEIGHT: f32 = 24.0;
/// How many extra rows to render above and below the visible viewport.
/// Large enough to hide pop-in during fast scrolling.
const OVERSCAN_ROWS: usize = 20;
const ID_COL_WIDTH_PX: f32 = 42.0;
const ID_COL_WIDTH: Length = Length::Fixed(ID_COL_WIDTH_PX);
/// Default pixel width for each data column; overridden per-column via
/// `SpreadsheetState::column_widths`. Using a fixed width (rather than
/// FillPortion) lets the table overflow its container and become scrollable.
const COL_WIDTH: f32 = 140.0;
const COL_WIDTH_MIN: f32 = 40.0;
const COL_WIDTH_MAX: f32 = 600.0;
/// Pixel width of the draggable separator between column headers.
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

/// High-level editing mode for the VIM-like status indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditingMode {
    #[default]
    Normal,
    Edit,
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

    // ── Selection / editing ────────────────────────────────────────────────
    pub selected_row: Option<usize>,
    /// `Some((filtered_idx, col))` when a cell is being edited.
    pub editing_cell: Option<(usize, usize)>,
    pub edit_buffer: String,
    /// Set when the last `set_field` or `validate_field` call rejected the
    /// value being edited; renders a red border on that cell.
    pub edit_invalid: bool,
    pub mode: EditingMode,

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

    // ── Column quick-filter ────────────────────────────────────────────────
    /// Per-column exact-match filters.  Key = column index, value = required cell value.
    pub column_filters: HashMap<usize, String>,
    /// Which column's quick-filter dropdown is currently open (`None` = closed).
    pub active_column_filter: Option<usize>,
    /// Pre-computed unique values for the open column filter dropdown.
    pub column_filter_options: Vec<String>,

    // ── Inspector textarea state ───────────────────────────────────────────
    /// One `text_editor::Content` per TextArea field of the currently-inspected
    /// record, keyed by field name. Populated on `SelectRow`, cleared on
    /// row change. Allows the cursor / selection to survive re-renders.
    pub inspector_textarea_contents: HashMap<String, TextAreaContent>,
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
            selected_row: None,
            editing_cell: None,
            edit_buffer: String::new(),
            edit_invalid: false,
            mode: EditingMode::Normal,
            show_inspector: false,
            pane_state: None,
            is_loading: false,
            filter_input_id: iced::advanced::widget::Id::unique(),
            body_scroll_id: iced::advanced::widget::Id::unique(),
            header_scroll_id: iced::advanced::widget::Id::unique(),
            column_widths: HashMap::new(),
            resizing_column: None,
            horizontal_scroll_offset: 0.0,
            vertical_scroll_offset: 0.0,
            viewport_height: 400.0,
            col_widths_gen: 0,
            column_filters: HashMap::new(),
            active_column_filter: None,
            column_filter_options: Vec::new(),
            inspector_textarea_contents: HashMap::new(),
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
            for (&col, required) in &self.column_filters {
                if let Some(desc) = descriptors.get(col) {
                    if !record.get_field(desc.name).eq_ignore_ascii_case(required) {
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

    pub fn select_row(&mut self, filtered_idx: usize) {
        self.selected_row = Some(filtered_idx);
    }

    pub fn start_editing<R: EditableRecord>(
        &mut self,
        filtered_idx: usize,
        col: usize,
        catalog: &[R],
    ) {
        let Some(&orig_idx) = self.filtered_indices.get(filtered_idx) else {
            return;
        };
        let Some(record) = catalog.get(orig_idx) else {
            return;
        };
        let descriptors = R::field_descriptors();
        let Some(desc) = descriptors.get(col) else {
            return;
        };
        self.editing_cell = Some((filtered_idx, col));
        self.edit_buffer = record.get_field(desc.name);
        self.edit_invalid = false;
        self.mode = EditingMode::Edit;
        // Keep selection consistent with the row being edited.
        if self.selected_row != Some(filtered_idx) {
            self.selected_row = Some(filtered_idx);
        }
    }

    pub fn commit_edit<R: EditableRecord>(
        &mut self,
        catalog: &mut [R],
        field_changed_msg: fn(usize, String, String) -> Message,
        orig_idx: usize,
    ) -> Option<Message> {
        let (_, col) = self.editing_cell?;
        let descriptors = R::field_descriptors();
        let Some(desc) = descriptors.get(col) else {
            self.cancel_editing();
            return None;
        };

        // Validate before we commit so the user keeps the invalid buffer and
        // can correct it in place rather than losing it silently.
        if let Some(_err) = catalog[orig_idx].validate_field(desc.name, &self.edit_buffer) {
            self.edit_invalid = true;
            return None;
        }

        let old_value = catalog[orig_idx].get_field(desc.name);
        let new_value = std::mem::take(&mut self.edit_buffer);
        self.editing_cell = None;
        self.edit_invalid = false;
        self.mode = EditingMode::Normal;

        if old_value == new_value {
            return None;
        }

        let applied = catalog[orig_idx].set_field(desc.name, new_value.clone());
        if !applied {
            // set_field rejected the parse; resurrect the edit so the user
            // can see what went wrong.
            self.edit_buffer = new_value;
            self.edit_invalid = true;
            self.editing_cell = Some((
                self.filtered_indices
                    .iter()
                    .position(|&i| i == orig_idx)
                    .unwrap_or(0),
                col,
            ));
            self.mode = EditingMode::Edit;
            return None;
        }

        Some(field_changed_msg(
            orig_idx,
            desc.name.to_string(),
            new_value,
        ))
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

    /// Called on mouse-down on a column's resize handle. Captures the
    /// baseline width; the anchor cursor position is captured lazily on the
    /// first subsequent `on_move` so the drag doesn't jump when the mouse
    /// first settles.
    pub fn begin_column_resize(&mut self, col: usize) {
        let anchor_width = self.column_width(col);
        self.resizing_column = Some(ColumnDragState {
            col,
            anchor_width,
            anchor_cursor_x: None,
        });
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
        self.col_widths_gen = self.col_widths_gen.wrapping_add(1);
    }

    pub fn end_column_resize(&mut self) {
        self.resizing_column = None;
    }

    /// Double-click reset: drop any override for `col` so it returns to the
    /// default `COL_WIDTH`.
    pub fn reset_column_width(&mut self, col: usize) {
        self.column_widths.remove(&col);
        self.col_widths_gen = self.col_widths_gen.wrapping_add(1);
    }

    pub fn cancel_editing(&mut self) {
        self.editing_cell = None;
        self.edit_buffer.clear();
        self.edit_invalid = false;
        self.mode = EditingMode::Normal;
    }

    /// Move selection one row up; returns the new `filtered_idx` or `None`
    /// when there is nothing to navigate.
    pub fn navigate_up(&mut self) -> Option<usize> {
        let total = self.filtered_indices.len();
        if total == 0 {
            return None;
        }
        let new_idx = match self.selected_row {
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
        let new_idx = match self.selected_row {
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
    /// the visible viewport. Used by keyboard navigation commands.
    pub fn scroll_y_for_row(&self, filtered_idx: usize) -> f32 {
        ((filtered_idx as f32 + 0.5) * ROW_HEIGHT - self.viewport_height / 2.0).max(0.0)
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
    StartEdit(usize, usize),
    EditCellInput(String),
    CommitEdit(usize),
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
    /// Double-click on a resize handle: drop the width override for `col`.
    ResetColumnWidth(usize),

    // ── Inspector textarea editing ─────────────────────────────────────────
    /// Fired by a `text_editor` widget in the inspector panel.
    /// `(orig_idx, field_name, action)` — the handler performs the action on
    /// the stored `TextAreaContent` and syncs the string value back to the
    /// record via `FieldChanged`.
    TextAreaChanged(usize, String, text_editor::Action),
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

    // Column quick-filter pick_list (visible only when a column filter menu is open).
    let col_filter_area: Element<Message> = if let Some(col) = spreadsheet.active_column_filter {
        let options = spreadsheet.column_filter_options.clone();
        let current = spreadsheet.column_filters.get(&col).cloned();
        let col_filter_pick = pick_list(options, current, move |val| {
            spreadsheet_msg(SpreadsheetMessage::ApplyColumnFilter(col, val))
        })
        .placeholder("Pick value…")
        .width(Length::Fixed(160.0));
        let close_btn = button(text("✕").size(11))
            .padding([2, 6])
            .on_press(spreadsheet_msg(SpreadsheetMessage::OpenColumnFilter(col)))
            .style(style::browse_button);
        row![
            text("Col filter:").size(11).style(style::subtle_text),
            col_filter_pick,
            close_btn,
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center)
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
    let mode_chip: Element<Message> = match spreadsheet.mode {
        EditingMode::Normal => container(text("NORMAL").size(10).style(style::normal_mode_text))
            .padding([2, 8])
            .style(style::normal_mode_chip)
            .into(),
        EditingMode::Edit => container(text("EDIT").size(10).style(style::edit_mode_text))
            .padding([2, 8])
            .style(style::edit_mode_chip)
            .into(),
    };

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
            mode_chip,
            horizontal_space().width(12),
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

    let header_row = build_header_row(descriptors, spreadsheet, spreadsheet_msg);

    // Precompute the `orig_idx` of the "currently highlighted" row for O(1)
    // checks while building each row.
    let current_highlight_orig = spreadsheet.current_highlight_orig_idx();
    let highlight_set: std::collections::HashSet<usize> =
        spreadsheet.highlighted_indices.iter().copied().collect();

    let total_visible = spreadsheet.filtered_indices.len();

    // Virtual-scroll window: only render rows that are in (or near) the viewport.
    let first_row = ((spreadsheet.vertical_scroll_offset / ROW_HEIGHT) as usize)
        .saturating_sub(OVERSCAN_ROWS);
    let last_row = (((spreadsheet.vertical_scroll_offset + spreadsheet.viewport_height)
        / ROW_HEIGHT) as usize
        + OVERSCAN_ROWS)
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

    for (filtered_idx, &orig_idx) in spreadsheet
        .filtered_indices
        .iter()
        .enumerate()
        .skip(first_row)
        .take(render_count)
    {
        let Some(record) = catalog.get(orig_idx) else {
            continue;
        };
        let is_selected = spreadsheet.selected_row == Some(filtered_idx);
        let is_highlighted = spreadsheet.filter_mode == GlobalFilterMode::Highlight
            && highlight_set.contains(&orig_idx);
        let is_current_highlight = Some(orig_idx) == current_highlight_orig;
        let is_editing_row = spreadsheet.editing_cell.map(|(f, _)| f) == Some(filtered_idx);

        if is_editing_row {
            // Non-lazy path: the editing row contains a `text_input` that borrows
            // `spreadsheet.edit_buffer`, so it cannot be wrapped in `lazy` (which
            // requires `Element<'static, ...>`). At most one row is ever in this
            // state, so the cost is negligible.
            let row_style = style::spreadsheet_row(
                is_selected,
                filtered_idx,
                is_highlighted,
                is_current_highlight,
            );
            let id_cell: Element<Message> = container(
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
            let mut cells: Vec<Element<Message>> = vec![id_cell];
            for (col, desc) in descriptors.iter().enumerate() {
                let value = record.get_field(desc.name);
                let is_cell_editing = spreadsheet.editing_cell.map(|(_, c)| c) == Some(col);
                cells.push(build_data_cell(
                    desc,
                    &value,
                    filtered_idx,
                    col,
                    orig_idx,
                    is_cell_editing,
                    is_selected,
                    spreadsheet.column_width(col),
                    spreadsheet,
                    record,
                    spreadsheet_msg,
                ));
            }
            data_rows.push(
                button(row(cells).spacing(0))
                    .on_press(spreadsheet_msg(SpreadsheetMessage::SelectRow(filtered_idx)))
                    .padding(0)
                    .style(row_style)
                    .into(),
            );
        } else {
            // Lazy path: pre-compute all owned data the closure needs.
            // The closure is only executed when its key changes; during scroll
            // the key is stable so iced reuses the cached widget tree, giving
            // near-zero CPU cost per scroll event.
            let field_values: Vec<String> = descriptors
                .iter()
                .map(|d| record.get_field(d.name))
                .collect();
            let validation_errors: Vec<bool> = descriptors
                .iter()
                .zip(field_values.iter())
                .map(|(d, v)| record.validate_field(d.name, v).is_some())
                .collect();
            let col_widths: Vec<f32> = (0..descriptors.len())
                .map(|c| spreadsheet.column_width(c))
                .collect();

            // Hash the field values into a single u64 so the key tuple stays
            // cheap to compare (no Vec comparison on every render).
            let field_hash: u64 = {
                use std::hash::{Hash, Hasher};
                let mut h = std::collections::hash_map::DefaultHasher::new();
                field_values.hash(&mut h);
                h.finish()
            };

            let row_key = (
                filtered_idx,
                orig_idx,
                is_selected as u8,
                is_highlighted as u8,
                is_current_highlight as u8,
                field_hash,
                spreadsheet.col_widths_gen,
            );

            data_rows.push(
                iced::widget::lazy(row_key, move |_| -> Element<'static, Message> {
                    let row_style = style::spreadsheet_row(
                        is_selected,
                        filtered_idx,
                        is_highlighted,
                        is_current_highlight,
                    );
                    let id_cell: Element<'static, Message> = container(
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

                    let mut cells: Vec<Element<'static, Message>> = vec![id_cell];
                    for (col, _desc) in descriptors.iter().enumerate() {
                        let value = &field_values[col];
                        let col_width = col_widths[col];
                        let is_invalid = validation_errors[col];

                        let char_budget = ((col_width / 7.0) as usize).max(4);
                        let display: String = if value.chars().count() > char_budget {
                            format!("{}…", value.chars().take(char_budget).collect::<String>())
                        } else {
                            value.clone()
                        };

                        let press_msg = if is_selected {
                            SpreadsheetMessage::StartEdit(filtered_idx, col)
                        } else {
                            SpreadsheetMessage::SelectRow(filtered_idx)
                        };

                        let container_style = if is_invalid {
                            style::spreadsheet_cell_invalid
                        } else {
                            style::spreadsheet_cell
                        };

                        cells.push(
                            container(
                                button(text(display).size(10).font(Font::MONOSPACE))
                                    .on_press(spreadsheet_msg(press_msg))
                                    .style(style::spreadsheet_cell_btn)
                                    .padding([3, 8])
                                    .width(Length::Fill),
                            )
                            .width(Length::Fixed(col_width))
                            .height(ROW_HEIGHT)
                            .align_y(iced::Alignment::Center)
                            .style(container_style)
                            .into(),
                        );
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

fn build_header_row<'a>(
    descriptors: &'a [FieldDescriptor],
    spreadsheet: &'a SpreadsheetState,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> Element<'a, Message> {
    use iced::widget::mouse_area;

    let mut header_cells: Vec<Element<Message>> =
        vec![container(text("#").size(10).style(style::subtle_text))
            .width(ID_COL_WIDTH)
            .padding([0, 6])
            .height(ROW_HEIGHT)
            .align_y(iced::Alignment::Center)
            .style(style::spreadsheet_id_cell)
            .into()];

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

    container(row(header_cells).spacing(0))
        .style(style::spreadsheet_header)
        .into()
}

#[allow(clippy::too_many_arguments)]
fn build_data_cell<'a, R: EditableRecord>(
    desc: &'a FieldDescriptor,
    value: &str,
    filtered_idx: usize,
    col: usize,
    orig_idx: usize,
    is_cell_editing: bool,
    is_row_selected: bool,
    col_width: f32,
    spreadsheet: &'a SpreadsheetState,
    record: &R,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> Element<'a, Message> {
    if is_cell_editing {
        let container_style = if spreadsheet.edit_invalid {
            style::spreadsheet_cell_invalid
        } else {
            style::spreadsheet_cell
        };
        return container(
            text_input("", &spreadsheet.edit_buffer)
                .on_input(move |v| spreadsheet_msg(SpreadsheetMessage::EditCellInput(v)))
                .on_submit(spreadsheet_msg(SpreadsheetMessage::CommitEdit(orig_idx)))
                .padding([3, 6])
                .size(10)
                .font(Font::MONOSPACE)
                .width(Length::Fill)
                .style(style::spreadsheet_cell_editor),
        )
        .width(Length::Fixed(col_width))
        .height(ROW_HEIGHT)
        .align_y(iced::Alignment::Center)
        .style(container_style)
        .into();
    }

    // Longer columns show more text; the ellipsis budget scales with width.
    let char_budget = ((col_width / 7.0) as usize).max(4);
    let display = if value.chars().count() > char_budget {
        format!("{}…", value.chars().take(char_budget).collect::<String>())
    } else {
        value.to_string()
    };

    // "Click to select, click again to edit": when the row is already selected,
    // a further click on the cell begins an edit. Otherwise the click just
    // selects the row.
    let press_msg = if is_row_selected {
        SpreadsheetMessage::StartEdit(filtered_idx, col)
    } else {
        SpreadsheetMessage::SelectRow(filtered_idx)
    };

    let invalid = record.validate_field(desc.name, value).is_some();
    let container_style = if invalid {
        style::spreadsheet_cell_invalid
    } else {
        style::spreadsheet_cell
    };

    container(
        button(text(display).size(10).font(Font::MONOSPACE))
            .on_press(spreadsheet_msg(press_msg))
            .style(style::spreadsheet_cell_btn)
            .padding([3, 8])
            .width(Length::Fill),
    )
    .width(Length::Fixed(col_width))
    .height(ROW_HEIGHT)
    .align_y(iced::Alignment::Center)
    .style(container_style)
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

    if let Some(filtered_idx) = spreadsheet.selected_row {
        if let Some(&orig_idx) = spreadsheet.filtered_indices.get(filtered_idx) {
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
                    field_changed_msg(orig_idx, field_name.clone(), selected_id)
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
        _ => {
            let field_name = descriptor.name.to_string();
            text_input("", &value)
                .on_input(move |v| field_changed_msg(orig_idx, field_name.clone(), v))
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
