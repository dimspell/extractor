use dispel_core::sprite::get_sequence_pngs_by_index;
use std::path::{Path, PathBuf};

use crate::message::editor::spritebrowser::ExportFormat;

/// A decoded sprite frame ready for display and export.
#[derive(Debug, Clone)]
pub struct SpriteFrame {
    pub sequence_idx: usize,
    pub frame_idx: usize,
    /// Handle used by the iced image widget.
    pub image: iced::widget::image::Handle,
    /// Original PNG bytes, used for export.
    pub png_bytes: Vec<u8>,
}

// ── Export dialog ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ExportStatus {
    #[default]
    Idle,
    Exporting,
    Done(String),
    Error(String),
}

#[derive(Debug, Clone, Default)]
pub struct ExportDialogState {
    pub format: ExportFormat,
    pub export_dir: Option<PathBuf>,
    pub status: ExportStatus,
}

// ── Viewer state ──────────────────────────────────────────────────────────────

/// State for one sprite file viewer tab.
#[derive(Debug, Clone)]
pub struct SpriteViewerState {
    // Identity
    pub path: PathBuf,
    pub name: String,
    // Sprite data
    pub sequence_count: usize,
    pub frame_counts: Vec<usize>,
    pub selected_sequence: usize,
    pub selected_frame: usize,
    pub frames: Vec<SpriteFrame>,
    pub error: Option<String>,
    // Playback
    pub is_playing: bool,
    pub is_looping: bool,
    /// Speed multiplier stored as 100× integer (100 = 1×, 200 = 2×, etc.)
    pub speed_100x: u32,
    /// Frames per second at 1× speed.
    pub fps: f32,
    /// Accumulated playback time in milliseconds (resets each frame advance).
    pub ms_accumulated: f32,
    // Export dialog (None = closed)
    pub export_dialog: Option<ExportDialogState>,
}

impl Default for SpriteViewerState {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            name: String::new(),
            sequence_count: 0,
            frame_counts: Vec::new(),
            selected_sequence: 0,
            selected_frame: 0,
            frames: Vec::new(),
            error: None,
            is_playing: false,
            is_looping: true,
            speed_100x: 100,
            fps: 10.0,
            ms_accumulated: 0.0,
            export_dialog: None,
        }
    }
}

impl SpriteViewerState {
    pub fn load_from_path(path: &Path) -> Self {
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        match dispel_core::sprite::get_sprite_metadata(path) {
            Ok((sequence_count, frame_counts)) => {
                let mut state = Self {
                    path: path.to_path_buf(),
                    name,
                    sequence_count,
                    frame_counts,
                    ..Default::default()
                };
                state.load_frames();
                state
            }
            Err(e) => Self {
                path: path.to_path_buf(),
                name,
                error: Some(e.to_string()),
                ..Default::default()
            },
        }
    }

    pub fn select_sequence(&mut self, seq_idx: usize) {
        self.selected_sequence = seq_idx;
        self.selected_frame = 0;
        self.is_playing = false;
        self.ms_accumulated = 0.0;
        self.load_frames();
    }

    pub fn select_frame(&mut self, frame_idx: usize) {
        self.selected_frame = frame_idx.min(self.frames.len().saturating_sub(1));
        self.ms_accumulated = 0.0;
    }

    /// Returns the playback speed as an `f32` multiplier (e.g. 1.0, 2.0).
    pub fn speed(&self) -> f32 {
        self.speed_100x as f32 / 100.0
    }

    /// Advance the animation by `delta_ms` real-time milliseconds.
    /// Called on every clock tick when `is_playing` is true.
    pub fn tick(&mut self, delta_ms: f32) {
        if !self.is_playing || self.frames.len() <= 1 {
            return;
        }
        let frame_ms = 1000.0 / self.fps;
        self.ms_accumulated += delta_ms * self.speed();

        while self.ms_accumulated >= frame_ms {
            self.ms_accumulated -= frame_ms;
            let next = self.selected_frame + 1;
            if next >= self.frames.len() {
                if self.is_looping {
                    self.selected_frame = 0;
                } else {
                    self.selected_frame = self.frames.len() - 1;
                    self.is_playing = false;
                    self.ms_accumulated = 0.0;
                    break;
                }
            } else {
                self.selected_frame = next;
            }
        }
    }

    fn load_frames(&mut self) {
        self.frames.clear();
        if let Ok(pngs) = get_sequence_pngs_by_index(&self.path, self.selected_sequence) {
            for (i, png_buf) in pngs.into_iter().enumerate() {
                self.frames.push(SpriteFrame {
                    sequence_idx: self.selected_sequence,
                    frame_idx: i,
                    image: iced::widget::image::Handle::from_bytes(png_buf.clone()),
                    png_bytes: png_buf,
                });
            }
        }
    }
}
