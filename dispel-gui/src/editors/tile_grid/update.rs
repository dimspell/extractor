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
    let status = handle_inner(msg, view, grid);
    if let Some(ref s) = status {
        view.last_status = Some(s.clone());
    }
    status
}

fn handle_inner(
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
        TileGridCanvasMessage::Zoom(factor, cx, cy) => {
            let old_zoom = view.zoom;
            let new_zoom =
                (view.zoom * factor).clamp(super::state::MIN_ZOOM, super::state::MAX_ZOOM);
            if (new_zoom - old_zoom).abs() > f32::EPSILON {
                // Zoom-to-cursor: keep the world point under (cx, cy) stationary.
                // world_x = (cx - pan_x) / (CELL_SIZE * old_zoom)
                // pan_x_new = cx - world_x * (CELL_SIZE * new_zoom)
                let cell_old = super::state::CELL_SIZE * old_zoom;
                let cell_new = super::state::CELL_SIZE * new_zoom;
                if cell_old > 0.0 {
                    let world_x = (cx - view.pan_x) / cell_old;
                    let world_y = (cy - view.pan_y) / cell_old;
                    view.pan_x = cx - world_x * cell_new;
                    view.pan_y = cy - world_y * cell_new;
                }
                view.zoom = new_zoom;
                view.invalidate_cache();
            }
            None
        }
        TileGridCanvasMessage::SetZoom(z) => {
            view.zoom = z.clamp(super::state::MIN_ZOOM, super::state::MAX_ZOOM);
            view.invalidate_cache();
            Some(format!("Zoom set to {:.2}×", view.zoom))
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
            let was_default = view.last_canvas_w == 0.0 || view.last_canvas_w == 800.0;
            view.last_canvas_w = w;
            view.last_canvas_h = h;
            view.invalidate_cache();
            // Auto-fit on first real resize if we still need to fit.
            if was_default && view.needs_fit && w > 0.0 && h > 0.0 {
                return handle_inner(TileGridCanvasMessage::FitToWindow, view, grid);
            }
            None
        }
        TileGridCanvasMessage::FitToWindow => {
            // If dimensions are still defaults, use a reasonable fallback
            if view.last_canvas_w <= 0.0 || view.last_canvas_h <= 0.0 {
                view.last_canvas_w = 800.0;
                view.last_canvas_h = 600.0;
            }
            // The canvas pane doesn't fill the full window — the inspector
            // takes the remaining fraction. Scale the available width/height
            // by `CANVAS_WIDTH_RATIO` before computing the fit zoom.
            let avail_w = view.last_canvas_w * super::state::CANVAS_WIDTH_RATIO;
            let avail_h = view.last_canvas_h;
            let gs = grid.grid_size as f32;
            let cell_total = gs * super::state::CELL_SIZE;
            let zoom_x = avail_w / cell_total;
            let zoom_y = avail_h / cell_total;
            view.zoom = zoom_x
                .min(zoom_y)
                .clamp(super::state::MIN_ZOOM, super::state::MAX_ZOOM);
            view.pan_x = (avail_w - cell_total * view.zoom) * 0.5;
            view.pan_y = (avail_h - cell_total * view.zoom) * 0.5;
            view.needs_fit = false;
            view.invalidate_cache();
            Some(format!("Fit to window: {:.2}×", view.zoom))
        }
        TileGridCanvasMessage::EditSelected(new_desc) => {
            if let Some(idx) = view.selected_cell {
                let word = encode_descriptor(new_desc);
                if (idx as usize) < grid.data.len() {
                    // TODO(bug: no-edit-history): EditSelected mutates grid.data
                    // directly without recording in EditHistory, so undo/redo
                    // won't work for canvas edits. Need to push a
                    // (cell_index, old_word, new_word) entry to the standard
                    // editor's EditHistory before mutating.
                    grid.data[idx as usize] = word;
                    // Keep synthetic metadata in sync — payload_length and grid_size
                    // are not on-disk fields for `.dat` but are written inside to_writer.
                    let new_len = grid.data.len() as u32 * 4;
                    grid.payload_length = new_len;
                    grid.grid_size = (grid.data.len() as f64).sqrt() as u32;
                    view.invalidate_cache();
                    return Some(format!("Edited cell {idx}"));
                }
                return Some(format!(
                    "Cell {idx} out of range (grid has {})",
                    grid.data.len()
                ));
            }
            Some("No cell selected".to_string())
        }
        TileGridCanvasMessage::ClearSelection => {
            view.selected_cell = None;
            Some("Selection cleared".to_string())
        }
        TileGridCanvasMessage::PaneResized(event) => {
            if let Some(ref mut ps) = view.pane_state {
                ps.resize(event.split, event.ratio);
            }
            None
        }
        TileGridCanvasMessage::PaneClicked(pane) => {
            view.pane_focus = Some(pane);
            None
        }
        TileGridCanvasMessage::EditFieldBuffer { field, value } => {
            if let Some(idx) = view.selected_cell {
                view.edit_buffers.insert((idx, field.clone()), value);
            }
            None
        }
        TileGridCanvasMessage::CommitFieldBuffer { field } => {
            if let Some(idx) = view.selected_cell {
                let buf_opt = view.edit_buffers.remove(&(idx, field.clone()));
                if buf_opt.is_none() {
                    return Some(format!("No edit buffer for '{field}' on cell {idx}"));
                }
                let buf = buf_opt.unwrap();
                if buf.trim().is_empty() {
                    return Some("Empty value — nothing to commit".to_string());
                }
                if let Ok(v) = buf.parse::<i32>()
                    && let Some(word) = grid.data.get(idx as usize).copied()
                {
                    let mut desc = TileDescriptor::from_word(word);
                    apply_field_to_desc(&mut desc, &field, v);
                    if (idx as usize) < grid.data.len() {
                        let old_word = grid.data[idx as usize];
                        let new_word = encode_descriptor(desc);
                        if new_word != old_word {
                            grid.data[idx as usize] = new_word;
                            view.invalidate_cache();
                        }
                    }
                    return Some(format!("Updated '{field}' to {v}"));
                }
            }
            None
        }
    }
}

/// Apply a clamped value to the correct field of a `TileDescriptor` based on the field name.
fn apply_field_to_desc(d: &mut TileDescriptor, field: &str, v: i32) {
    if field == "object_id" {
        d.object_id = v.clamp(0, 1023) as u16;
    } else if field == "tile_index" {
        d.tile_index = v.clamp(0, 32767) as u16;
    } else if field == "render_flags" {
        d.render_flags = v.clamp(0, 3) as u8;
    } else if field == "reserved" {
        d.reserved = v.clamp(0, 31) as u8;
    }
}

/// Pack a `TileDescriptor` back into a `u32` word using the file's bit layout.
pub fn encode_descriptor(d: TileDescriptor) -> u32 {
    (d.object_id as u32 & 0x03ff)
        | ((d.tile_index as u32 & 0x7fff) << 10)
        | ((d.render_flags as u32 & 0x03) << 25)
        | ((d.reserved as u32 & 0x1f) << 27)
}
