use crate::components::audio_playback::PendingPlayback;

#[derive(Debug, Clone)]
pub enum AudioViewerMessage {
    Play,
    /// Async result of the deferred audio-device open, tagged with the tab and
    /// playback generation that issued the request. A generation that no longer
    /// matches is stale (Stop or tab switch won in the meantime) and is ignored.
    PlaybackReady(usize, u64, Result<PendingPlayback, String>),
    Pause,
    Stop,
    ToggleLoop,
    SetVolume(f32),
    Tick,
}
