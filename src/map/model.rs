use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{BufReader, Result};

use super::types::{TILE_HEIGHT_HALF, TILE_HORIZONTAL_OFFSET_HALF};

/// Computed geometry of a map, used to drive rendering and tile coordinate maths.
#[derive(Copy, Clone, Debug)]
pub struct MapModel {
    /// Number of tiles along the X axis of the tiled grid.
    pub tiled_map_width: i32,
    /// Number of tiles along the Y axis of the tiled grid.
    pub tiled_map_height: i32,
    /// Total pixel width of the full (non-occluded) isometric image.
    pub map_width_in_pixels: i32,
    /// Total pixel height of the full (non-occluded) isometric image.
    pub map_height_in_pixels: i32,
    /// X pixel offset at which the visible (occluded) viewport starts.
    pub map_non_occluded_start_x: i32,
    /// Y pixel offset at which the visible (occluded) viewport starts.
    pub map_non_occluded_start_y: i32,
    /// Width in pixels of the cropped occluded image.
    pub occluded_map_in_pixels_width: i32,
    /// Height in pixels of the cropped occluded image.
    pub occluded_map_in_pixels_height: i32,
}

/// Reads the two leading i32 values (chunk width × height) and derives all
/// pixel dimensions and occlusion offsets for the map.
pub fn read_map_model(reader: &mut BufReader<File>) -> Result<MapModel> {
    // map size in chunks
    let width = reader.read_i32::<LittleEndian>()?;
    let height = reader.read_i32::<LittleEndian>()?;
    let diagonal = width
        .checked_add(height)
        .ok_or_else(|| std::io::Error::other(format!("Map size overflow: {}x{}", width, height)))?;

    const MAP_CHUNK_SIZE: i32 = 25;
    let tiled_map_width = width * MAP_CHUNK_SIZE - 1;
    let tiled_map_height = height * MAP_CHUNK_SIZE - 1;

    let map_width_in_pixels = diagonal * MAP_CHUNK_SIZE * TILE_HORIZONTAL_OFFSET_HALF;
    let map_height_in_pixels = diagonal * MAP_CHUNK_SIZE * TILE_HEIGHT_HALF;

    let x_aspect: f64 = 0.3;
    let y_aspect: f64 = 0.2;

    let compensate_x: f64 = TILE_HORIZONTAL_OFFSET_HALF.into();
    let compensate_y: f64 = 0.0;

    let map_non_occluded_start_x: f64 = map_width_in_pixels.into();
    let map_non_occluded_start_x: f64 = x_aspect * map_non_occluded_start_x - compensate_x;
    let map_non_occluded_start_x: i32 = map_non_occluded_start_x.round() as i32;

    let map_non_occluded_start_y: f64 = map_height_in_pixels.into();
    let map_non_occluded_start_y: f64 = y_aspect * map_non_occluded_start_y - compensate_y;
    let map_non_occluded_start_y: i32 = map_non_occluded_start_y.round() as i32;

    let occluded_map_in_pixels_width = map_width_in_pixels - (map_non_occluded_start_x * 2);
    let occluded_map_in_pixels_height = map_height_in_pixels - (map_non_occluded_start_y * 2);

    Ok(MapModel {
        tiled_map_width,
        tiled_map_height,
        map_width_in_pixels,
        map_height_in_pixels,
        map_non_occluded_start_x,
        map_non_occluded_start_y,
        occluded_map_in_pixels_width,
        occluded_map_in_pixels_height,
    })
}
