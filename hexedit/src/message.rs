use super::selection::NavDir;

/// Messages produced by the hex editor.
#[derive(Debug, Clone)]
pub enum HexEditorMessage {
    /// User asked to change the row width (8/16/32).
    SetBytesPerRow(u8),
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
    /// Commit the active draft. If `advance` is true, move the cursor +1
    /// byte and re-enter edit mode at the new address (Tab/Enter/auto on
    /// second digit). Otherwise just commit and stay put.
    EditCommit { advance: bool },

    // ── Programmatic writes ───────────────────────────────────────────────
    /// Overwrite `bytes.len()` bytes starting at `addr`. Used by the
    /// inspector modal and any future scripted edit path.
    WriteBytes { addr: u64, bytes: Vec<u8> },

    // ── Inspector ────────────────────────────────────────────────────────
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
    /// Toggle search mode (hex / ASCII).
    ToggleSearchMode,
    /// Navigate to the next match.
    SearchNext,
    /// Navigate to the previous match.
    SearchPrev,
    /// Close the search overlay.
    CloseSearch,

    // ── Pattern list panel ───────────────────────────────────────────────
    /// Show/hide the pattern list panel.
    TogglePatternList,
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
    /// Export all patterns and groups to a text file.
    ExportPatterns,
    /// Import patterns and groups from a text file.
    ImportPatterns,
    /// Async result after exporting.
    PatternsExported(Result<(), String>),
    /// Async result after importing — carries the file content so the handler
    /// can parse it with mutable state access.
    PatternsImported(Result<String, String>),

    // ── Address format ──────────────────────────────────────────────────
    /// Toggle between hex and decimal address display.
    ToggleAddrFormat,

    // ── Copy / Paste ────────────────────────────────────────────────────
    /// Copy the selected byte range as hex text to the clipboard.
    CopySelection,
    /// Read clipboard contents for paste (triggers async clipboard read).
    Paste,
    /// Async result: clipboard contents to paste as hex bytes.
    PasteContent(String),

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
