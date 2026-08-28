//! Messages for the tile grid canvas view.
//!
//! These are handled by `update::handle` and routed via `dispatch_table.rs`.

use dispel_core::references::tile_grid::TileDescriptor;

/// User actions on the tile grid 2D canvas.
#[derive(Debug, Clone)]
pub enum TileGridCanvasMessage {
    /// Pan the canvas by (dx, dy) in screen pixels.
    Pan(f32, f32),
    /// Zoom by a multiplicative factor (e.g. 1.1 = zoom in 10%).
    Zoom(f32),
    /// Set zoom to a specific value (clamped to `[MIN_ZOOM, MAX_ZOOM]`).
    SetZoom(f32),
    /// User clicked at canvas-local pixel coords; selects the cell under the cursor.
    Click(f32, f32),
    /// User moved the cursor; updates the hovered cell.
    Hover(f32, f32),
    /// Canvas was resized.
    Resize(f32, f32),
    /// Fit the entire grid into the current canvas size.
    FitToWindow,
    /// Edit the currently selected cell's descriptor fields.
    EditSelected(TileDescriptor),
    /// Clear the current selection.
    ClearSelection,
    /// Toggle between canvas view and raw text view.
    ToggleRawView,
}
