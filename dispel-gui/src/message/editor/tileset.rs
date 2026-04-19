use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum TileExportFormat {
    /// One PNG file per tile, written into a subdirectory.
    #[default]
    SeparateTiles,
    /// All tiles packed into a single atlas PNG.
    Atlas,
}

#[derive(Debug, Clone)]
pub enum TilesetEditorMessage {
    SetZoom(f32),
    // ── Export dialog ────────────────────────────────────────────────────────
    ShowExportDialog,
    CloseExportDialog,
    SetExportFormat(TileExportFormat),
    ChooseExportDir,
    ExportDirChosen(Option<PathBuf>),
    ExportConfirm,
    ExportDone(Result<String, String>),
}
