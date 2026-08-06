use std::path::{Path, PathBuf};

/// Parameters for recording a sprite-save into an active mod session.
#[derive(Debug, Clone)]
pub struct RecordingParams {
    pub workspace_root: PathBuf,
    pub game_path: PathBuf,
    pub mod_slug: String,
    pub relative_path: String,
}

impl RecordingParams {
    /// Compute the relative path from a game-path-absolute path.
    pub fn relative_path_for(path: &Path, game_path: &Path) -> Option<String> {
        path.strip_prefix(game_path)
            .ok()
            .map(|r| r.to_string_lossy().replace('\\', "/"))
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ExportFormat {
    #[default]
    PngFrames,
    SpriteSheet,
}

#[derive(Debug, Clone)]
pub enum SpriteViewerMessage {
    // ── Navigation ───────────────────────────────────────────────────────────
    SelectSequence(usize),
    SelectFrame(usize),
    /// Scrub the timeline to a specific frame (also pauses playback).
    ScrubTo(usize),
    // ── Playback ─────────────────────────────────────────────────────────────
    Play,
    Pause,
    StepBack,
    StepForward,
    ToggleLoop,
    /// Set playback speed multiplier (0.25 / 0.5 / 1.0 / 2.0).
    SetSpeed(u32),
    /// Animation clock tick — fired by the iced time subscription.
    Tick,
    // ── Zoom ──────────────────────────────────────────────────────────────────
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ZoomToFit,
    // ── Editing ──────────────────────────────────────────────────────────────
    /// Save the current sprite file to disk.
    Save,
    /// Save completed (async result).
    SaveComplete(Result<String, String>),
    /// Undo last edit.
    Undo,
    /// Redo last undone edit.
    Redo,
    /// Insert a blank frame at the current position.
    InsertFrame,
    /// Duplicate the selected frame after it.
    DuplicateFrame,
    /// Delete the selected frame.
    DeleteFrame,
    /// Move the selected frame one position left.
    MoveFrameLeft,
    /// Move the selected frame one position right.
    MoveFrameRight,
    /// Move the selected frame to the start of the sequence.
    MoveFrameToStart,
    /// Move the selected frame to the end of the sequence.
    MoveFrameToEnd,
    // ── PNG import ───────────────────────────────────────────────────────────
    /// Import a PNG file as a new frame (insert after current).
    ImportPngFrame,
    /// Import a PNG file replacing the selected frame.
    ImportPngReplace,
    /// Result of the file-picker + decode step.
    PngImportReady(Result<(Vec<u8>, u32, u32), String>),
    // ── Export dialog ────────────────────────────────────────────────────────
    ShowExportDialog,
    CloseExportDialog,
    SetExportFormat(ExportFormat),
    ChooseExportDir,
    ExportDirChosen(Option<PathBuf>),
    ExportConfirm,
    ExportDone(Result<String, String>),
}
