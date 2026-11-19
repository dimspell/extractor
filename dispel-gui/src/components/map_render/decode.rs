// ── Tile / sprite decoder ────────────────────────────────────────────────────
//
// Pixel geometry and color conversion are owned by `dispel_core::decode` (the
// version-agnostic rendering abstraction shared with the core renderer). This
// module only reads bytes and turns the returned RGBA buffers into iced image
// handles — it deliberately contains no RGB565 math of its own.

use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::components::map_render::InternalSpriteHandle;

/// Decode a single tile from raw RGB565 bytes to a 62×32 RGBA image.
///
/// Both the `*255/31` style channel expansion and the 62×32 diamond geometry
/// now live in the core decode module; this is a thin version-agnostic wrapper.
pub fn decode_tile_to_rgba(tile_bytes: &[u8]) -> Vec<u8> {
    dispel_core::decode::decode_tile_rgba(tile_bytes)
}

/// Decode all unique tile IDs referenced in the given HashMap from a tileset file.
pub fn decode_tileset_file(
    path: &Path,
    tile_ids: &HashSet<i32>,
) -> Result<HashMap<i32, Vec<u8>>, String> {
    if tile_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut file =
        std::fs::File::open(path).map_err(|e| format!("Cannot open {}: {}", path.display(), e))?;

    let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let max_tiles = (file_size / dispel_core::decode::TILE_BYTE_COUNT as u64) as i32;

    let mut result = HashMap::with_capacity(tile_ids.len());
    let mut buf = [0u8; dispel_core::decode::TILE_BYTE_COUNT];

    for &tile_id in tile_ids {
        if tile_id < 0 || tile_id >= max_tiles {
            continue;
        }
        let offset = tile_id as u64 * dispel_core::decode::TILE_BYTE_COUNT as u64;
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| format!("Seek error in {}: {}", path.display(), e))?;
        file.read_exact(&mut buf)
            .map_err(|e| format!("Read error in {}: {}", path.display(), e))?;
        result.insert(tile_id, decode_tile_to_rgba(&buf));
    }

    Ok(result)
}

/// Decode all internal map sprites (thrones, decor, vases …) from a .map file.
///
/// Reads each sprite block's first frame from the open file, decoding its
/// 16-bit pixels via the shared core decoder into a version-agnostic RGBA
/// buffer, then packs the result into `InternalSpriteHandle` (ready for the
/// GPU). `nox`/`noy` are `map_non_occluded_start_{x,y}` from the map model.
pub fn decode_internal_sprites<R>(
    reader: &mut R,
    map_data: &dispel_core::map::MapData,
) -> Vec<InternalSpriteHandle>
where
    R: Read + Seek,
{
    use dispel_core::decode::rgb16_to_color;
    use iced::widget::image::Handle;

    let nox = map_data.model.map_non_occluded_start_x;
    let noy = map_data.model.map_non_occluded_start_y;

    let mut result = Vec::new();
    for block in &map_data.sprite_blocks {
        let Some(sequence) = map_data.internal_sprites.get(block.sprite_id) else {
            continue;
        };
        let Some(frame) = sequence.frame_infos.first() else {
            continue;
        };
        if frame.width <= 0 || frame.height <= 0 {
            continue;
        }
        if reader
            .seek(SeekFrom::Start(frame.image_start_position))
            .is_err()
        {
            continue;
        }

        let w = frame.width as u32;
        let h = frame.height as u32;
        let pixel_count = (w * h) as usize;
        let mut raw = vec![0u8; pixel_count * 2];
        if reader.read_exact(&mut raw).is_err() {
            continue;
        }

        // Core owns the channel expansion (and the RGB555/RGB565 distinction).
        let mut pixels = vec![0u8; pixel_count * 4];
        for i in 0..pixel_count {
            let pixel = u16::from_le_bytes([raw[i * 2], raw[i * 2 + 1]]);
            if pixel == 0 {
                continue; // transparent
            }
            let color = rgb16_to_color(pixel, frame.pixel_format);
            let idx = i * 4;
            pixels[idx] = color.r;
            pixels[idx + 1] = color.g;
            pixels[idx + 2] = color.b;
            pixels[idx + 3] = 255;
        }

        result.push(InternalSpriteHandle {
            handle: Handle::from_rgba(w, h, pixels),
            x: block.sprite_x + nox,
            y: block.sprite_y + noy,
            sort_y: block.sprite_bottom_right_y,
            width: w,
            height: h,
        });
    }
    result
}
