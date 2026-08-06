use iced::Point;

/// Per-canvas interaction state (managed by Iced).
#[derive(Default)]
pub struct MapCanvasState {
    pub is_dragging: bool,
    /// Canvas-local drag anchor, set in position_in coordinates.
    pub drag_last: Option<Point>,
    /// Canvas-local press position used to distinguish click from drag.
    pub drag_start: Option<Point>,
}

/// A decoded internal-map sprite ready to draw on the canvas.
#[derive(Clone)]
pub struct InternalSpriteHandle {
    /// Pixel x in the full (non-occluded) image space.
    pub x: i32,
    /// Pixel y in the full (non-occluded) image space.
    pub y: i32,
    /// Y-sort key for interlaced rendering (`sprite_bottom_right_y` from the file).
    pub sort_y: i32,
    pub handle: iced::widget::image::Handle,
    pub width: u32,
    pub height: u32,
}

/// A decoded entity sprite (NPC / monster / extra) ready to draw.
#[derive(Clone)]
pub struct EntitySpriteHandle {
    pub handle: iced::widget::image::Handle,
    pub width: u32,
    pub height: u32,
    pub origin_x: i32,
    pub origin_y: i32,
    pub flip: bool,
}
