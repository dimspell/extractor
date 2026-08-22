// ── GenericTilesLayer — renders GTL/BTL tiles + interlaced objects ─────────

use crate::components::map_render::draw_helpers::{
    diamond_path, draw_entity_sprite, draw_tile_layer,
};
use crate::components::map_render::geometry::{is_visible, tile_center, tile_to_screen};
use crate::components::map_render::traits::{EntityKind, MapRenderSource};
use crate::components::map_render::{MapCanvasState, TILE_H, TILE_W};
use iced::advanced::image::Image as CoreImage;
use iced::widget::canvas::{self, Action, Geometry};
use iced::{Color, Event, Point, Rectangle, Size, mouse};

/// Canvas Program for tile layers only (GTL, BTL, internal sprites).
/// Images always draw on top of primitives within a single canvas,
/// so we split into two canvases: tiles first, then overlays.
pub struct GenericTilesLayer<'a, S: MapRenderSource> {
    pub state: &'a S,
}

impl<'a, S: MapRenderSource, M: 'static> canvas::Program<M> for GenericTilesLayer<'a, S> {
    type State = MapCanvasState;

    fn update(
        &self,
        _interaction: &mut MapCanvasState,
        _event: &Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<Action<M>> {
        // Input is handled by a separate overlay layer in each consumer
        None
    }

    fn draw(
        &self,
        _interaction: &MapCanvasState,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        // Use the per-state cache so cursor moves (which only touch the overlay canvas)
        // don't trigger an expensive tile-layer redraw.
        let geometry = self
            .state
            .view()
            .tile_layer_cache
            .draw(renderer, bounds.size(), |frame| {
                // Fill background
                frame.fill_rectangle(
                    Point::ORIGIN,
                    bounds.size(),
                    Color::from_rgb(0.1, 0.1, 0.12),
                );

                let Some(map_handle) = self.state.map_data() else {
                    return;
                };
                let map_data = &map_handle.0;
                let model = &map_data.model;
                let diagonal = model.tiled_map_width + model.tiled_map_height;

                let view = self.state.view();
                let pan_x = view.pan_x;
                let pan_y = view.pan_y;
                let zoom = view.zoom;

                // Draw ground layer
                if view.show_ground && self.state.tiles_ready() {
                    draw_tile_layer(
                        frame,
                        &map_data.gtl_tiles,
                        self.state.gtl_handles(),
                        diagonal,
                        pan_x,
                        pan_y,
                        zoom,
                        bounds,
                    );
                }

                // ── Interlaced object pass ────────────────────────────────────
                if self.state.tiles_ready() {
                    let nox = model.map_non_occluded_start_x;
                    let noy = model.map_non_occluded_start_y;
                    let nox_f = nox as f32;
                    let noy_f = noy as f32;

                    enum Item {
                        TiledObject(usize),
                        Sprite(usize),
                        Entity(usize),
                    }

                    let mut items: Vec<(i32, i32, i32, Item)> = Vec::new();

                    if view.show_buildings {
                        // Buildings draw as single units ordered by their stack bottom.
                        for (i, info) in map_data.tiled_infos.iter().enumerate() {
                            let pos = dispel_core::map::types::tiled_object_sort_key(
                                info.y,
                                info.ids.len(),
                            );
                            items.push((pos, 0, info.x, Item::TiledObject(i)));
                        }
                    }

                    if view.show_internal_sprites {
                        for (i, spr) in self.state.internal_sprite_handles().iter().enumerate() {
                            // Half-tile window: props lose near-ties to characters sitting/standing
                            // on them (see internal_sprite_sort_key).
                            items.push((
                                dispel_core::map::types::internal_sprite_sort_key(spr.sort_y),
                                1,
                                spr.x,
                                Item::Sprite(i),
                            ));
                        }
                    }

                    // External entity sort key — kind rung breaks ties between entities sharing a
                    // tile (NPC must draw over an Extra).
                    for i in 0..self.state.entity_count() {
                        if let Some(ed) = self.state.entity_data(i) {
                            if !ed.visible {
                                continue;
                            }
                            items.push((
                                ed.sort_key,
                                ed.kind.type_order(),
                                ed.tile_x,
                                Item::Entity(i),
                            ));
                        }
                    }

                    items.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));

                    for (_, _, _, item) in &items {
                        match item {
                            Item::TiledObject(i) => {
                                let info = &map_data.tiled_infos[*i];
                                let base_x = (info.x as f32 + nox_f) * zoom + pan_x;
                                let base_y = (info.y as f32 + noy_f) * zoom + pan_y;
                                let w = TILE_W * zoom;
                                let h = TILE_H * zoom;
                                for (j, &btl_id) in info.ids.iter().enumerate() {
                                    if btl_id <= 0 {
                                        continue;
                                    }
                                    let handle_id = btl_id.unsigned_abs() as i32;
                                    let Some(handle) = self.state.btl_handles().get(&handle_id)
                                    else {
                                        continue;
                                    };
                                    let px = base_x;
                                    let py = base_y + j as f32 * h;
                                    if !is_visible(px, py, w, h, bounds) {
                                        continue;
                                    }
                                    frame.draw_image(
                                        Rectangle::new(Point::new(px, py), Size::new(w, h)),
                                        CoreImage::new(handle.clone()),
                                    );
                                }
                            }
                            Item::Sprite(i) => {
                                let spr = &self.state.internal_sprite_handles()[*i];
                                let sx = spr.x as f32 * zoom + pan_x;
                                let sy = spr.y as f32 * zoom + pan_y;
                                let sw = spr.width as f32 * zoom;
                                let sh = spr.height as f32 * zoom;
                                if is_visible(sx, sy, sw, sh, bounds) {
                                    frame.draw_image(
                                        Rectangle::new(Point::new(sx, sy), Size::new(sw, sh)),
                                        CoreImage::new(spr.handle.clone()),
                                    );
                                }
                            }
                            Item::Entity(i) => {
                                if let Some(ed) = self.state.entity_data(*i) {
                                    let (px, py) = tile_to_screen(
                                        ed.tile_x, ed.tile_y, diagonal, pan_x, pan_y, zoom,
                                    );
                                    if is_visible(px, py, TILE_W * zoom, TILE_H * zoom, bounds) {
                                        let (tile_cx, tile_cy) = tile_center(px, py, zoom);
                                        if let Some(spr) = ed.sprite {
                                            draw_entity_sprite(frame, spr, tile_cx, tile_cy, zoom);
                                        } else {
                                            // Fallback shape
                                            match ed.kind {
                                                EntityKind::Monster => {
                                                    let r = 4.0 * zoom;
                                                    frame.fill(
                                                        &diamond_path(tile_cx, tile_cy, r),
                                                        Color::from_rgba(0.9, 0.15, 0.15, 0.85),
                                                    );
                                                }
                                                EntityKind::Npc => {
                                                    let r = 3.5 * zoom;
                                                    frame.fill(
                                                        &canvas::Path::circle(
                                                            Point::new(tile_cx, tile_cy),
                                                            r,
                                                        ),
                                                        Color::from_rgba(0.15, 0.45, 0.9, 0.85),
                                                    );
                                                }
                                                EntityKind::Extra => {
                                                    let s = 5.0 * zoom;
                                                    frame.fill_rectangle(
                                                        Point::new(
                                                            tile_cx - s * 0.5,
                                                            tile_cy - s * 0.5,
                                                        ),
                                                        Size::new(s, s),
                                                        Color::from_rgba(0.95, 0.85, 0.1, 0.85),
                                                    );
                                                }
                                                EntityKind::DrawItem => {
                                                    let r = 6.0 * zoom;
                                                    frame.fill(
                                                        &diamond_path(tile_cx, tile_cy, r),
                                                        Color::from_rgba(0.6, 0.6, 0.8, 0.85),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Draw flat BTL roof layer (top, after all depth-sorted objects)
                if view.show_roofs && self.state.tiles_ready() {
                    draw_tile_layer(
                        frame,
                        &map_data.btl_tiles,
                        self.state.btl_handles(),
                        diagonal,
                        pan_x,
                        pan_y,
                        zoom,
                        bounds,
                    );
                }

                // ── Shadow/lighting pass (observed behavior) ─────────────────────
                // On Dark maps, level-0 tiles black out and lit tiles fade
                // through the fogdata.dat tables. Drawn after world pixels;
                // entities on the overlay canvas above stay fully visible,
                // like the player character. The GUI fades all channels
                // uniformly (canvas has no per-channel multiply); the CLI
                // renderer is the pixel-exact reference.
                if view.show_shadows
                    && let Some(fog) = self.state.shadow_data()
                {
                    draw_shadow_layer(
                        frame,
                        &map_data.access_ref_words,
                        fog,
                        diagonal,
                        pan_x,
                        pan_y,
                        zoom,
                        bounds,
                    );
                }
            });

        vec![geometry]
    }

    fn mouse_interaction(
        &self,
        _state: &MapCanvasState,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::Idle
        }
    }
}

// ── Shadow/lighting pass ─────────────────────────────────────────────────────

/// Draws the observed per-tile lighting pass over the world pixels.
///
/// Level-0 tiles (fog of war) black out; tiles with a light level fade
/// through the fogdata tables, one factor byte per 2×1-pixel pair walking
/// the span mask. The GUI fades all channels uniformly via alpha (canvas
/// has no per-channel multiply) — the CLI renderer is the pixel-exact
/// reference.
#[allow(clippy::too_many_arguments)]
fn draw_shadow_layer(
    frame: &mut canvas::Frame,
    words: &std::collections::HashMap<(i32, i32), u32>,
    fog: &dispel_core::map::render::FogData,
    diagonal: i32,
    pan_x: f32,
    pan_y: f32,
    zoom: f32,
    bounds: Rectangle,
) {
    use dispel_core::map::render::SHADOW_SPANS;

    let tile_w = TILE_W * zoom;
    let tile_h = TILE_H * zoom;
    let pair_w = 2.0 * zoom;
    let row_h = zoom;

    for (&(tx, ty), &word) in words {
        let level = (word >> 15) & 0x7FFF;
        if level >= 200 {
            continue;
        }

        let (px, py) = tile_to_screen(tx, ty, diagonal, pan_x, pan_y, zoom);
        if !is_visible(px, py, tile_w, tile_h, bounds) {
            continue;
        }

        if level == 0 {
            // Fog of war: solid black diamond covering the tile.
            let cx = px + tile_w * 0.5;
            let cy = py + tile_h * 0.5;
            let hw = tile_w * 0.5;
            let hh = tile_h * 0.5;
            let path = canvas::Path::new(|b| {
                b.move_to(Point::new(cx, cy - hh));
                b.line_to(Point::new(cx + hw, cy));
                b.line_to(Point::new(cx, cy + hh));
                b.line_to(Point::new(cx - hw, cy));
                b.close();
            });
            frame.fill(&path, Color::BLACK);
            continue;
        }

        // Lit tile: one overlay rect per pixel pair, alpha = 1 − f/32.
        let mut pair = 0usize;
        for (row, &(start, width)) in SHADOW_SPANS.iter().enumerate() {
            let y = py + row as f32 * row_h;
            for p in 0..(width / 2) {
                let x = px + (start + p * 2) as f32 * zoom;
                if is_visible(x, y, pair_w, row_h, bounds) {
                    let f = f32::from(fog.factor(level, pair)) / 32.0;
                    frame.fill_rectangle(
                        Point::new(x, y),
                        Size::new(pair_w, row_h),
                        Color::from_rgba(0.0, 0.0, 0.0, 1.0 - f),
                    );
                }
                pair += 1;
            }
        }
    }
}
