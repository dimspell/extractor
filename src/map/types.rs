/// Isometric (x, y) tile coordinate.
pub type Coords = (i32, i32);

pub const TILE_WIDTH_HALF: i32 = 62 / 2;
pub const TILE_HEIGHT_HALF: i32 = 32 / 2;
pub const TILE_HORIZONTAL_OFFSET_HALF: i32 = 32;
pub const TILE_PIXEL_NUMBER: i32 = 32 * 32;

/// Converts tile (x, y) into image pixel coordinates for isometric rendering.
pub fn convert_map_coords_to_image_coords(x: i32, y: i32, map_diagonal_tiles: i32) -> (i32, i32) {
    let start_x = (x + y) * TILE_HORIZONTAL_OFFSET_HALF;
    let start_y = (-x + y) * TILE_HEIGHT_HALF + (map_diagonal_tiles / 2 * TILE_HEIGHT_HALF);
    (start_x, start_y)
}

/// An event trigger attached to a tile on the map.
#[derive(Copy, Clone, Debug)]
pub struct EventBlock {
    pub x: i32,
    pub y: i32,
    pub(crate) unknown_value: i16,
    pub event_id: i16,
}

/// Placement record for a sprite embedded directly in the map file.
#[derive(Copy, Clone, Debug)]
pub struct SpriteInfoBlock {
    pub sprite_id: usize,
    pub sprite_x: i32,
    pub sprite_y: i32,
}

/// A building/object made up of stacked BTL tileset tiles.
#[derive(Clone, Debug)]
pub struct TiledObjectInfo {
    pub ids: Vec<i16>,
    pub x: i32,
    pub y: i32,
}
