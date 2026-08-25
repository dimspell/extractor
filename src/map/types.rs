use serde::{Deserialize, Serialize};

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

/// Y-sort key for a stacked BTL building in the interlaced pass: the visual
/// bottom edge of the whole stack (`y + count * tile_h`).
///
/// The building draws as a single unit ordered by its stack bottom.
/// Per-tile interleaving was tried and rejected: a wall's lower tiles then
/// slice through objects standing in front of the structure.
///
/// `anchor_y` is the map-local pixel Y of the stack anchor (the `y` field of
/// [`TiledObjectInfo`]); `stack_size` is `ids.len()`.
pub fn tiled_object_sort_key(anchor_y: i32, stack_size: usize) -> i32 {
    let tile_h = TILE_HEIGHT_HALF * 2; // 32
    anchor_y + stack_size as i32 * tile_h
}

/// Y-sort key for an internal map sprite (chair, throne, statue…) in the
/// interlaced pass: the file's `sprite_bottom_right_y` minus a half-tile
/// window.
pub fn internal_sprite_sort_key(bottom_right_y: i32) -> i32 {
    bottom_right_y - TILE_HEIGHT_HALF
}

/// An event trigger attached to a tile on the map.
///
/// Stored in the file as one packed u32 (the first of the three end grids).
/// Decode it through the accessors instead of masking by hand:
///
/// ```text
/// bits  0-13  event/transition id (low 14 bits; ids < 70 are
///             valid and resolve to a name via Map.ini / AllMap.ini)
/// bits 14-21  unmapped
/// bit  22     "tile marked / entity occupies" — monster chase logic
///             treats marked tiles as blocked
/// bits 23-31  unmapped parameters
/// ```
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct EventBlock {
    pub x: i32,
    pub y: i32,
    /// The tile's packed u32 exactly as stored in the file.
    pub word: u32,
}

impl EventBlock {
    /// Event/transition id — bits 0–13 of [`Self::word`].
    pub fn event_id(&self) -> u16 {
        (self.word & 0x3FFF) as u16
    }

    /// Overwrite the event id (bits 0–13), preserving every other bit.
    pub fn set_event_id(&mut self, id: u16) {
        self.word = (self.word & !0x3FFF) | (u32::from(id) & 0x3FFF);
    }

    /// Word bit 22 (`0x0040_0000`): the tile is marked/occupied by an entity.
    /// Movement code makes chasing monsters give up when their
    /// target stands on such a tile.
    pub fn is_tile_marked(&self) -> bool {
        self.word & 0x0040_0000 != 0
    }
}

/// Placement record for a sprite embedded directly in the map file.
///
/// On-disk this is `[sprite_id] + frame-0 record`; the frame record is
/// 24 bytes = bounding box `{left, top, right, bottom}` in *map pixel
/// coordinates* followed by a duplicated `{x, y}` anchor. Verified against
/// fixtures: `right - left == frame0.width` and `bottom - top ==
/// frame0.height` for every placement in cat1/map1.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SpriteInfoBlock {
    pub sprite_id: usize,
    /// Left edge of the sprite's bounding box in map pixels (duplicated on disk).
    pub sprite_x: i32,
    /// Top edge of the sprite's bounding box in map pixels (duplicated on disk).
    pub sprite_y: i32,
    /// Bottom edge of the sprite's bounding box (= top + frame height).
    /// Y-sort key for interlaced rendering.
    pub sprite_bottom_right_y: i32,
    /// Left edge of the frame-0 bounding box in map pixels (raw disk value;
    /// duplicates `sprite_x` in known maps).
    pub bbox_left: i32,
    /// Top edge of the frame-0 bounding box in map pixels (duplicates `sprite_y`).
    pub bbox_top: i32,
    /// Right edge of the frame-0 bounding box (== left + frame width).
    pub bbox_right: i32,
}

/// Retained per-bundle metadata from the tiled objects block, preserved so the
/// DB export is lossless. The 264-byte bundle header is stored verbatim
/// (its first i32 is a stamp); control words, unmapped parameters and trailing
/// counts are captured as named fields.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TiledObjectMetadata {
    /// The raw 264-byte bundle header, first i32 stamp included verbatim.
    pub metadata_blob: Vec<u8>,
    /// Control words — always observed as `(8, 0, 1, 0)` in known maps.
    pub control_0: i32,
    pub control_1: i32,
    pub control_2: i32,
    pub control_3: i32,
    /// Unmapped parameters preceding/following the stack anchor.
    pub param_0: i32,
    pub param_1: i32,
    pub param_2: i32,
    pub param_3: i32,
    pub param_4: i32,
    pub param_5: i32,
    /// Trailing counts gating data covered by the post-stack skip.
    pub extra_count_a: i32,
    pub extra_count_b: i32,
    /// The fixed 84-byte trailer after the tile stack (unmapped; retained
    /// verbatim for lossless DB export).
    pub trailing_fixed: Vec<u8>,
    /// The variable trailer of `(extra_count_a + extra_count_b +
    /// tile_stack_len) * 4` bytes after the fixed trailer (unmapped; retained
    /// verbatim).
    pub trailing_variable: Vec<u8>,
}

/// A building/object made up of stacked BTL tileset tiles.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TiledObjectInfo {
    pub ids: Vec<i16>,
    pub x: i32,
    pub y: i32,
}
