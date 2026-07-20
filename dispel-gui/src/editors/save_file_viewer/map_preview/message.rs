//! Messages for the map preview component.

use crate::editors::save_file_viewer::map_preview::state::PreviewLayer;

#[derive(Debug, Clone)]
pub enum PreviewMessage {
    /// Canvas panning (pixel delta).
    Pan(f32, f32),
    /// Canvas zoom change — first f32 is the multiplicative factor, last two are
    /// the canvas-local cursor position (NaN when from a toolbar button).
    Zoom(f32, f32, f32),
    /// Fit the view to the full map.
    FitToWindow,
    /// Toggle a display layer.
    LayerToggle(PreviewLayer),
    /// Canvas clicked at canvas-local position (x, y).
    Click(f32, f32),
}
