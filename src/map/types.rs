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

/// Per-tile Y-sort key for one tile of a stacked BTL building (interlaced pass).
///
/// Each tile of a building stack sorts at its own screen band so entities can
/// interleave correctly with tall structures. Sorting the whole stack by
/// its visual bottom edge (`y + count * tile_h`) instead makes tall buildings
/// wrongly occlude entities that stand behind or beside their base.
///
/// `anchor_y` is the map-local pixel Y of the stack anchor (the `y` field of
/// [`TiledObjectInfo`]); `stack_index` is the 0-based position within `ids`.
pub fn tiled_object_sort_key(anchor_y: i32, stack_index: usize) -> i32 {
    let tile_h = TILE_HEIGHT_HALF * 2; // 32
    anchor_y + stack_index as i32 * tile_h + tile_h
}

/// Y-sort key for an internal map sprite (chair, throne, statue…) in the
/// interlaced pass: the file's `sprite_bottom_right_y` minus a half-tile
/// window.
pub fn internal_sprite_sort_key(bottom_right_y: i32) -> i32 {
    bottom_right_y - TILE_HEIGHT_HALF
}

/// An event trigger attached to a tile on the map.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct EventBlock {
    pub x: i32,
    pub y: i32,
    pub _unknown_value: i16,
    pub event_id: i16,
}

/// Placement record for a sprite embedded directly in the map file.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SpriteInfoBlock {
    pub sprite_id: usize,
    pub sprite_x: i32,
    pub sprite_y: i32,
    /// Bottom-right Y pixel (occluded space) — used as the Y-sort key for interlaced rendering.
    pub sprite_bottom_right_y: i32,
}

/// A building/object made up of stacked BTL tileset tiles.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TiledObjectInfo {
    pub ids: Vec<i16>,
    pub x: i32,
    pub y: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiled_object_sort_key_per_tile_bands() {
        // Each stack tile sorts one tile-height (32px) after the previous.
        assert_eq!(tiled_object_sort_key(100, 0), 132);
        assert_eq!(tiled_object_sort_key(100, 1), 164);
        assert_eq!(tiled_object_sort_key(100, 5), 292);
    }

    /// Regression: an NPC standing behind a tall building must be drawn after
    /// the building's base-level tiles. The old whole-stack key
    /// (`y + count * 32`) sorted the entire building after him, letting it
    /// paint over his sprite even though it stands behind him.
    #[test]
    fn test_npc_behind_tall_building_interleaves_with_stack() {
        let npc_feet = 560; // map-local bottom of the NPC's tile diamond

        // Building anchored at y=512, 4 tiles tall (visual bottom edge = 640).
        let keys: Vec<i32> = (0..4)
            .map(|level| tiled_object_sort_key(512, level))
            .collect();
        assert_eq!(keys, [544, 576, 608, 640]);

        // Only tiles whose band extends below the NPC's feet sort after him;
        // the base tile sorts before, so he renders in front of it.
        assert!(keys[0] < npc_feet, "base tile must draw before the NPC");
        assert_eq!(
            keys.iter().filter(|&&k| k > npc_feet).count(),
            3,
            "only the 3 upper-stack tiles may draw after the NPC"
        );
    }
}
