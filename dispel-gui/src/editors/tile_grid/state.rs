//! View state for the 2D tile grid canvas.
//!
//! Holds pan/zoom/cursor state for the axis-aligned 2D grid view of a
//! `TileGrid`. This is intentionally simpler than the isometric
//! `MapViewState` — no layer toggles, no sprite handles, no tool system.

use iced::widget::canvas;

/// Viewport state for the tile grid canvas.
#[derive(Debug)]
pub struct TileGridViewState {
    /// Pixel pan offset (canvas translation).
    pub pan_x: f32,
    /// Pixel pan offset (canvas translation).
    pub pan_y: f32,
    /// Zoom factor (1.0 = `CELL_SIZE` pixels per cell).
    pub zoom: f32,
    /// Flat index of the hovered cell (`y * grid_size + x`), or `None`.
    pub hovered_cell: Option<u32>,
    /// Flat index of the selected cell, or `None`.
    pub selected_cell: Option<u32>,
    /// Last known canvas size in pixels (updated from `Resized`).
    pub last_canvas_w: f32,
    pub last_canvas_h: f32,
    /// Cached grid layer. Cleared on pan/zoom/grid-data change.
    pub grid_cache: canvas::Cache,
}

impl Default for TileGridViewState {
    fn default() -> Self {
        Self {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
            hovered_cell: None,
            selected_cell: None,
            last_canvas_w: 800.0,
            last_canvas_h: 600.0,
            grid_cache: canvas::Cache::new(),
        }
    }
}

/// Rendered size of one cell in canvas pixels at zoom = 1.0.
pub const CELL_SIZE: f32 = 8.0;
/// Minimum zoom (cells become very small).
pub const MIN_ZOOM: f32 = 0.25;
/// Maximum zoom (cells become very large).
pub const MAX_ZOOM: f32 = 8.0;

impl TileGridViewState {
    /// Invalidate the grid cache (call when data, pan, or zoom changes).
    pub fn invalidate_cache(&mut self) {
        self.grid_cache = canvas::Cache::new();
    }

    /// Clamp zoom to `[MIN_ZOOM, MAX_ZOOM]`.
    pub fn clamp_zoom(&mut self) {
        self.zoom = self.zoom.clamp(MIN_ZOOM, MAX_ZOOM);
    }

    /// Convert canvas-local pixel coords to a flat cell index.
    /// Returns `None` if the position is outside the grid.
    pub fn pixel_to_cell(
        &self,
        cx: f32,
        cy: f32,
        grid_size: u32,
    ) -> Option<u32> {
        let world_x = (cx - self.pan_x) / (CELL_SIZE * self.zoom);
        let world_y = (cy - self.pan_y) / (CELL_SIZE * self.zoom);
        if world_x < 0.0 || world_y < 0.0 {
            return None;
        }
        let gx = world_x as u32;
        let gy = world_y as u32;
        if gx >= grid_size || gy >= grid_size {
            return None;
        }
        Some(gy * grid_size + gx)
    }

    /// Convert a flat cell index to canvas-local pixel coords (top-left).
    pub fn cell_to_pixel(&self, index: u32, grid_size: u32) -> (f32, f32) {
        let gx = (index % grid_size) as f32;
        let gy = (index / grid_size) as f32;
        (
            gx * CELL_SIZE * self.zoom + self.pan_x,
            gy * CELL_SIZE * self.zoom + self.pan_y,
        )
    }
}
