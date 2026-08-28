//! Update handler for the tile grid canvas view.

use super::message::TileGridCanvasMessage;
use super::state::TileGridViewState;
use dispel_core::references::tile_grid::{TileDescriptor, TileGrid};

/// Handle a `TileGridCanvasMessage`. Returns a status string for the
/// inspector panel (e.g. "Selected cell (42, 17)") or `None` for silent ops.
pub fn handle(
    msg: TileGridCanvasMessage,
    view: &mut TileGridViewState,
    grid: &mut TileGrid,
) -> Option<String> {
    match msg {
        TileGridCanvasMessage::Pan(dx, dy) => {
            view.pan_x += dx;
            view.pan_y += dy;
            view.invalidate_cache();
            None
        }
        TileGridCanvasMessage::Zoom(factor) => {
            let old_zoom = view.zoom;
            view.zoom *= factor;
            view.clamp_zoom();
            if (view.zoom - old_zoom).abs() > f32::EPSILON {
                view.invalidate_cache();
            }
            None
        }
        TileGridCanvasMessage::SetZoom(z) => {
            view.zoom = z.clamp(super::state::MIN_ZOOM, super::state::MAX_ZOOM);
            view.invalidate_cache();
            None
        }
        TileGridCanvasMessage::Click(cx, cy) => {
            let gs = grid.grid_size;
            if let Some(idx) = view.pixel_to_cell(cx, cy, gs) {
                view.selected_cell = Some(idx);
                let gx = idx % gs;
                let gy = idx / gs;
                return Some(format!("Selected cell ({gx}, {gy})"));
            }
            view.selected_cell = None;
            None
        }
        TileGridCanvasMessage::Hover(cx, cy) => {
            let gs = grid.grid_size;
            view.hovered_cell = view.pixel_to_cell(cx, cy, gs);
            None
        }
        TileGridCanvasMessage::Resize(w, h) => {
            view.last_canvas_w = w;
            view.last_canvas_h = h;
            view.invalidate_cache();
            None
        }
        TileGridCanvasMessage::FitToWindow => {
            let gs = grid.grid_size as f32;
            let cell_total = gs * super::state::CELL_SIZE;
            let zoom_x = view.last_canvas_w / cell_total;
            let zoom_y = view.last_canvas_h / cell_total;
            view.zoom = zoom_x.min(zoom_y).clamp(super::state::MIN_ZOOM, super::state::MAX_ZOOM);
            view.pan_x = (view.last_canvas_w - cell_total * view.zoom) * 0.5;
            view.pan_y = (view.last_canvas_h - cell_total * view.zoom) * 0.5;
            view.invalidate_cache();
            None
        }
        TileGridCanvasMessage::EditSelected(new_desc) => {
            if let Some(idx) = view.selected_cell {
                let word = encode_descriptor(new_desc);
                if (idx as usize) < grid.data.len() {
                    grid.data[idx as usize] = word;
                    view.invalidate_cache();
                    return Some(format!("Edited cell {idx}"));
                }
            }
            Some("No cell selected".to_string())
        }
        TileGridCanvasMessage::ClearSelection => {
            view.selected_cell = None;
            None
        }
        TileGridCanvasMessage::ToggleRawView => None, // handled by view
    }
}

/// Pack a `TileDescriptor` back into a `u32` word using the file's bit layout.
pub fn encode_descriptor(d: TileDescriptor) -> u32 {
    (d.object_id as u32 & 0x03ff)
        | ((d.tile_index as u32 & 0x7fff) << 10)
        | ((d.render_flags as u32 & 0x03) << 25)
        | ((d.reserved as u32 & 0x1f) << 27)
}
