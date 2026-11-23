// Tileset extraction and manipulation module
//
// This module handles the extraction of tiles from binary .GTL/.BTL files,
// tile plotting, and tileset atlas generation.

// ===========================================================================
// DISPEL GAME TILESET FILE FORMAT (.GTL/.BTL)
// ===========================================================================
//
// ASCII Diagram of File Structure:
//
// +------------------------------+
// | TILESET FILE HEADER         |
// | (none - direct tile data)   |
// +------------------------------+
// | TILE #1                    |
// | - 1024 pixels (32x32)       |
// | - RGB565 format (2 bytes)   |
// +------------------------------+
// | TILE #2                    |
// | - 1024 pixels (32x32)       |
// | - RGB565 format (2 bytes)   |
// +------------------------------+
// | ...                        |
// +------------------------------+
// | TILE #N                    |
// | - 1024 pixels (32x32)       |
// | - RGB565 format (2 bytes)   |
// +------------------------------+
//
// FILE STRUCTURE DETAILS:
// - No header or metadata
// - Direct sequence of tile data
// - Each tile: 32×32 pixels = 1024 pixels
// - Each pixel: 2 bytes (RGB565 format)
// - Total tile count = file_size / (32*32*2)
//
// RGB565 COLOR FORMAT:
// - 5 bits: Red channel
// - 6 bits: Green channel
// - 5 bits: Blue channel
// - Stored as u16: 0xRRRRRGGGGGGBBBBB
//
// ISOMETRIC TILE PROPERTIES:
// - Rendered width: 62 pixels (TILE_WIDTH)
// - Rendered height: 32 pixels (TILE_HEIGHT)
// - Diamond-shaped mask for isometric projection
// - Transparency: RGB(0,0,0) treated as transparent
//
// FILE TYPES:
// - .GTL files: Ground Tile Layer (terrain, paths, etc.)
// - .BTL files: Building Tile Layer (structures, roofs, etc.)
//
// ===========================================================================

use byteorder::{LittleEndian, ReadBytesExt};
use image::{Rgb, RgbImage, RgbaImage};
use std::io::{BufReader, Result, Seek};
use std::{fs::File, path::Path};

use crate::map::types::TILE_PIXEL_NUMBER;
// Tile geometry and color conversion are owned by the version-agnostic decode
// abstraction. TILE_WIDTH/TILE_HEIGHT are re-exported to keep existing paths.
use crate::decode::{Color, rgb565_to_color};
pub use crate::decode::{TILE_HEIGHT, TILE_WIDTH};

/// Represents a single tile with its color data
#[derive(Debug, Clone)]
pub struct Tile {
    pub colors: [Color; 1024],
}

/// Extracts tiles from a binary tileset file (.GTL or .BTL)
///
/// This function parses the simple binary tileset format used by Dispel.
/// The format consists of a direct sequence of 32×32 pixel tiles stored
/// in RGB565 format (2 bytes per pixel), with no header or metadata.
///
/// # File Structure
/// - No header or metadata
/// - Direct sequence of tile data
/// - Each tile: 32×32 pixels = 1024 pixels × 2 bytes = 2048 bytes
/// - RGB565 color format: 5-6-5 bits for R-G-B channels
/// - Total tiles = file_size / 2048
///
/// # Arguments
/// * `source_path` - Path to the binary tileset file (.gtl or .btl)
///
/// # Returns
/// Vector of Tile structs containing the extracted tile data
///
/// # Tile Types
/// - .GTL files: Ground Tile Layer - terrain, paths, natural features
/// - .BTL files: Building Tile Layer - structures, roofs, man-made objects
///
/// # Color Conversion
/// Converts RGB565 to RGB888 during extraction:
/// - 5-bit red → 8-bit red (scaled 0-31 to 0-255)
/// - 6-bit green → 8-bit green (scaled 0-63 to 0-255)
/// - 5-bit blue → 8-bit blue (scaled 0-31 to 0-255)
///
/// # Isometric Properties
/// Tiles are designed for isometric projection:
/// - Rendered as diamonds (62×32 pixels when projected)
/// - RGB(0,0,0) pixels treated as transparent
/// - Uses diamond-shaped mask for proper isometric rendering
pub fn extract(source_path: &Path) -> Result<Vec<Tile>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    let tile_number = file_len / (32 * 32 * 2);
    let mut tiles = Vec::<Tile>::with_capacity(tile_number as usize);

    for _ in 0..tile_number {
        let pos = reader.stream_position()?;
        if pos > file_len {
            break;
        }

        let mut pixels: [Color; 1024] = [Color { r: 0, g: 0, b: 0 }; TILE_PIXEL_NUMBER as usize];
        for p in 0..TILE_PIXEL_NUMBER {
            let pixel = reader.read_u16::<LittleEndian>()?;
            let color = rgb565_to_color(pixel);
            pixels[p as usize] = color;
        }

        let tile = Tile { colors: pixels };
        tiles.push(tile);
    }

    Ok(tiles)
}

/// Plots all tiles as individual PNG files
///
/// # Arguments
///
/// * `tiles` - Vector of Tile structs to plot
/// * `out_dir` - Output directory path
pub fn plot_all_tiles(tiles: &[Tile], out_dir: &str) -> std::io::Result<()> {
    let out_path = std::path::Path::new(out_dir);
    std::fs::create_dir_all(out_path)?;

    for (tile_index, tile) in tiles.iter().enumerate() {
        let mut imgbuf = image::ImageBuffer::new(TILE_WIDTH, TILE_HEIGHT);
        let dest_x = 0;
        let dest_y: i32 = 0;
        plot_tile_rgba(&mut imgbuf, tile.colors, dest_x, dest_y);
        let file_path = out_path.join(format!("tile_{:04}.png", tile_index));
        imgbuf.save(file_path).map_err(std::io::Error::other)?;
    }
    Ok(())
}

/// Plots a single tile onto an RGB image buffer
///
/// # Arguments
///
/// * `imgbuf` - Target RGB image buffer
/// * `colors` - Color data for the tile
/// * `dest_x` - X coordinate to plot the tile
/// * `dest_y` - Y coordinate to plot the tile
pub fn plot_tile(imgbuf: &mut RgbImage, colors: [Color; 1024], dest_x: i32, dest_y: i32) {
    let img_w = imgbuf.width() as i32;
    let img_h = imgbuf.height() as i32;

    // Early-out: tile is entirely outside the image
    if dest_x + TILE_WIDTH as i32 <= 0
        || dest_x >= img_w
        || dest_y + TILE_HEIGHT as i32 <= 0
        || dest_y >= img_h
    {
        return;
    }

    let mask = crate::decode::tile_mask();
    let mut i = 0;
    for (y, row) in mask.iter().enumerate() {
        for x in 0..row[1] {
            let pixel: Color = colors[i];
            i += 1;

            if pixel.r == 0 && pixel.g == 0 && pixel.b == 0 {
                continue;
            }

            let final_x = dest_x + x + row[0];
            let final_y = dest_y + y as i32;

            if final_x >= 0 && final_x < img_w && final_y >= 0 && final_y < img_h {
                imgbuf.put_pixel(
                    final_x as u32,
                    final_y as u32,
                    Rgb([pixel.r, pixel.g, pixel.b]),
                );
            }
        }
    }
}

/// Same diamond-masked blit as [`plot_tile`], but writes black (0,0,0)
/// pixels too. The overlay id's mode entry picks per tile between this and
/// the black-skipping variant.
pub fn plot_tile_opaque(imgbuf: &mut RgbImage, colors: [Color; 1024], dest_x: i32, dest_y: i32) {
    let img_w = imgbuf.width() as i32;
    let img_h = imgbuf.height() as i32;

    if dest_x + TILE_WIDTH as i32 <= 0
        || dest_x >= img_w
        || dest_y + TILE_HEIGHT as i32 <= 0
        || dest_y >= img_h
    {
        return;
    }

    let mask = crate::decode::tile_mask();
    let mut i = 0;
    for (y, row) in mask.iter().enumerate() {
        for x in 0..row[1] {
            let pixel: Color = colors[i];
            i += 1;

            let final_x = dest_x + x + row[0];
            let final_y = dest_y + y as i32;

            if final_x >= 0 && final_x < img_w && final_y >= 0 && final_y < img_h {
                imgbuf.put_pixel(
                    final_x as u32,
                    final_y as u32,
                    Rgb([pixel.r, pixel.g, pixel.b]),
                );
            }
        }
    }
}

/// Generates a tileset atlas image containing all tiles in a grid
///
/// # Arguments
///
/// * `tiles` - Vector of Tile structs
/// * `out_path` - Output file path for the atlas PNG
pub fn plot_tileset_map(tiles: &[Tile], out_path: &str) -> std::io::Result<()> {
    plot_tileset_map_with(tiles, out_path, false)
}

/// Writes the atlas PNG. `opaque` draws the tiles' black pixels; otherwise
/// they are skipped.
pub fn plot_tileset_map_with(tiles: &[Tile], out_path: &str, opaque: bool) -> std::io::Result<()> {
    // Flexible atlas size to make it square
    // let count = tiles.len() as f64;
    // let w = count.sqrt().ceil() as u32;
    // let h = (count / w as f64).ceil() as u32;

    let w = 48; // Fixed 48 tiles per row
    let h = (tiles.len() as f64 / w as f64).ceil() as u32;

    println!("Tiles: {}, Atlas size: {}x{} tiles", tiles.len(), w, h);
    let width: u32 = TILE_WIDTH * w;
    let height: u32 = TILE_HEIGHT * h;
    let mut bitmap = image::ImageBuffer::new(width, height);

    let mut tile_index = 0;
    for y in 0..h {
        for x in 0..w {
            if tiles.len() == tile_index {
                break;
            }

            let tile = &tiles[tile_index];
            tile_index += 1;

            let offset_x = 0;
            let offset_y = 0;
            // if y % 2 != 0 {
            //     offset_x = TILE_WIDTH / 2 + 1;
            //     offset_y = TILE_HEIGHT / 2;
            // }

            let dest_x: u32 = x * TILE_WIDTH + offset_x;
            let dest_x: i32 = dest_x.try_into().unwrap_or(0);
            let dest_y: u32 = y * TILE_HEIGHT + offset_y;
            let dest_y: i32 = dest_y.try_into().unwrap_or(0);

            plot_tile_rgba_opaque(&mut bitmap, tile.colors, dest_x, dest_y, opaque)
        }
    }

    bitmap.save(out_path).map_err(std::io::Error::other)
}

/// Plots a single tile onto an RGBA image buffer, skipping its RGB(0,0,0)
/// pixels.
///
/// # Arguments
///
/// * `imgbuf` - Target RGBA image buffer
/// * `colors` - Color data for the tile
/// * `dest_x` - X coordinate to plot the tile
/// * `dest_y` - Y coordinate to plot the tile
pub fn plot_tile_rgba(imgbuf: &mut RgbaImage, colors: [Color; 1024], dest_x: i32, dest_y: i32) {
    plot_tile_rgba_opaque(imgbuf, colors, dest_x, dest_y, false)
}

/// Blits one tile into an RGBA atlas.
///
/// `opaque` selects how black pixels are drawn: when true they are copied
/// like any other pixel; when false RGB(0,0,0) pixels are skipped.
pub fn plot_tile_rgba_opaque(
    imgbuf: &mut RgbaImage,
    colors: [Color; 1024],
    dest_x: i32,
    dest_y: i32,
    opaque: bool,
) {
    let img_w = imgbuf.width() as i32;
    let img_h = imgbuf.height() as i32;

    // Early-out: tile is entirely outside the image
    if dest_x + TILE_WIDTH as i32 <= 0
        || dest_x >= img_w
        || dest_y + TILE_HEIGHT as i32 <= 0
        || dest_y >= img_h
    {
        return;
    }

    let mask = crate::decode::tile_mask();
    let mut i = 0;
    for (y, row) in mask.iter().enumerate() {
        for x in 0..row[1] {
            let pixel: Color = colors[i];
            i += 1;

            if !opaque && pixel.r == 0 && pixel.g == 0 && pixel.b == 0 {
                continue;
            }

            let final_x = dest_x + x + row[0];
            let final_y = dest_y + y as i32;

            if final_x >= 0 && final_x < img_w && final_y >= 0 && final_y < img_h {
                imgbuf.put_pixel(
                    final_x as u32,
                    final_y as u32,
                    image::Rgba([pixel.r, pixel.g, pixel.b, 255]),
                );
            }
        }
    }
}

/// Mixes a color with a tile canvas using alpha blending
///
/// # Arguments
///
/// * `canvas` - Base tile color data
/// * `color` - Color to mix in
/// * `alpha` - Alpha value for blending (0-255)
///
/// # Returns
///
/// New color data with the mixed colors
pub fn mix_color(canvas: [Color; 1024], color: Color, alpha: u8) -> [Color; 1024] {
    let mut pixels = [Color { r: 0, g: 0, b: 0 }; TILE_PIXEL_NUMBER as usize];
    let amount: f64 = alpha as f64 / 255.0;

    for i in 0..TILE_PIXEL_NUMBER as usize {
        let base = canvas[i];
        let r = ((color.r as f64 * amount) + base.r as f64 * (1.0 - amount)) as u8;
        let g: u8 = ((color.g as f64 * amount) + base.g as f64 * (1.0 - amount)) as u8;
        let b: u8 = ((color.b as f64 * amount) + base.b as f64 * (1.0 - amount)) as u8;
        pixels[i] = Color { r, g, b };
    }
    pixels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mix_color() {
        let canvas = [Color {
            r: 255,
            g: 255,
            b: 255,
        }; 1024];
        let color = Color { r: 0, g: 0, b: 0 };
        let result = mix_color(canvas, color, 128);
        assert_eq!(result[0].r, 127);
    }

    #[test]
    fn test_tileset_plot_reports_unwritable_output_path() {
        let tile = Tile {
            colors: [Color { r: 1, g: 2, b: 3 }; 1024],
        };
        let tiles = [tile];

        let base = std::env::temp_dir().join(format!("dispel_tileset_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        let blocker = base.join("invented_blocker.bin");
        std::fs::write(&blocker, b"not a directory").unwrap();

        let bad_dir = blocker.join("invented_tiles");
        let result = plot_all_tiles(&tiles, &bad_dir.to_string_lossy());
        assert!(result.is_err(), "plot_all_tiles must report a failed save");

        let bad_atlas = blocker.join("invented_atlas.png");
        let result = plot_tileset_map(&tiles, &bad_atlas.to_string_lossy());
        assert!(
            result.is_err(),
            "plot_tileset_map must report a failed save"
        );

        std::fs::remove_dir_all(&base).unwrap();
    }

    // RGB565 golden values and the diamond-mask geometry test moved to
    // `crate::decode::tests` — the single abstraction boundary.
}
