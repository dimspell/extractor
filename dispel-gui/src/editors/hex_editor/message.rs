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
    /// Clear all patterns.
    ClearAllPatterns,
    /// Right-click at a specific address — used to determine which context
    /// menu options to show (remove pattern vs create pattern).
    RightClickAt(u64),

    // ── Goto address ────────────────────────────────────────────────────
    /// Open the goto-address dialog.
    OpenGotoDialog,
    /// Update the input draft.
    SetGotoDraft(String),
    /// Parse and navigate.
    CommitGoto,
    /// Dismiss the dialog.
    CloseGotoDialog,

    // ── Search & Find/Replace ───────────────────────────────────────────
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
    /// Replace the current match with the given replacement bytes.
    ReplaceOne(String),
    /// Show the replace-all confirmation dialog.
    ShowReplaceConfirm(String),
    /// Set the replace query (for live input without triggering replace).
    SetReplaceQuery(String),
    /// Confirm replace-all.
    CommitReplaceAll,
    /// Dismiss replace-all confirmation.
    CancelReplaceAll,
    /// Close the search overlay.
    CloseSearch,

    // ── Pattern list panel ───────────────────────────────────────────────
    /// Show/hide the pattern list panel.
    TogglePatternList,
    /// Navigate to a pattern's start address.
    NavigateToPattern(usize),
    /// Remove a pattern by its id.
    RemovePattern(usize),
}
