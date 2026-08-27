use serde::{Deserialize, Serialize};

/// Serde support for fixed-size byte arrays larger than 32 (serde's built-in
/// array impls stop at 32); serializes as a plain byte sequence.
mod bytes_array {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<const N: usize, S: Serializer>(v: &[u8; N], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(v)
    }

    pub fn deserialize<'de, const N: usize, D: Deserializer<'de>>(
        d: D,
    ) -> Result<[u8; N], D::Error> {
        let vec = Vec::<u8>::deserialize(d)?;
        if vec.len() != N {
            return Err(serde::de::Error::invalid_length(
                vec.len(),
                &"exact byte array",
            ));
        }
        let arr: [u8; N] = vec.try_into().expect("length just checked");
        Ok(arr)
    }
}

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

/// One leaf record inside a tiled-bundle item.
///
/// On-disk layout, per entry:
/// ```text
/// [i32 × 4]             bounding box {bound_x, bound_y, bound_right, bound_bottom}
/// [i32 × 7]             anchor_x, anchor_y, draw_x, draw_y, grid_width,
///                       grid_height, stored_cell_count
/// [u16 × stored_cell_count] ids — BTL tile-stack ids for this entry
/// if type_flag == 1:
///     [u8 × stored_cell_count] extra_payload bytes
/// ```
///
/// Discovered field semantics (verified over all 33 shipped map fixtures,
/// 43,554 entries):
/// - The leading four i32 words are a **bounding box** in map pixels:
///   word0 == `anchor_x`, word1 == `anchor_y`, and in shipped maps always
///   `bound_right == bound_x + 64`, `bound_bottom == bound_y + grid_height * 32`.
/// - `anchor_x`/`anchor_y` are the stack anchor `(x, y)` in map-local pixels —
///   they surface as [`TiledObjectInfo::x`] / [`TiledObjectInfo::y`].
/// - `draw_x` (signed) and `draw_y` are position terms for drawing relative to
///   the camera. They do not correlate with the anchors or the bounding box.
/// - `grid_width` is constant 1 across all shipped maps; `grid_height`
///   equals `ids.len()`. The `grid_height` of the first entry sizes the two
///   parallel flag arrays that follow ([`TiledBundle::level_flags`]).
/// - `stored_cell_count` is a redundant duplicate of
///   `grid_width * grid_height` with zero exceptions. It sizes both trailing
///   arrays of the entry on disk (IDs and optional payload).
///
/// Shipped data is degenerate: exactly one record per bundle, eight items per
/// record, at most one entry per item; `BundleItem::type_flag ≡ 0` and
/// `BundleItem::field_14 ≡ 0` everywhere.
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BundleEntry {
    /// Left edge of the entry's bounding box in map pixels (== `anchor_x`).
    pub bound_x: i32,
    /// Top edge of the entry's bounding box in map pixels (== `anchor_y`).
    pub bound_y: i32,
    /// Right edge of the entry's bounding box in map pixels
    /// (== `bound_x + 64` in every shipped entry).
    pub bound_right: i32,
    /// Bottom edge of the entry's bounding box in map pixels
    /// (== `bound_y + grid_height * 32` in every shipped entry).
    pub bound_bottom: i32,
    /// Stack anchor X in map-local pixels.
    pub anchor_x: i32,
    /// Stack anchor Y in map-local pixels.
    pub anchor_y: i32,
    /// Signed X term subtracted from camera scroll during blit placement.
    pub draw_x: i32,
    /// Y term subtracted from camera scroll during blit placement.
    pub draw_y: i32,
    /// Grid width of the entry's cell layout (constant 1 across all shipped
    /// maps).
    pub grid_width: i32,
    /// Grid height of the entry's cell layout; equals `ids.len()`.
    pub grid_height: i32,
    /// On-disk count for the ID and optional payload arrays. It equals
    /// `grid_width * grid_height` in every shipped map.
    pub stored_cell_count: i32,
    /// `stored_cell_count` BTL overlay/tile ids (u16, little-endian) drawn
    /// bottom → top.
    pub ids: Vec<u16>,
    /// Present only when the parent item's `type_flag == 1`:
    /// `stored_cell_count` raw bytes.
    pub extra_payload: Option<Vec<u8>>,
}

impl BundleEntry {
    /// Anchor of this entry's tile stack in map-local pixels.
    pub fn coords(&self) -> (i32, i32) {
        (self.anchor_x, self.anchor_y)
    }

    /// Bounding box `(left, top, right, bottom)` of this entry in map pixels.
    pub fn bounds(&self) -> (i32, i32, i32, i32) {
        (
            self.bound_x,
            self.bound_y,
            self.bound_right,
            self.bound_bottom,
        )
    }
}

/// Two binary 0/1 flags for one stack level.
///
/// There are `n = first_entry().grid_height` of these, one per stack level;
/// semantics not yet pinned.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StackLevelFlags {
    pub flag_a: i32,
    pub flag_b: i32,
}

/// One record of the .map file's first block (the tiled-object refs).
///
/// Both words are **constant across ALL shipped maps** — `word0 == 0` and
/// `word1 == 1` in all 99,104 records probed. The earlier "linear tile index"
/// interpretation was wrong; the true semantics are unknown.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TiledObjectRef {
    /// Always 0 across all shipped maps; purpose unknown.
    pub word0: i32,
    /// Always 1 across all shipped maps; purpose unknown.
    pub word1: i32,
}

/// One mid-level node of the tiled-bundle tree.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BundleItem {
    /// First i32 of the item. When `1`, every child entry carries an extra
    /// payload array after its id list. Always 0 in all shipped maps.
    pub type_flag: i32,
    /// Third i32 of the item. It is 0 in all shipped maps. Its purpose is
    /// unknown.
    pub field_14: i32,
    pub entries: Vec<BundleEntry>,
}

/// One sub-record of a tiled bundle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BundleRecord {
    /// First i32 of the record. It is 0 in known maps.
    pub field_04: i32,
    // Binary metadata. It does not contain valid Windows-1250 or EUC-KR text.
    #[serde(with = "bytes_array")]
    pub body: [u8; 260],
    pub items: Vec<BundleItem>,
}

/// A placed tiled building ("bundle") from the .map tiled-objects block.
///
/// After the records, the first entry's `grid_height` gives the number of flag
/// pairs. The three end grids follow immediately. The format has no
/// end-of-block sentinel.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TiledBundle {
    pub records: Vec<BundleRecord>,
    /// One [`StackLevelFlags`] per stack level following the records
    /// (`n = first_entry().grid_height`); two binary 0/1 flags each.
    pub level_flags: Vec<StackLevelFlags>,
}

impl TiledBundle {
    /// First entry of the first record's first item, if any.
    ///
    /// This entry gives the number of flag pairs.
    pub fn first_entry(&self) -> Option<&BundleEntry> {
        self.records.first()?.items.first()?.entries.first()
    }

    /// Derives the renderer-facing [`TiledObjectInfo`] from the tree.
    ///
    /// Mapping (verified byte-exact against the previous empirical parser on
    /// every shipped map fixture): the anchor is the first entry's
    /// `(anchor_x, anchor_y)` and the BTL tile stack is that entry's u16
    /// `ids`.
    pub fn info(&self) -> Option<TiledObjectInfo> {
        let entry = self.first_entry()?;
        let (x, y) = entry.coords();
        Some(TiledObjectInfo {
            ids: entry.ids.iter().map(|&id| id as i16).collect(),
            x,
            y,
        })
    }
}

/// A building/object made up of stacked BTL tileset tiles.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TiledObjectInfo {
    pub ids: Vec<i16>,
    pub x: i32,
    pub y: i32,
}
