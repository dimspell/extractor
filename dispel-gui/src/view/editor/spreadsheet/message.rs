//! All messages emitted from the spreadsheet view. Editor handlers route
//! them through `handle_spreadsheet_messages!` in
//! `crate::update::editor::common`.

use super::caches::ComputedCaches;
use super::state::GlobalFilterMode;
use iced::widget::text_editor;

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
    /// Fired by the `TableWidget` after it commits a scroll. Carries the
    /// absolute offset and the visible viewport height so programmatic
    /// scroll-to-row math computes against an up-to-date viewport.
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
    /// Close the column filter modal.
    CloseColumnFilterModal,
}
