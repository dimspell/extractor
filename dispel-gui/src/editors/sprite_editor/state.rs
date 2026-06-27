use crate::editors::sprite_editor::ExportFormat;
use dispel_core::sprite::{read_sprite_file, SpriteFile};
use std::path::{Path, PathBuf};

/// A decoded sprite frame ready for display, editing, and export.
#[derive(Debug, Clone)]
pub struct SpriteFrame {
    pub sequence_idx: usize,
    pub frame_idx: usize,
    /// Handle used by the iced image widget (from `Handle::from_rgba`).
    pub image: iced::widget::image::Handle,
    /// RGBA pixel data (width × height × 4 bytes), used for PNG export.
    pub rgba_bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
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

// ── Viewer / editor state ─────────────────────────────────────────────────────

/// State for one sprite file editor tab.
#[derive(Debug, Clone)]
pub struct SpriteViewerState {
    // Identity
    pub path: PathBuf,
    pub save_path: PathBuf,
    pub name: String,
    // Sprite data
    pub sprite_file: Option<SpriteFile>,
    pub sequence_count: usize,
    pub frame_counts: Vec<usize>,
    pub selected_sequence: usize,
    pub selected_frame: usize,
    pub frames: Vec<SpriteFrame>,
    pub error: Option<String>,
    // Editing
    pub dirty: bool,
    pub undo_stack: Vec<SpriteFile>,
    pub redo_stack: Vec<SpriteFile>,
    // Zoom
    /// Zoom multiplier (1.0 = 100%, clamped to [0.1, 10.0]).
    pub zoom: f32,
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
            save_path: PathBuf::new(),
            name: String::new(),
            sprite_file: None,
            sequence_count: 0,
            frame_counts: Vec::new(),
            selected_sequence: 0,
            selected_frame: 0,
            frames: Vec::new(),
            error: None,
            dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            zoom: 1.0,
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

        match read_sprite_file(path) {
            Ok(sprite_file) => {
                let mut state = Self {
                    path: path.to_path_buf(),
                    save_path: path.to_path_buf(),
                    name,
                    sprite_file: Some(sprite_file),
                    ..Default::default()
                };
                state.rebuild_preview_frames();
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
        if seq_idx >= self.sequence_count {
            return;
        }
        self.selected_sequence = seq_idx;
        self.selected_frame = 0;
        self.is_playing = false;
        self.ms_accumulated = 0.0;
    }

    pub fn select_frame(&mut self, frame_idx: usize) {
        let max_frames = self.frame_counts.get(self.selected_sequence).copied().unwrap_or(0);
        self.selected_frame = frame_idx.min(max_frames.saturating_sub(1));
        self.ms_accumulated = 0.0;
    }

    /// Global index into `self.frames` for `(selected_sequence, selected_frame)`.
    pub fn selected_frame_global(&self) -> usize {
        let offset: usize = self.frame_counts[..self.selected_sequence.min(self.frame_counts.len())]
            .iter()
            .sum();
        offset + self.selected_frame
    }

    /// Number of frames in the currently selected sequence.
    pub fn frames_in_sequence(&self) -> usize {
        self.frame_counts.get(self.selected_sequence).copied().unwrap_or(0)
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
            let max_frames = self.frame_counts.get(self.selected_sequence).copied().unwrap_or(0);
            if max_frames == 0 {
                return;
            }
            let next = self.selected_frame + 1;
            if next >= max_frames {
                if self.is_looping {
                    self.selected_frame = 0;
                } else {
                    self.selected_frame = max_frames - 1;
                    self.is_playing = false;
                    self.ms_accumulated = 0.0;
                    break;
                }
            } else {
                self.selected_frame = next;
            }
        }
    }

    /// Rebuild `self.frames` from `self.sprite_file` by decoding all RGB565
    /// pixel data to RGBA and creating iced image handles.
    ///
    /// Call this after any edit to `sprite_file` to keep the display in sync.
    pub fn rebuild_preview_frames(&mut self) {
        self.frames.clear();
        let Some(ref sf) = self.sprite_file else {
            self.sequence_count = 0;
            self.frame_counts.clear();
            return;
        };

        self.sequence_count = sf.sequences.len();
        self.frame_counts = sf.sequences.iter().map(|s| s.frames.len()).collect();

        for (seq_idx, seq) in sf.sequences.iter().enumerate() {
            for (frame_idx, frame) in seq.frames.iter().enumerate() {
                let w = frame.width.max(0) as u32;
                let h = frame.height.max(0) as u32;
                let rgba = if w > 0 && h > 0 {
                    frame.decode_to_rgba()
                } else {
                    Vec::new()
                };
                let image = if rgba.len() >= 4 {
                    iced::widget::image::Handle::from_rgba(w, h, rgba.clone())
                } else {
                    iced::widget::image::Handle::from_rgba(1, 1, vec![0, 0, 0, 0])
                };
                self.frames.push(SpriteFrame {
                    sequence_idx: seq_idx,
                    frame_idx,
                    image,
                    rgba_bytes: rgba,
                    width: w,
                    height: h,
                });
            }
        }

        // Clamp selection
        if self.selected_sequence >= self.sequence_count {
            self.selected_sequence = self.sequence_count.saturating_sub(1);
        }
        let max_frames = self.frame_counts.get(self.selected_sequence).copied().unwrap_or(0);
        self.selected_frame = self.selected_frame.min(max_frames.saturating_sub(1));
    }

    // ── Undo/redo ─────────────────────────────────────────────────────────

    /// Push the current `sprite_file` onto the undo stack (before an edit).
    /// Clears the redo stack (new edit invalidates redo history).
    pub fn push_undo(&mut self) {
        if let Some(ref sf) = self.sprite_file {
            self.undo_stack.push(sf.clone());
            self.redo_stack.clear();
            // Cap undo stack at 50 entries
            if self.undo_stack.len() > 50 {
                self.undo_stack.remove(0);
            }
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self) {
        let Some(sf) = self.undo_stack.pop() else {
            return;
        };
        if let Some(current) = self.sprite_file.take() {
            self.redo_stack.push(current);
        }
        self.sprite_file = Some(sf);
        self.dirty = !self.undo_stack.is_empty();
        self.rebuild_preview_frames();
    }

    pub fn redo(&mut self) {
        let Some(sf) = self.redo_stack.pop() else {
            return;
        };
        if let Some(current) = self.sprite_file.take() {
            self.undo_stack.push(current);
        }
        self.sprite_file = Some(sf);
        self.dirty = true;
        self.rebuild_preview_frames();
    }

    // ── Dirty state ───────────────────────────────────────────────────────

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
}
