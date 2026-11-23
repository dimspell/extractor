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
    /// Save the current audio back to the original .snf file. The tab id is
    /// supplied by the editor descriptor that issued the save.
    Save(usize),
    /// Async result of Save, tagged with the tab that issued it.
    SaveDone(usize, Result<String, String>),
    /// Dismiss a toast notification by index (triggered by close button or timeout).
    DismissToast(usize),
    Tick,
}
