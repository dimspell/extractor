use dispel_core::snf::SnfFile;

#[derive(Debug, Clone)]
pub enum SnfEditorMessage {
    Play,
    Pause,
    Stop,
    ToggleLoop,
    SetVolume(f32),
    ExportWav,
    ExportWavDone(Result<String, String>),
    /// Replace current audio with audio from a WAV file.
    ImportWav,
    /// Async result of ImportWav.
    ImportWavDone(Result<(SnfFile, String), String>),
    /// Save the current audio back to the original .snf file.
    Save,
    /// Async result of Save.
    SaveDone(Result<String, String>),
    /// Auto-dismiss the toast notification.
    ClearToast,
    Tick,
}
