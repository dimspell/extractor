//! View state for the 2D tile grid canvas.
//!
//! Holds pan/zoom/cursor state for the axis-aligned 2D grid view of a
//! `TileGrid`. This is intentionally simpler than the isometric
//! `MapViewState` — no layer toggles, no sprite handles, no tool system.

use iced::widget::canvas;
use iced::widget::pane_grid;
use std::collections::HashMap;

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
    /// `true` until the first `FitToWindow` has been applied after a grid load.
    /// Used by the canvas view to auto-fit on first render.
    // TODO(bug: no-auto-fit-on-catalog-load): needs_fit is set to `true` in
    // Default, but the render path can't mutate state (view is `&App`,
    // immutable). The flag is only cleared by explicit `FitToWindow` or
    // `Resize` events. Loading a `.dat` file requires the user to click
    // "Fit" or resize the window before the grid is visible. Fix by
    // dispatching `FitToWindow` from the standard editor's `CatalogLoaded`
    // handler — requires adding a post-load hook to `define_standard_editor!`.
    pub needs_fit: bool,
    /// Last status message from a canvas operation (e.g. "Edited cell 5").
    /// Shown in the inspector panel.
    pub last_status: Option<String>,
    /// Resizable pane grid layout (canvas | inspector).
    pub pane_state: Option<pane_grid::State<PaneContent>>,
    /// Last focused pane in the pane grid.
    pub pane_focus: Option<pane_grid::Pane>,
    /// Per-field edit buffers. Keyed by field name (e.g. "object_id").
    /// When `Some`, the inspector shows the buffer text instead of the live
    /// `grid.data` value, so the user can type "999" and see "999" before
    /// it gets clamped on commit.
    pub edit_buffers: HashMap<String, String>,
}

/// Identifies which pane the user is looking at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneContent {
    /// The 2D canvas.
    Canvas,
    /// The inspector panel.
    Inspector,
}

impl Default for TileGridViewState {
    fn default() -> Self {
        let (mut pane_state, canvas_pane) =
            pane_grid::State::new(PaneContent::Canvas);
        // Split the canvas pane vertically to place the inspector below
        // the canvas. In `pane_grid::Axis`, `Vertical` means the split
        // line runs top-to-bottom, so the new pane appears beneath.
        let (inspector_pane, split) = pane_state
            .split(pane_grid::Axis::Vertical, canvas_pane, PaneContent::Inspector)
            .expect("split on fresh pane_grid should always succeed");
        // Set the split so the inspector (bottom pane) takes ~30% of the
        // vertical space, with the canvas taking the remaining ~70%.
        // The ratio is the fraction of space given to the *first* child
        // (the canvas), so 0.7 = 70% canvas, 30% inspector.
        pane_state.resize(split, 0.7);
        let _ = inspector_pane;
        Self {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
            hovered_cell: None,
            selected_cell: None,
            last_canvas_w: 800.0,
            last_canvas_h: 600.0,
            grid_cache: canvas::Cache::new(),
            needs_fit: true,
            last_status: None,
            pane_state: Some(pane_state),
            pane_focus: None,
            edit_buffers: HashMap::new(),
        }
    }
}

/// Rendered size of one cell in canvas pixels at zoom = 1.0.
pub const CELL_SIZE: f32 = 8.0;
/// Minimum zoom (cells become very small).
pub const MIN_ZOOM: f32 = 0.25;
/// Maximum zoom (cells become very large).
pub const MAX_ZOOM: f32 = 8.0;
/// Fraction of the full window width occupied by the canvas pane.
/// The remaining fraction is taken by the inspector pane beside it.
/// Must match the split ratio set in `Default`.
pub const CANVAS_WIDTH_RATIO: f32 = 0.7;

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
