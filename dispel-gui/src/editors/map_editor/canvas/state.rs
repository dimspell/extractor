// ── Interaction state ─────────────────────────────────────────────────────────

use iced::Point;

/// Per-canvas interaction state (managed by Iced).
#[derive(Default)]
pub struct MapCanvasState {
    pub is_dragging: bool,
    /// Canvas-local drag anchor, set in position_in coordinates.
    pub drag_last: Option<Point>,
    /// Canvas-local press position used to distinguish click from drag.
    pub drag_start: Option<Point>,
    /// Entity currently under the cursor (for hover highlight + pointer cursor).
    pub hovered_entity: Option<crate::editors::map_editor::message::SelectedEntity>,
}
