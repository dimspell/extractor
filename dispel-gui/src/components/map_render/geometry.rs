// ── Coordinate transforms for the isometric map canvas ───────────────────────

use crate::components::map_render::{TILE_H, TILE_W};
use iced::Rectangle;

/// World pixel centre of an isometric tile.
pub fn tile_world_center(tx: i32, ty: i32, diagonal: i32) -> (f32, f32) {
    let (px, py) = dispel_core::map::types::convert_map_coords_to_image_coords(tx, ty, diagonal);
    (px as f32 + TILE_W * 0.5, py as f32 + TILE_H * 0.5)
}

/// Screen-space centre of a tile bounding box at the given zoom.
/// `px`, `py` are the top-left corner returned by `tile_to_screen`.
#[inline]
pub fn tile_center(px: f32, py: f32, zoom: f32) -> (f32, f32) {
    (px + TILE_W * zoom * 0.5, py + TILE_H * zoom * 0.5)
}

/// Returns true if the point (px, py) in canvas-local coords is inside the
/// isometric diamond of the tile at screen position (tile_screen_x, tile_screen_y).
pub fn point_in_tile_diamond(
    px: f32,
    py: f32,
    tile_screen_x: f32,
    tile_screen_y: f32,
    zoom: f32,
) -> bool {
    let cx = tile_screen_x + TILE_W * zoom * 0.5;
    let cy = tile_screen_y + TILE_H * zoom * 0.5;
    let dx = (px - cx).abs() / (TILE_W * zoom * 0.5);
    let dy = (py - cy).abs() / (TILE_H * zoom * 0.5);
    dx + dy <= 1.0
}

/// Convert tile coordinates to canvas-local screen coordinates.
pub fn tile_to_screen(
    tx: i32,
    ty: i32,
    diagonal: i32,
    pan_x: f32,
    pan_y: f32,
    zoom: f32,
) -> (f32, f32) {
    let (px, py) = dispel_core::map::types::convert_map_coords_to_image_coords(tx, ty, diagonal);
    (px as f32 * zoom + pan_x, py as f32 * zoom + pan_y)
}

/// Convert canvas-local cursor coords to tile coords by checking the diamond
/// hit-test against the approximate tile and its 8 neighbours.
///
/// The raw inverse formula `(tx,ty) = f(world_x, world_y)` gives the tile whose
/// *top-left corner* is closest to the cursor.  Because the diamond centre sits
/// at `(+31, +16)` from the top-left but the isometric grid step is `(32, 16)`,
/// rounding alone systematically picks the wrong tile for most points inside the
/// diamond.  Checking neighbours with the diamond test fixes this.
#[allow(clippy::too_many_arguments)]
pub fn screen_to_tile(
    cx: f32,
    cy: f32,
    diagonal: i32,
    pan_x: f32,
    pan_y: f32,
    zoom: f32,
    model_w: i32,
    model_h: i32,
) -> Option<(i32, i32)> {
    let world_x = (cx - pan_x) / zoom;
    let world_y = (cy - pan_y) / zoom;
    let a = world_x / 32.0;
    let b = (world_y - (diagonal as f32 / 2.0 * 16.0)) / 16.0;
    let approx_tx = ((a - b) / 2.0).round() as i32;
    let approx_ty = ((a + b) / 2.0).round() as i32;

    for dy in -1..=1i32 {
        for dx in -1..=1i32 {
            let test_tx = approx_tx + dx;
            let test_ty = approx_ty + dy;
            if test_tx < 0 || test_tx >= model_w || test_ty < 0 || test_ty >= model_h {
                continue;
            }
            let (sx, sy) = tile_to_screen(test_tx, test_ty, diagonal, pan_x, pan_y, zoom);
            if point_in_tile_diamond(cx, cy, sx, sy, zoom) {
                return Some((test_tx, test_ty));
            }
        }
    }
    None
}

/// Returns true if the rectangle overlaps the visible canvas area (canvas-local coords).
pub fn is_visible(x: f32, y: f32, w: f32, h: f32, bounds: Rectangle) -> bool {
    x + w > 0.0 && x < bounds.width && y + h > 0.0 && y < bounds.height
}
