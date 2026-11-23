use crate::components::audio_playback::PlaybackHandle;
use dispel_core::snf::SnfFile;
use rodio::Source;
use std::io::Cursor;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Wav,
    Ogg,
}

pub struct AudioViewerState {
    pub path: PathBuf,
    pub name: String,
    pub format: Option<AudioFormat>,
    pub audio_bytes: Vec<u8>,
    /// Parsed WAV metadata; `None` for OGG, which is decoded directly by rodio.
    pub snf: Option<SnfFile>,
    pub waveform: Vec<(f32, f32)>,
    pub duration_secs: f32,
    pub error: Option<String>,
    pub playback: Option<PlaybackHandle>,
    pub is_looping: bool,
    pub volume: f32,
}

impl AudioViewerState {
    pub fn load_from_path(path: &Path) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let mut state = AudioViewerState {
            path: path.to_path_buf(),
            name,
            format: None,
            audio_bytes: Vec::new(),
            snf: None,
            waveform: Vec::new(),
            duration_secs: 0.0,
            error: None,
            playback: None,
            is_looping: false,
            volume: 0.5,
        };

        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) => {
                state.error = Some(e.to_string());
                return state;
            }
        };

        if bytes.starts_with(b"OggS") {
            state.duration_secs = rodio::Decoder::new(Cursor::new(bytes.clone()))
                .ok()
                .and_then(|decoder| decoder.total_duration())
                .map(|duration| duration.as_secs_f32())
                .unwrap_or(0.0);
            state.format = Some(AudioFormat::Ogg);
            state.audio_bytes = bytes;
            return state;
        }

        match SnfFile::from_wav_bytes(&bytes) {
            Ok(snf) => {
                state.waveform = snf.waveform_points(1000);
                state.duration_secs = snf.duration_secs();
                state.format = Some(AudioFormat::Wav);
                state.snf = Some(snf);
                state.audio_bytes = bytes;
            }
            Err(e) => state.error = Some(format!("Unsupported audio file: {e}")),
        }

        state
    }
}
