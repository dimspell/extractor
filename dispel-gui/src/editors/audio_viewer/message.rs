use crate::components::audio_playback::PendingPlayback;

#[derive(Debug, Clone)]
pub enum AudioViewerMessage {
    Play,
    /// Async result of the deferred audio-device open, tagged with the tab that
    /// issued the play request.
    PlaybackReady(usize, Result<PendingPlayback, String>),
    Pause,
    Stop,
    ToggleLoop,
    SetVolume(f32),
    Tick,
}
