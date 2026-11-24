// ── Shared draw helpers for tile and overlay canvases ─────────────────────────

use crate::components::map_render::geometry::{is_visible, tile_to_screen};
use crate::components::map_render::{EntitySpriteHandle, TILE_H, TILE_W};
use iced::advanced::image::Image as CoreImage;
use iced::widget::canvas::{self, Frame};
use iced::{Color, Point, Rectangle, Size};
use std::collections::HashMap;

#[allow(clippy::too_many_arguments)]
pub fn draw_tile_layer(
    frame: &mut Frame,
    tile_map: &HashMap<(i32, i32), i32>,
    handles: &HashMap<i32, iced::widget::image::Handle>,
    diagonal: i32,
    pan_x: f32,
    pan_y: f32,
    zoom: f32,
    bounds: Rectangle,
) {
    let w = TILE_W * zoom;
    let h = TILE_H * zoom;

    for (&(tx, ty), &tile_id) in tile_map {
        let Some(handle) = handles.get(&tile_id) else {
            continue;
        };
        let (px, py) = tile_to_screen(tx, ty, diagonal, pan_x, pan_y, zoom);
        if !is_visible(px, py, w, h, bounds) {
            continue;
        }
        let rect = Rectangle::new(Point::new(px, py), Size::new(w, h));
        frame.draw_image(rect, CoreImage::new(handle.clone()));
    }
}

/// Mirror an RGBA image horizontally, one pixel row at a time. Flipped
/// entities need their pixels mirrored as well as their origin offset shifted,
/// otherwise the sprite is drawn in the wrong direction.
pub fn mirror_rgba_rows(pixels: &[u8], width: u32, height: u32) -> Vec<u8> {
    let w = width as usize;
    let row_len = w.saturating_mul(4);
    if row_len == 0 {
        return Vec::new();
    }
    let mut mirrored = vec![0u8; pixels.len()];
    for y in 0..height as usize {
        let start = y.saturating_mul(row_len);
        let Some(row) = pixels.get(start..start.saturating_add(row_len)) else {
            break;
        };
        for (x, px) in row.as_chunks::<4>().0.iter().enumerate() {
            let dst = start.saturating_add((w - 1 - x) * 4);
            mirrored[dst..dst + 4].copy_from_slice(px);
        }
    }
    mirrored
}

/// Render an entity sprite handle onto the canvas frame.
pub fn draw_entity_sprite(
    frame: &mut Frame,
    spr: &EntitySpriteHandle,
    tile_cx: f32,
    tile_cy: f32,
    zoom: f32,
) {
    let w = spr.width as f32 * zoom;
    let h = spr.height as f32 * zoom;
    let dest_x = if spr.flip {
        tile_cx + (spr.origin_x as f32 - spr.width as f32) * zoom
    } else {
        tile_cx - spr.origin_x as f32 * zoom
    };
    let dest_y = tile_cy - spr.origin_y as f32 * zoom;
    frame.draw_image(
        Rectangle::new(Point::new(dest_x, dest_y), Size::new(w, h)),
        CoreImage::new(spr.handle.clone()),
    );
}

/// Return a colour for a draw item based on its item type.
pub fn draw_item_color(item_type: dispel_core::references::enums::ItemTypeId) -> Color {
    use dispel_core::references::enums::ItemTypeId;
    match item_type {
        ItemTypeId::Weapon => Color::from_rgba(0.9, 0.15, 0.15, 0.85),
        ItemTypeId::Healing => Color::from_rgba(0.15, 0.9, 0.15, 0.85),
        ItemTypeId::Edit => Color::from_rgba(0.15, 0.45, 0.9, 0.85),
        ItemTypeId::Event => Color::from_rgba(0.8, 0.15, 0.8, 0.85),
        ItemTypeId::Misc => Color::from_rgba(0.95, 0.85, 0.1, 0.85),
        ItemTypeId::NoItem => Color::from_rgba(0.6, 0.6, 0.6, 0.85),
    }
}

/// Build a diamond (rotated square) path centered at (cx, cy) with half-size r.
pub fn diamond_path(cx: f32, cy: f32, r: f32) -> canvas::Path {
    canvas::Path::new(|b| {
        b.move_to(Point::new(cx, cy - r));
        b.line_to(Point::new(cx + r, cy));
        b.line_to(Point::new(cx, cy + r));
        b.line_to(Point::new(cx - r, cy));
        b.close();
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mirror_rgba_rows_reverses_each_row() {
        // 2×2 image with a distinct colour per pixel.
        let pixels = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        assert_eq!(
            mirror_rgba_rows(&pixels, 2, 2),
            vec![5, 6, 7, 8, 1, 2, 3, 4, 13, 14, 15, 16, 9, 10, 11, 12]
        );
    }

    #[test]
    fn test_mirror_rgba_rows_zero_width_is_empty() {
        assert!(mirror_rgba_rows(&[1, 2, 3, 4], 0, 2).is_empty());
    }
}
