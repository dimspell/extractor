//! Audio playback for SNF files using rodio.
// Provides SNF audio file loading and playback controls.

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use std::path::Path;

/// Audio player state.
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// SNF audio player.
pub struct AudioPlayer {
    pub hwnd: HWND,
    pub state: PlaybackState,
    pub file_path: Option<std::path::PathBuf>,
    pub volume: f32,
}

impl AudioPlayer {
    pub fn new(parent: HWND) -> Result<Self> {
        Ok(Self {
            hwnd: parent,
            state: PlaybackState::Stopped,
            file_path: None,
            volume: 1.0,
        })
    }

    /// Load an SNF file for playback.
    pub fn load(&mut self, path: &Path) -> Result<()> {
        self.file_path = Some(path.to_path_buf());
        self.state = PlaybackState::Stopped;
        // TODO: Parse SNF file header and prepare for playback
        Ok(())
    }

    /// Start or resume playback.
    pub fn play(&mut self) -> Result<()> {
        // TODO: Start audio playback using rodio or DirectSound
        self.state = PlaybackState::Playing;
        Ok(())
    }

    /// Pause playback.
    pub fn pause(&mut self) {
        self.state = PlaybackState::Paused;
    }

    /// Stop playback.
    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
    }

    /// Set volume (0.0 to 1.0).
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Check if currently playing.
    pub fn is_playing(&self) -> bool {
        matches!(self.state, PlaybackState::Playing)
    }
}
