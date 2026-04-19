use std::path::PathBuf;

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
    SetSpeed(u32), // stored as 100× to avoid f32 in messages (100=1×, 200=2×, …)
    /// Animation clock tick — fired by the iced time subscription.
    Tick,
    // ── Export dialog ────────────────────────────────────────────────────────
    ShowExportDialog,
    CloseExportDialog,
    SetExportFormat(ExportFormat),
    ChooseExportDir,
    ExportDirChosen(Option<PathBuf>),
    ExportConfirm,
    ExportDone(Result<String, String>),
}
