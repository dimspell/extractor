use dispel_core::snf::SnfFile;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct PlaybackHandle {
    pub sink: Arc<rodio::Sink>,
    stop_flag: Arc<AtomicBool>,
    pub loop_flag: Arc<AtomicBool>,
    _thread: std::thread::JoinHandle<()>,
}

impl PlaybackHandle {
    pub fn new(
        sink: Arc<rodio::Sink>,
        stop_flag: Arc<AtomicBool>,
        loop_flag: Arc<AtomicBool>,
        thread: std::thread::JoinHandle<()>,
    ) -> Self {
        PlaybackHandle {
            sink,
            stop_flag,
            loop_flag,
            _thread: thread,
        }
    }

    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        self.sink.stop();
    }

    pub fn set_looping(&self, v: bool) {
        self.loop_flag.store(v, Ordering::Relaxed);
    }
}

impl Drop for PlaybackHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

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
            },
        }
    }
}
