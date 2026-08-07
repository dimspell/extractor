use iced::widget::pane_grid;

use super::domain::write_mode::{EncodingEntry, WriteMode};
use super::selection::NavDir;
use crate::state::InspectorSource;

/// Messages produced by the hex editor.
#[derive(Debug, Clone)]
pub enum HexEditorMessage {
    // ── Pane grid layout (Halloy-style movable panels) ──────────────────
    /// A pane was clicked — sets focus for keyboard routing.
    PaneClicked(pane_grid::Pane),
    /// A split divider was dragged.
    PaneResized(pane_grid::ResizeEvent),
    /// A pane was dragged to a new position (reordering / docking).
    PaneDragged(pane_grid::DragEvent),
    /// Split the focused pane along the given axis.
    SplitPane(pane_grid::Axis),
    /// Close the focused pane (removed from grid).
    ClosePane,

    /// User asked to change the row width. Any value within
    /// `MIN_BYTES_PER_ROW..=MAX_BYTES_PER_ROW` (1–64) is accepted; the 8/16/32
    /// buttons are just presets, and the settings modal also allows typing a
    /// custom value.
    SetBytesPerRow(u8),
    /// Draft text changed in the custom bytes-per-row input of the settings
    /// modal. Parsed on submit.
    BytesPerRowInputChanged(String),
    /// The custom bytes-per-row input was submitted with an invalid value
    /// (non-numeric or outside `MIN_BYTES_PER_ROW..=MAX_BYTES_PER_ROW`).
    /// Surfaces a transient status message.
    BytesPerRowInputInvalid,
    /// Single click on a cell — sets `anchor = cursor = addr`.
    SelectAt(u64),
    /// Shift-click or drag — moves cursor only.
    ExtendTo(u64),
    /// Keyboard navigation; `extend = true` for Shift-modified moves.
    Nav { dir: NavDir, extend: bool },

    // ── Inline editing (matrix) ───────────────────────────────────────────
    /// Enter overwrite-edit mode at `addr` with an empty draft.
    BeginEdit(u64),
    /// Append a hex digit to the active edit draft.
    EditTypeChar(char),
    /// Remove the last digit from the active edit draft.
    EditBackspace,
    /// Discard the active edit without writing.
    EditCancel,
    /// In text write mode: write 0x00 at the current cursor and advance.
    DeleteByteAtCursor,
    /// Commit the active draft. If `advance` is true, move the cursor +1
    /// byte and re-enter edit mode at the new address (Tab/Enter/auto on
    /// second digit). Otherwise just commit and stay put.
    EditCommit { advance: bool },

    // ── Programmatic writes ───────────────────────────────────────────────
    /// Overwrite `bytes.len()` bytes starting at `addr`. Used by the
    /// inspector modal and any future scripted edit path.
    WriteBytes { addr: u64, bytes: Vec<u8> },

    // ── Inspector ────────────────────────────────────────────────────────
    /// Switch which buffer the inspector decodes (main file vs comparison).
    SetInspectorSource(InspectorSource),
    /// Copy the decoded value of inspector entry `idx` to the clipboard.
    CopyInspectorValue(usize),

    // ── Inspector edit modal ──────────────────────────────────────────────
    /// Open the inspector edit modal for entry index `idx` at the current
    /// cursor.
    BeginInspectorEdit(usize),
    /// Update the modal's text-input draft.
    SetInspectorDraft(String),
    /// Close the modal without writing.
    CloseInspectorEdit,
    /// Encode the modal's draft and write it to the buffer.
    CommitInspectorEdit,

    // ── Save into recording ───────────────────────────────────────────────
    /// User pressed "Save into recording" — fire-and-forget; the async
    /// follow-up message is [`SavedIntoRecording`].
    SaveIntoRecording,
    /// Async result from the save flow.
    SavedIntoRecording(Result<String, String>),
    /// Wipe the editor's transient status_msg.
    ClearStatus,

    // ── Pattern highlighting ─────────────────────────────────────────────
    /// Create a pattern from the current selection range (CTRL+E).
    CreatePattern,
    /// Remove pattern at a specific address.
    RemovePatternAt(u64),
    /// Remove pattern at `context_menu_addr` — used by the context menu to
    /// avoid baking stale addresses into native menu entries.
    RemovePatternAtContextMenu,
    /// Clear all patterns.
    ClearAllPatterns,
    /// Right-click at a specific address — used to determine which context
    /// menu options to show (remove pattern vs create pattern).
    RightClickAt(u64),

    // ── Repeat pattern dialog ────────────────────────────────────────────
    /// Open the repeat-pattern dialog from a selection range.
    BeginRepeatedPattern,
    /// Update the repeat-count draft.
    SetRepeatedPatternDraft(String),
    /// Update the label draft.
    SetRepeatedPatternLabel(String),
    /// Parse the count and create repeated pattern entries under a named group.
    CommitRepeatedPattern,
    /// Dismiss the dialog without creating patterns.
    CloseRepeatedPattern,

    // ── Goto address ────────────────────────────────────────────────────
    /// Open the goto-address dialog.
    OpenGotoDialog,
    /// Update the input draft.
    SetGotoDraft(String),
    /// Parse and navigate.
    CommitGoto,
    /// Dismiss the dialog.
    CloseGotoDialog,

    // ── Search ──────────────────────────────────────────────────────────
    /// Open the search overlay.
    OpenSearch,
    /// Trigger a search with the given query string.
    Search(String),
    /// Toggle search mode (hex / ASCII / decimal).
    ToggleSearchMode,
    /// Navigate to the next match.
    SearchNext,
    /// Navigate to the previous match.
    SearchPrev,
    /// Close the search overlay.
    CloseSearch,
    /// Set the decimal search byte width (1/2/4/8).
    SetSearchWidth(u8),
    /// Toggle decimal search endianness (little/big).
    ToggleSearchEndian,

    // ── Pattern list panel ───────────────────────────────────────────────
    /// Show/hide the pattern list panel.
    TogglePatternList,
    /// Show/hide the inspector panel.
    ToggleInspector,
    /// Navigate to a pattern's start address.
    NavigateToPattern(usize),
    /// Remove a pattern by its id.
    RemovePattern(usize),
    /// Collapse / expand a repeated-pattern group in the pattern list.
    TogglePatternGroup(usize),

    // ── Pattern group operations ─────────────────────────────────────────
    /// Remove an entire group and all its patterns.
    RemovePatternGroup(usize),
    /// Begin inline rename of a group (opens text input).
    BeginRenameGroup(usize),
    /// Update the rename draft.
    SetRenameGroupDraft(String),
    /// Commit the new label for the group being renamed.
    CommitRenameGroup,
    /// Cancel inline rename.
    CancelRenameGroup,
    /// Cycle the group's colour index (all child patterns update too).
    CycleGroupColor(usize),
    /// Cycle a single pattern's colour index.
    CyclePatternColor(usize),

    // ── Pattern annotations ──────────────────────────────────────────────
    /// Set (or replace) a pattern's annotation text.
    SetPatternAnnotation(usize, String),
    /// Remove a pattern's annotation.
    ClearPatternAnnotation(usize),

    // ── Pattern import / export ──────────────────────────────────────────
    /// Export all patterns and groups to a JSON file.
    ExportPatterns,
    /// Import patterns and groups from a JSON file.
    ImportPatterns,
    /// Async result after exporting.
    PatternsExported(Result<(), String>),
    /// Async result after importing — carries the file content so the handler
    /// can parse it with mutable state access.
    PatternsImported(Result<String, String>),

    // ── Address format ──────────────────────────────────────────────────
    /// Toggle between hex and decimal address display.
    ToggleAddrFormat,

    // ── Settings modal ──────────────────────────────────────────────────
    /// Open the settings modal.
    OpenSettings,
    /// Close the settings modal.
    CloseSettings,
    /// Switch the active colour theme (dark / light).
    SetTheme(crate::ui::theme::ThemeVariant),
    /// Switch the default byte-colouring scheme.
    SetColorScheme(crate::ui::coloring::ColorScheme),
    /// Enable/disable dim-nulls regardless of colour scheme.
    SetDimNulls(bool),
    /// Show/hide the entropy colour band in the address gutter.
    SetShowEntropyBand(bool),
    /// Show/hide the minimap overview strip.
    SetShowMinimapEnabled(bool),

    // ── Write mode / text encoding ──────────────────────────────────────
    /// Switch the active write mode (Hex, ASCII, UTF-8, Windows-1250,
    /// EUC-KR, or a custom encoding).
    SetWriteMode(WriteMode),
    /// Open the "encoding settings" modal where the user can add/remove
    /// custom text encodings.
    OpenEncodingSettings,
    /// Close the encoding-settings modal.
    CloseEncodingSettings,
    /// Add a custom encoding entry by its index in the common list.
    AddCustomEncoding(usize),
    /// Remove a custom encoding entry by its index.
    RemoveCustomEncoding(usize),
    /// Bulk-replace the entire custom encoding list (e.g. on deserialise).
    SetCustomEncodings(Vec<EncodingEntry>),
    /// Set the address format: `true` = decimal, `false` = hex.
    SetAddrFormat(bool),
    /// Reset all settings to their defaults (colour scheme, dim-nulls, address
    /// format, bytes-per-row).
    ResetSettings,

    // ── Fill Selection ──────────────────────────────────────────────────
    /// Open the fill-selection dialog (context menu with active selection).
    BeginFill,
    /// Update the fill-pattern draft text.
    SetFillDraft(String),
    /// Parse the draft and write the repeated pattern across the selection.
    CommitFill,
    /// Dismiss the fill dialog without writing.
    CloseFill,

    // ── Extend File ─────────────────────────────────────────────────────
    /// Open the extend-file dialog (context menu at cursor).
    BeginExtend,
    /// Update the byte-count draft text.
    SetExtendCount(String),
    /// Update the fill-pattern draft text.
    SetExtendPattern(String),
    /// Insert the drafted byte count, filled with the drafted pattern.
    CommitExtend,
    /// Dismiss the extend dialog without inserting.
    CloseExtend,

    // ── Copy / Paste ────────────────────────────────────────────────────
    /// Copy the selected byte range as hex text to the clipboard.
    CopySelection,
    /// Read clipboard contents for paste (triggers async clipboard read).
    Paste,
    /// Async result: clipboard contents to paste as hex bytes.
    PasteContent(String),
    /// Async result from clipboard write (always succeeds — we don't surface errors).
    ClipboardWriteResult,

    // ── Byte statistics / entropy panel ───────────────────────────────
    /// Show/hide the byte statistics panel.
    ToggleStats,
    /// Trigger full-file analysis and update cached stats.
    AnalyzeFile,
    /// Trigger selection-only analysis.
    AnalyzeSelection,
    /// Async result: file-level statistics + row entropies (computed together
    /// to avoid double-copying the byte slice and a frame-race between messages).
    FileAndRowEntropiesComputed(
        Box<crate::domain::byte_stats::ByteStatistics>,
        Box<crate::domain::byte_stats::RowEntropyCache>,
    ),
    /// Async result: selection-level statistics computed.
    SelectionAnalyzed(Box<crate::domain::byte_stats::ByteStatistics>),

    // ── Side-by-side diff view ───────────────────────────────────────────
    /// User clicked "Diff Against File…" — triggers async file picker + load.
    LoadComparisonFile,
    /// Async result: the comparison file was loaded or an error occurred.
    /// `Ok((data, name))` on success, `Err(reason)` on failure.
    ComparisonFileLoaded(Result<(Vec<u8>, String), String>),
    /// Close the diff view and revert the pane to a regular matrix.
    CloseComparison,
    /// User clicked or navigated to a byte address in the diff view.
    /// Selects the address and centers the viewport on it. `is_baseline`
    /// is `true` when the click landed on the left (baseline) side, so the
    /// inspector follows the file being inspected.
    DiffAddrSelected { addr: u64, is_baseline: bool },
    /// User shift-dragged in the diff view to extend the selection.
    /// `is_baseline` marks which side the drag ended on.
    DiffExtendTo { addr: u64, is_baseline: bool },
    /// Jump to the next contiguous diff chunk in the comparison.
    DiffNavNext,
    /// Jump to the previous contiguous diff chunk.
    DiffNavPrev,
    /// Toggle "show only diff rows" mode in the diff view.
    ToggleDiffReview,

    // ── Export as text ──────────────────────────────────────────────────
    /// Open the export config modal.
    OpenExportConfig,
    /// Close the export config modal without exporting.
    CloseExportConfig,
    /// Toggle the address-gutter checkbox in the export config modal.
    SetExportShowAddress(bool),
    /// Toggle the decimal-address checkbox.
    SetExportAddressDecimal(bool),
    /// Toggle the show-ASCII column checkbox.
    SetExportShowAscii(bool),
    /// User confirmed the config — start the export flow (file dialog + write).
    CommitExport,
    /// Result after the text export completes.
    TextExportCompleted(Result<(), String>),
}
