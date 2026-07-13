//! Update handlers for map preview interaction messages.

use crate::components::map_preview::message::PreviewMessage;
use crate::components::map_preview::state::{MapPreviewState, PreviewLayer};
use iced::widget::canvas;
use iced::Task;

pub fn handle(msg: PreviewMessage, state: &mut MapPreviewState) -> Task<Message> {
    match msg {
        PreviewMessage::Pan(dx, dy) => {
            state.view.pan_x += dx;
            state.view.pan_y += dy;
            state.view.tile_cache.clear();
            Task::none()
        }
        PreviewMessage::Zoom(factor, cx, cy) => {
            let old_zoom = state.view.zoom;
            let new_zoom = (old_zoom * factor).clamp(0.05, 8.0);
            let ratio = new_zoom / old_zoom;
            if cx.is_finite() && cy.is_finite() {
                state.view.pan_x = cx - (cx - state.view.pan_x) * ratio;
                state.view.pan_y = cy - (cy - state.view.pan_y) * ratio;
            }
            state.view.zoom = new_zoom;
            state.view.tile_cache.clear();
            Task::none()
        }
        PreviewMessage::FitToWindow => {
            // Use the map geometry to compute optimal zoom
            if let Some(ref map_data) = state.map_data {
                let model = &map_data.model;
                let diagonal = state.diagonal;
                let map_diagonal = model.tiled_map_width + model.tiled_map_height;
                let map_px_w = map_diagonal as f32 * 32.0;
                let map_px_h = map_diagonal as f32 * 16.0;
                let (cx, cy) = dispel_core::map::types::convert_map_coords_to_image_coords(
                    model.tiled_map_width / 2,
                    model.tiled_map_height / 2,
                    diagonal,
                );
                let vp_w = state.view.last_canvas_w;
                let vp_h = state.view.last_canvas_h;
                let zoom = (vp_w / map_px_w).min(vp_h / map_px_h).clamp(0.05, 1.0);
                state.view.zoom = zoom;
                state.view.pan_x = vp_w / 2.0 - cx as f32 * zoom;
                state.view.pan_y = vp_h / 2.0 - cy as f32 * zoom;
            } else {
                state.view.zoom = 1.0;
                state.view.pan_x = 0.0;
                state.view.pan_y = 0.0;
            }
            state.view.tile_cache.clear();
            Task::none()
        }
        PreviewMessage::LayerToggle(layer) => {
            match layer {
                PreviewLayer::Ground => state.view.show_ground = !state.view.show_ground,
                PreviewLayer::Buildings => state.view.show_buildings = !state.view.show_buildings,
                PreviewLayer::Roofs => state.view.show_roofs = !state.view.show_roofs,
                PreviewLayer::InternalSprites => {
                    state.view.show_internal_sprites = !state.view.show_internal_sprites;
                }
                PreviewLayer::Monsters => state.view.show_monsters = !state.view.show_monsters,
                PreviewLayer::Npcs => state.view.show_npcs = !state.view.show_npcs,
                PreviewLayer::Extras => state.view.show_extras = !state.view.show_extras,
                PreviewLayer::DrawItems => state.view.show_draw_items = !state.view.show_draw_items,
            }
            state.view.tile_cache.clear();
            Task::none()
        }
    }
}
