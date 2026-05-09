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
}
