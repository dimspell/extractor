//! Messages for the tile grid canvas view.
//!
//! These are handled by `update::handle` and routed via `dispatch_table.rs`.

use dispel_core::references::tile_grid::TileDescriptor;

/// User actions on the tile grid 2D canvas.
#[derive(Debug, Clone)]
pub enum TileGridCanvasMessage {
    /// Pan the canvas by (dx, dy) in screen pixels.
    Pan(f32, f32),
    /// Zoom by a multiplicative factor, keeping the world point under the
    /// cursor at the same screen position. `(factor, cursor_x, cursor_y)`.
    Zoom(f32, f32, f32),
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
    /// Inspector pane was resized by the user.
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    /// Inspector pane was clicked/focused.
    PaneClicked(iced::widget::pane_grid::Pane),
    /// Update the local edit buffer for a field without committing to
    /// `grid.data`. Used so the user can type "999" and see "999" before
    /// it gets clamped on commit (Enter/blur).
    EditFieldBuffer {
        field: String,
        value: String,
    },
    /// Commit the edit buffer for a field to `grid.data` (called on Enter
    /// or blur). If the buffer is empty or invalid, the field is ignored.
    CommitFieldBuffer {
        field: String,
    },
}
