#[derive(Debug, Clone)]
pub enum SnfEditorMessage {
    Play,
    Pause,
    Stop,
    ToggleLoop,
    SetVolume(f32),
    ExportWav,
    ExportWavDone(Result<String, String>),
    Tick,
}
