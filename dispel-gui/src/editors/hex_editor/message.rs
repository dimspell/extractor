use super::selection::NavDir;

/// Messages produced by the hex editor.
///
/// Commit 2 adds selection messages on top of the original SetBytesPerRow.
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
}
