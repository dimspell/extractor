// ── Tile decoder ─────────────────────────────────────────────────────────────

use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Decode a single tile from raw RGB565 bytes to a 62×32 RGBA image.
pub fn decode_tile_to_rgba(tile_bytes: &[u8]) -> Vec<u8> {
    let mut rgba = vec![0u8; 62 * 32 * 4];
    let mut src = 0usize;

    for y in 0u32..32 {
        let hs = y.min(31 - y) as usize;
        let x_offset = (15 - hs) * 2;
        let width = 2 + hs * 4;

        for x in 0..width {
            let pixel565 = u16::from_le_bytes([tile_bytes[src * 2], tile_bytes[src * 2 + 1]]);
            src += 1;

            let r5 = ((pixel565 >> 11) & 0x1F) as u32;
            let g6 = ((pixel565 >> 5) & 0x3F) as u32;
            let b5 = (pixel565 & 0x1F) as u32;

            let r = (r5 * 255 / 31) as u8;
            let g = (g6 * 255 / 63) as u8;
            let b = (b5 * 255 / 31) as u8;
            let a = if r == 0 && g == 0 && b == 0 {
                0u8
            } else {
                255u8
            };

            let dst = (y as usize * 62 + x_offset + x) * 4;
            rgba[dst] = r;
            rgba[dst + 1] = g;
            rgba[dst + 2] = b;
            rgba[dst + 3] = a;
        }
    }

    rgba
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
    let max_tiles = (file_size / 2048) as i32;

    let mut result = HashMap::with_capacity(tile_ids.len());
    let mut buf = [0u8; 2048];

    for &tile_id in tile_ids {
        if tile_id < 0 || tile_id >= max_tiles {
            continue;
        }
        let offset = tile_id as u64 * 2048;
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| format!("Seek error in {}: {}", path.display(), e))?;
        file.read_exact(&mut buf)
            .map_err(|e| format!("Read error in {}: {}", path.display(), e))?;
        result.insert(tile_id, decode_tile_to_rgba(&buf));
    }

    Ok(result)
}
