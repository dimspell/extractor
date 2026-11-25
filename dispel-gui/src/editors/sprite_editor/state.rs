use crate::editors::sprite_editor::ExportFormat;
use dispel_core::sprite::{SpriteFile, SpriteRenderOptions, read_sprite_file};
use iced::{Point, Size};
use std::path::{Path, PathBuf};

/// Scale that fits a frame inside the preview viewport while preserving its
/// aspect ratio. A zero-sized frame or viewport falls back to 1:1 so the
/// preview cannot collapse before layout has run.
pub fn fit_scale(viewport: Size, frame: (u32, u32)) -> f32 {
    let (w, h) = frame;
    if w == 0 || h == 0 || viewport.width <= 0.0 || viewport.height <= 0.0 {
        return 1.0;
    }
    (viewport.width / w as f32)
        .min(viewport.height / h as f32)
        .max(0.01)
}

/// Largest frame dimensions within one sequence, used as the shared preview
/// canvas so frames of different sizes stay on one center during animation.
/// Returns `(1, 1)` when the sequence has no decoded frames.
pub fn sequence_bounds(frames: &[SpriteFrame], sequence_idx: usize) -> (u32, u32) {
    let mut width = 0u32;
    let mut height = 0u32;
    for frame in frames.iter().filter(|f| f.sequence_idx == sequence_idx) {
        width = width.max(frame.width);
        height = height.max(frame.height);
    }
    (width.max(1), height.max(1))
}

/// Resolved geometry for the preview at a given viewport size.
#[derive(Debug, Clone, Copy)]
pub struct PreviewLayout {
    /// Size of the scrollable inner container: at least the viewport, larger
    /// when the scaled canvas does not fit.
    pub inner_size: Size,
    /// Scaled size of the sequence canvas (bounds × scale).
    pub canvas_size: Size,
    /// Scaled size of the current frame.
    pub frame_size: Size,
    /// Center of the inner container; every frame is placed around this point.
    pub center: Point,
    /// Top-left corner of the current frame within the inner container.
    pub frame_offset: Point,
    /// Uniform scale applied to the canvas and to every frame.
    pub scale: f32,
}

/// Compute preview geometry from the viewport, the sequence canvas bounds and
/// the current frame size. Fit mode scales the whole sequence; manual zoom
/// scales it by `zoom` on top. The frame is always centered on the canvas
/// center, so changing frames never shifts the sprite.
pub fn preview_layout(
    viewport: Size,
    bounds: (u32, u32),
    frame: (u32, u32),
    fit_mode: bool,
    zoom: f32,
) -> PreviewLayout {
    let scale = if fit_mode {
        fit_scale(viewport, bounds)
    } else {
        1.0
    } * zoom;

    let canvas_w = (bounds.0 as f32 * scale).ceil().max(1.0);
    let canvas_h = (bounds.1 as f32 * scale).ceil().max(1.0);
    let frame_w = (frame.0 as f32 * scale).ceil().max(1.0);
    let frame_h = (frame.1 as f32 * scale).ceil().max(1.0);

    let inner_w = canvas_w.max(viewport.width).max(1.0);
    let inner_h = canvas_h.max(viewport.height).max(1.0);
    let center = Point::new(inner_w / 2.0, inner_h / 2.0);
    let frame_offset = Point::new(center.x - frame_w / 2.0, center.y - frame_h / 2.0);

    PreviewLayout {
        inner_size: Size::new(inner_w, inner_h),
        canvas_size: Size::new(canvas_w, canvas_h),
        frame_size: Size::new(frame_w, frame_h),
        center,
        frame_offset,
        scale,
    }
}

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
    /// Show raw `0x0000` pixels as opaque black instead of transparent.
    pub preserve_black: bool,
    // Editing
    pub dirty: bool,
    pub undo_stack: Vec<SpriteFile>,
    pub redo_stack: Vec<SpriteFile>,
    // Zoom
    /// Zoom multiplier (1.0 = 100%, clamped to [0.1, 10.0]).
    pub zoom: f32,
    /// When true the frame is scaled to the live preview viewport instead of
    /// using the fixed `zoom` multiplier.
    pub fit_mode: bool,
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
            preserve_black: false,
            dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            zoom: 1.0,
            fit_mode: false,
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
        let max_frames = self
            .frame_counts
            .get(self.selected_sequence)
            .copied()
            .unwrap_or(0);
        self.selected_frame = frame_idx.min(max_frames.saturating_sub(1));
        self.ms_accumulated = 0.0;
    }

    /// Global index into `self.frames` for `(selected_sequence, selected_frame)`.
    pub fn selected_frame_global(&self) -> usize {
        let offset: usize = self.frame_counts
            [..self.selected_sequence.min(self.frame_counts.len())]
            .iter()
            .sum();
        offset + self.selected_frame
    }

    /// Number of frames in the currently selected sequence.
    pub fn frames_in_sequence(&self) -> usize {
        self.frame_counts
            .get(self.selected_sequence)
            .copied()
            .unwrap_or(0)
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
            let max_frames = self
                .frame_counts
                .get(self.selected_sequence)
                .copied()
                .unwrap_or(0);
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

    /// Rebuild `self.frames` from `self.sprite_file` by decoding all 16-bit
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
                    frame.decode_to_rgba_with_options(SpriteRenderOptions {
                        preserve_black: self.preserve_black,
                        ..Default::default()
                    })
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
        let max_frames = self
            .frame_counts
            .get(self.selected_sequence)
            .copied()
            .unwrap_or(0);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fit_scale_uses_the_limiting_viewport_axis() {
        let scale = fit_scale(Size::new(640.0, 480.0), (32, 32));
        assert!(
            (scale - 15.0).abs() < 1e-5,
            "expected 480/32 = 15, got {scale}"
        );
        let scale = fit_scale(Size::new(100.0, 400.0), (100, 50));
        assert!(
            (scale - 1.0).abs() < 1e-5,
            "expected 100/100 = 1, got {scale}"
        );
    }

    #[test]
    fn test_fit_scale_handles_unlaid_out_viewport() {
        assert_eq!(fit_scale(Size::new(0.0, 0.0), (32, 32)), 1.0);
        assert_eq!(fit_scale(Size::new(100.0, 100.0), (0, 10)), 1.0);
        assert!(fit_scale(Size::new(100.0, 100.0), (10000, 1)) >= 0.01);
        assert!(!SpriteViewerState::default().fit_mode);
    }

    #[test]
    fn test_default_preview_is_native_size() {
        let state = SpriteViewerState::default();
        assert!(!state.fit_mode, "preview starts at 1:1, not Fit");
        assert_eq!(state.zoom, 1.0, "preview starts at native size");
    }

    fn invented_frame(seq: usize, frame: usize, width: u32, height: u32) -> SpriteFrame {
        SpriteFrame {
            sequence_idx: seq,
            frame_idx: frame,
            image: iced::widget::image::Handle::from_rgba(1, 1, vec![0, 0, 0, 0]),
            rgba_bytes: vec![],
            width,
            height,
        }
    }

    #[test]
    fn test_sequence_bounds_use_largest_frame_in_selected_sequence() {
        let frames = vec![
            invented_frame(0, 0, 32, 48),
            invented_frame(0, 1, 64, 24),
            invented_frame(1, 0, 16, 16),
            invented_frame(1, 1, 10, 96),
        ];
        assert_eq!(sequence_bounds(&frames, 0), (64, 48));
        assert_eq!(sequence_bounds(&frames, 1), (16, 96));
        assert_eq!(sequence_bounds(&frames, 2), (1, 1));
    }

    #[test]
    fn test_frames_share_one_center_across_animation() {
        let viewport = Size::new(800.0, 600.0);
        let bounds = (64, 48);
        let small = preview_layout(viewport, bounds, (16, 12), false, 1.0);
        let large = preview_layout(viewport, bounds, (64, 48), false, 1.0);
        assert_eq!(small.center.x, large.center.x);
        assert_eq!(small.center.y, large.center.y);
        assert_eq!(
            small.frame_offset.x + small.frame_size.width / 2.0,
            small.center.x
        );
        assert_eq!(
            large.frame_offset.x + large.frame_size.width / 2.0,
            large.center.x
        );
        assert_eq!(
            small.frame_offset.y + small.frame_size.height / 2.0,
            small.center.y
        );
        assert_eq!(
            large.frame_offset.y + large.frame_size.height / 2.0,
            large.center.y
        );
    }

    #[test]
    fn test_preview_centers_when_smaller_than_viewport() {
        let viewport = Size::new(800.0, 600.0);
        let layout = preview_layout(viewport, (100, 80), (100, 80), false, 1.0);
        assert_eq!(layout.inner_size.width, 800.0);
        assert_eq!(layout.inner_size.height, 600.0);
        assert_eq!(layout.scale, 1.0);
        assert_eq!(layout.frame_offset.x, (800.0 - 100.0) / 2.0);
        assert_eq!(layout.frame_offset.y, (600.0 - 80.0) / 2.0);
    }

    #[test]
    fn test_preview_scrolls_when_larger_than_viewport() {
        let viewport = Size::new(200.0, 150.0);
        let layout = preview_layout(viewport, (400, 300), (400, 300), false, 2.0);
        assert_eq!(layout.scale, 2.0);
        assert_eq!(layout.inner_size.width, 800.0);
        assert_eq!(layout.inner_size.height, 600.0);
        assert_eq!(layout.frame_size.width, 800.0);
        assert_eq!(layout.frame_size.height, 600.0);
        assert_eq!(layout.frame_offset.x, 0.0);
        assert_eq!(layout.frame_offset.y, 0.0);
    }

    #[test]
    fn test_preview_fit_uses_sequence_bounds_not_current_frame() {
        let viewport = Size::new(640.0, 480.0);
        let layout = preview_layout(viewport, (64, 48), (16, 12), true, 1.0);
        assert_eq!(layout.scale, 10.0);
        assert_eq!(layout.frame_size.width, 160.0);
        assert_eq!(layout.frame_size.height, 120.0);
        assert_eq!(layout.inner_size.width, 640.0);
        assert_eq!(layout.inner_size.height, 480.0);
        assert_eq!(layout.frame_offset.x, (640.0 - 160.0) / 2.0);
        assert_eq!(layout.frame_offset.y, (480.0 - 120.0) / 2.0);
    }
}
