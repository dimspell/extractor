/// Messages produced by the hex editor.
///
/// Commit 1 only carries scroll/configuration messages; selection, editing
/// and inspector messages land in subsequent commits.
#[derive(Debug, Clone)]
pub enum HexEditorMessage {
    /// User asked to change the row width (8/16/32).
    SetBytesPerRow(u8),
}
