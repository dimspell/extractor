//! Version-agnostic rendering primitives shared by the core renderer and the GUI.
//!
//! This module is the single owner of the two pieces of pixel knowledge that
//! used to be reimplemented on both sides of the core/GUI boundary:
//!
//! * the RGB555 / RGB565 → RGB888 channel expansion, and
//! * the isometric 62×32 diamond geometry of a Dispel tile (the 32-row mask
//!   and the raw-tile → RGBA blit).
//!
//! It deliberately depends on **no** `image` crate version. All pixel APIs
//! return plain RGBA byte buffers (`Vec<u8>`, row-major, 4 bytes per pixel), so
//! callers convert to their own image type only at the very edge (an iced
//! `Handle`, an `image` 0.24 `RgbaImage`, an `image` 0.25 PNG, …) without
//! inheriting somebody else's dependency version.

use serde::{Deserialize, Serialize};

// Isometric tile geometry. Kept here (not in `map::tileset`) so that the GUI
// can use it without pulling in the core renderer's `image` 0.24 types.
pub const TILE_WIDTH: u32 = 62;
pub const TILE_HEIGHT: u32 = 32;
/// A raw tile is a 32×32 grid of RGB565 pixels.
pub const TILE_PIXEL_COUNT: usize = 32 * 32;
/// 1024 pixels × 2 bytes.
pub const TILE_BYTE_COUNT: usize = TILE_PIXEL_COUNT * 2;

/// An 8-bit RGB color decoded from a 16-bit game pixel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Channel layout of a 16-bit sprite pixel. Tiles are always RGB565; this
/// distinction exists only for sprites (a handful of shipped assets are
/// RGB555, detectable because they never set bit 15).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpritePixelFormat {
    Rgb555,
    Rgb565,
}

/// Expand an `n`-bit channel (max value `max`) to 8 bits with rounding.
#[inline]
fn expand_to_u8(value: u16, max: u16) -> u8 {
    // `value * 255 / max` with a half-max offset for rounding; the product is
    // computed in u32 so it cannot overflow.
    ((u32::from(value) * 255 + u32::from(max) / 2) / u32::from(max)) as u8
}

/// Decode a 16-bit RGB565 pixel into an 8-bit RGB [`Color`].
///
/// Used for tiles and for sprites packed as RGB565. `0x0000` decodes to black;
/// callers that treat black as transparent decide that at the blit boundary.
#[inline]
pub fn rgb565_to_color(pixel: u16) -> Color {
    Color {
        r: expand_to_u8((pixel >> 11) & 0x1F, 0x1F),
        g: expand_to_u8((pixel >> 5) & 0x3F, 0x3F),
        b: expand_to_u8(pixel & 0x1F, 0x1F),
    }
}

/// Decode a 16-bit RGB555 pixel into an 8-bit RGB [`Color`].
#[inline]
pub fn rgb555_to_color(pixel: u16) -> Color {
    Color {
        r: expand_to_u8((pixel >> 10) & 0x1F, 0x1F),
        g: expand_to_u8((pixel >> 5) & 0x1F, 0x1F),
        b: expand_to_u8(pixel & 0x1F, 0x1F),
    }
}

/// Decode a 16-bit pixel using the requested channel packing.
#[inline]
pub fn rgb16_to_color(pixel: u16, format: SpritePixelFormat) -> Color {
    match format {
        SpritePixelFormat::Rgb555 => rgb555_to_color(pixel),
        SpritePixelFormat::Rgb565 => rgb565_to_color(pixel),
    }
}

/// The isometric diamond mask: `[[x_offset, width]; 32]`, one entry per row of
/// the 32-row tile.
///
/// Row widths run 2, 6, …, 62, 62, …, 6, 2 and consume exactly 1024 texels.
/// Diagonal neighbours step (±32, ∓16) and abut edge-to-edge; horizontal
/// neighbours step 64px, leaving the 2px isometric grid seam.
pub fn tile_mask() -> [[i32; 2]; TILE_HEIGHT as usize] {
    let mut mask = [[0i32; 2]; TILE_HEIGHT as usize];
    let mut half_width: i32 = 1;
    let mut direction: i32 = 1;
    for row in mask.iter_mut() {
        row[0] = TILE_WIDTH as i32 / 2 - half_width;
        row[1] = half_width * 2;
        half_width += 2 * direction;
        if half_width > 31 {
            direction = -1;
            half_width = 31;
        }
    }
    mask
}

/// Blit a decoded tile (already-expanded colors) into an RGBA byte buffer.
///
/// `rgba` is row-major with `canvas_w * canvas_h * 4` bytes. The tile is drawn
/// at `(dest_x, dest_y)` with the diamond mask applied and out-of-canvas pixels
/// clipped. When `opaque` is false, RGB(0,0,0) texels are skipped (the BTL
/// "skip-zero" blit); otherwise they are written like any other texel (the GTL
/// opaque row-copy).
pub fn blit_tile_rgba(
    colors: &[Color; TILE_PIXEL_COUNT],
    rgba: &mut [u8],
    canvas_w: u32,
    canvas_h: u32,
    dest_x: i32,
    dest_y: i32,
    opaque: bool,
) {
    let w = canvas_w as i32;
    let h = canvas_h as i32;

    // Early-out: tile entirely outside the canvas.
    if dest_x + TILE_WIDTH as i32 <= 0
        || dest_x >= w
        || dest_y + TILE_HEIGHT as i32 <= 0
        || dest_y >= h
    {
        return;
    }

    let mask = tile_mask();
    let mut src = 0usize;
    for (y, row) in mask.iter().enumerate() {
        for x in 0..row[1] {
            let color = colors[src];
            src += 1;

            if !opaque && color.r == 0 && color.g == 0 && color.b == 0 {
                continue;
            }

            let final_x = dest_x + x + row[0];
            let final_y = dest_y + y as i32;
            if final_x < 0 || final_x >= w || final_y < 0 || final_y >= h {
                continue;
            }

            let dst = (final_y as usize * canvas_w as usize + final_x as usize) * 4;
            if dst + 3 < rgba.len() {
                rgba[dst..dst + 4].copy_from_slice(&[color.r, color.g, color.b, 255]);
            }
        }
    }
}

/// Decode a standalone tile into a 62×32 RGBA byte buffer, applying the
/// diamond mask and treating RGB(0,0,0) as transparent.
pub fn tile_colors_to_rgba(colors: &[Color; TILE_PIXEL_COUNT]) -> Vec<u8> {
    let mut rgba = vec![0u8; (TILE_WIDTH * TILE_HEIGHT * 4) as usize];
    blit_tile_rgba(colors, &mut rgba, TILE_WIDTH, TILE_HEIGHT, 0, 0, false);
    rgba
}

/// Decode 2048 raw RGB565 bytes (one 32×32 tile) straight into a 62×32 RGBA
/// byte buffer. This is the version-agnostic entry point used by the GUI map
/// renderer: it owns the color conversion and delegates the diamond geometry
/// to [`blit_tile_rgba`], so only one blit loop exists.
pub fn decode_tile_rgba(tile_bytes: &[u8]) -> Vec<u8> {
    let mut colors = [Color { r: 0, g: 0, b: 0 }; TILE_PIXEL_COUNT];
    for (i, color) in colors.iter_mut().enumerate() {
        let base = i * 2;
        if base + 1 >= tile_bytes.len() {
            // Truncated input: leave this and all remaining texels black.
            // blit_tile_rgba treats black as transparent, matching the old
            // early-return behavior which left them clear.
            break;
        }
        *color = rgb565_to_color(u16::from_le_bytes([tile_bytes[base], tile_bytes[base + 1]]));
    }
    tile_colors_to_rgba(&colors)
}

// ===========================================================================
// Tests — golden values and geometry invariants live at the abstraction boundary.
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ── RGB565 golden values ───────────────────────────────────────────

    #[test]
    fn rgb565_decodes_black() {
        assert_eq!(rgb565_to_color(0x0000), Color { r: 0, g: 0, b: 0 });
    }

    #[test]
    fn rgb565_decodes_primaries() {
        assert_eq!(rgb565_to_color(0xF800), Color { r: 255, g: 0, b: 0 });
        assert_eq!(rgb565_to_color(0x07E0), Color { r: 0, g: 255, b: 0 });
        assert_eq!(rgb565_to_color(0x001F), Color { r: 0, g: 0, b: 255 });
    }

    #[test]
    fn rgb565_decodes_white_and_secondaries() {
        assert_eq!(
            rgb565_to_color(0xFFFF),
            Color {
                r: 255,
                g: 255,
                b: 255
            }
        );
        assert_eq!(
            rgb565_to_color(0xF81F),
            Color {
                r: 255,
                g: 0,
                b: 255
            }
        );
        assert_eq!(
            rgb565_to_color(0x07FF),
            Color {
                r: 0,
                g: 255,
                b: 255
            }
        );
    }

    #[test]
    fn rgb555_decodes_five_bit_green() {
        // 0x03E0 is green at full five-bit intensity (RGB555 has no 6th green
        // bit) and must expand to 255, not the 248 the old <<3 path produced.
        assert_eq!(
            rgb16_to_color(0x03E0, SpritePixelFormat::Rgb555),
            Color { r: 0, g: 255, b: 0 }
        );
    }

    // ── Isometric tile geometry ────────────────────────────────────────

    #[test]
    fn tile_mask_is_a_symmetric_1024_texel_diamond() {
        let mask = tile_mask();
        assert_eq!(mask.len(), TILE_HEIGHT as usize);
        assert!(mask.iter().all(|row| row.len() == 2));

        assert_eq!((mask[0][0], mask[0][1]), (30, 2), "top row");
        assert_eq!((mask[15][0], mask[15][1]), (0, 62), "first middle row");
        assert_eq!((mask[16][0], mask[16][1]), (0, 62), "second middle row");
        assert_eq!((mask[31][0], mask[31][1]), (30, 2), "bottom row");

        for i in 0..mask.len() {
            assert_eq!(mask[i], mask[mask.len() - 1 - i], "row {i} symmetry");
        }
        assert_eq!(mask.iter().map(|row| row[1]).sum::<i32>(), 1024);
    }

    #[test]
    fn decode_tile_rgba_places_pixels_on_the_diamond() {
        // Every texel is opaque red; only diamond pixels may be non-transparent.
        let tile = [0x00u8, 0xF8].repeat(TILE_PIXEL_COUNT);
        let rgba = decode_tile_rgba(&tile);
        assert_eq!(rgba.len(), (TILE_WIDTH * TILE_HEIGHT * 4) as usize);

        // Top row: transparent corners, two red texels at x=30 and x=31.
        assert_eq!(&rgba[0..4], &[0, 0, 0, 0]);
        assert_eq!(&rgba[30 * 4..30 * 4 + 4], &[255, 0, 0, 255]);
        assert_eq!(&rgba[31 * 4..31 * 4 + 4], &[255, 0, 0, 255]);
        assert_eq!(&rgba[32 * 4..32 * 4 + 4], &[0, 0, 0, 0]);

        // Center of the diamond is red and opaque.
        let center = (16 * TILE_WIDTH as usize + 31) * 4;
        assert_eq!(&rgba[center..center + 4], &[255, 0, 0, 255]);

        // Exactly the diamond texels are opaque.
        let opaque = rgba.chunks_exact(4).filter(|px| px[3] == 255).count();
        assert_eq!(opaque, TILE_PIXEL_COUNT);
    }

    #[test]
    fn decode_tile_rgba_treats_black_as_transparent() {
        let tile = [0x00u8, 0x00].repeat(TILE_PIXEL_COUNT);
        let rgba = decode_tile_rgba(&tile);
        assert!(rgba.chunks_exact(4).all(|px| px == [0, 0, 0, 0]));
    }

    #[test]
    fn decode_tile_rgba_handles_truncated_input() {
        let rgba = decode_tile_rgba(&[0x00, 0xF8]);
        assert_eq!(rgba.len(), (TILE_WIDTH * TILE_HEIGHT * 4) as usize);
        assert_eq!(&rgba[30 * 4..30 * 4 + 4], &[255, 0, 0, 255]);
        assert_eq!(rgba.chunks_exact(4).filter(|px| px[3] == 255).count(), 1);
    }

    #[test]
    fn decode_tile_rgba_matches_blitting_decoded_colors() {
        // Synthetic full tile with varied RGB565 values and scattered black
        // texels. Both entry points must produce byte-identical RGBA output.
        let mut tile = Vec::with_capacity(TILE_BYTE_COUNT);
        for i in 0..TILE_PIXEL_COUNT {
            let pixel: u16 = if i % 7 == 0 {
                0x0000
            } else {
                (i as u16).wrapping_mul(0x9E37)
            };
            tile.extend_from_slice(&pixel.to_le_bytes());
        }
        assert_eq!(tile.len(), TILE_BYTE_COUNT);

        let mut colors = [Color { r: 0, g: 0, b: 0 }; TILE_PIXEL_COUNT];
        for (i, color) in colors.iter_mut().enumerate() {
            *color = rgb565_to_color(u16::from_le_bytes([tile[i * 2], tile[i * 2 + 1]]));
        }

        assert_eq!(decode_tile_rgba(&tile), tile_colors_to_rgba(&colors));
    }

    // ── Property tests ─────────────────────────────────────────────────

    proptest::proptest! {
        #[test]
        fn rgb565_decode_recovers_original_fields(pixel: u16) {
            let c = rgb565_to_color(pixel);
            // Re-encoding the expanded channels must recover the same 5/6/5
            // fields (truncation is the exact inverse of the expanding scale).
            prop_assert_eq!(u16::from(c.r) >> 3, pixel >> 11 & 0x1F);
            prop_assert_eq!(u16::from(c.g) >> 2, pixel >> 5 & 0x3F);
            prop_assert_eq!(u16::from(c.b) >> 3, pixel & 0x1F);
        }

        #[test]
        fn rgb555_decode_recovers_original_fields(pixel: u16) {
            let c = rgb555_to_color(pixel);
            prop_assert_eq!(u16::from(c.r) >> 3, pixel >> 10 & 0x1F);
            prop_assert_eq!(u16::from(c.g) >> 3, pixel >> 5 & 0x1F);
            prop_assert_eq!(u16::from(c.b) >> 3, pixel & 0x1F);
        }
    }
}
