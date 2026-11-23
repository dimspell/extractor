use dispel_core::snf::SnfFile;
use std::path::{Path, PathBuf};

pub use crate::components::audio_playback::PlaybackHandle;

#[derive(Debug, Clone)]
pub enum ExportStatus {
    Idle,
    Done(String),
    Error(String),
}

pub struct SnfEditorState {
    pub path: PathBuf,
    pub name: String,
    pub snf: Option<SnfFile>,
    pub waveform: Vec<(f32, f32)>,
    pub error: Option<String>,
    pub playback: Option<PlaybackHandle>,
    pub is_looping: bool,
    pub volume: f32,
    pub export_status: ExportStatus,
    pub modified: bool,
    /// Active toast notifications displayed as overlay.
    pub toasts: Vec<gui_widgets::components::toast::Toast>,
}

impl SnfEditorState {
    pub fn load_from_path(path: &Path) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        match dispel_core::snf::read(path) {
            Ok(snf) => {
                let waveform = snf.waveform_points(1000);
                SnfEditorState {
                    path: path.to_path_buf(),
                    name,
                    snf: Some(snf),
                    waveform,
                    error: None,
                    playback: None,
                    is_looping: false,
                    volume: 0.5,
                    export_status: ExportStatus::Idle,
                    modified: false,
                    toasts: Vec::new(),
                }
            }
            Err(e) => SnfEditorState {
                path: path.to_path_buf(),
                name,
                snf: None,
                waveform: Vec::new(),
                error: Some(e.to_string()),
                playback: None,
                is_looping: false,
                volume: 0.5,
                export_status: ExportStatus::Idle,
                modified: false,
                toasts: Vec::new(),
            },
        }
    }
}
