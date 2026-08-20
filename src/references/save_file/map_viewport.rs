use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
// use dispel_macros::BinaryRecord;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Number of isometric viewport cells stored in each save.
pub const MAP_VIEWPORT_CELL_COUNT: usize = 500;

/// Fixed size of the serialized map viewport state stored after the map-ID list.
pub const MAP_VIEWPORT_STATE_SIZE: usize = 10_148;

/// Save-world header and the map viewport state after map data.
///
/// Layout: `[map-section terminator: u32][8 × 4-byte header values]
/// [visited-map count][visited map IDs][map viewport state: 10,148 bytes]`.
///
/// The active record-size fields are 329, 349, and 200 in known saves; they
/// match the monster, NPC, and extra-object records. A fourth, unused section
/// has a record size of zero.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostMapsData {
    /// Terminator after the final map section. Known saves store zero.
    pub map_section_terminator: u32,
    /// Save-format version, observed as 1.45.
    pub game_version: f32,
    /// `AllMap.ini` record ID for the loaded map's resources, geometry, and presentation.
    pub all_map_ini_id: u32,
    /// `Ref/Map.ini` record ID for the entrance and spawn configuration used to enter the map.
    pub ref_map_ini_id: u32,
    /// Reserved save-header word; observed as zero.
    pub reserved_header_word: u32,
    /// Size of a MonsterRecord in the map section.
    pub monster_block_size: u32,
    /// Size of an NpcRecord in the map section.
    pub npc_block_size: u32,
    /// Record size for an unused map-object section; observed as zero.
    pub unused_map_object_block_size: u32,
    /// Size of an ExtraObjectRecord in the map section.
    pub extra_object_block_size: u32,
    /// Number of visited maps, which must match the preceding map section.
    pub number_of_visited_maps: u32,
    /// IDs of the visited maps.
    pub map_ids: Vec<u32>,
}

impl PostMapsData {
    pub(super) fn read_from<R: Read>(
        reader: &mut R,
        expected_map_count: u32,
    ) -> std::io::Result<Self> {
        let map_section_terminator = reader.read_u32::<LittleEndian>()?;
        let game_version = reader.read_f32::<LittleEndian>()?;
        let all_map_ini_id = reader.read_u32::<LittleEndian>()?;
        let ref_map_ini_id = reader.read_u32::<LittleEndian>()?;
        let reserved_header_word = reader.read_u32::<LittleEndian>()?;
        let monster_block_size = reader.read_u32::<LittleEndian>()?;
        let npc_block_size = reader.read_u32::<LittleEndian>()?;
        let unused_map_object_block_size = reader.read_u32::<LittleEndian>()?;
        let extra_object_block_size = reader.read_u32::<LittleEndian>()?;
        let number_of_visited_maps = reader.read_u32::<LittleEndian>()?;

        if number_of_visited_maps != expected_map_count {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "post-maps visited-map count is {number_of_visited_maps}, expected {expected_map_count}"
                ),
            ));
        }

        let mut map_ids = vec![0u32; number_of_visited_maps as usize];
        for map_id in &mut map_ids {
            *map_id = reader.read_u32::<LittleEndian>()?;
        }

        Ok(Self {
            map_section_terminator,
            game_version,
            all_map_ini_id,
            ref_map_ini_id,
            reserved_header_word,
            monster_block_size,
            npc_block_size,
            unused_map_object_block_size,
            extra_object_block_size,
            number_of_visited_maps,
            map_ids,
        })
    }

    pub(super) fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_u32::<LittleEndian>(self.map_section_terminator)?;
        writer.write_f32::<LittleEndian>(self.game_version)?;
        writer.write_u32::<LittleEndian>(self.all_map_ini_id)?;
        writer.write_u32::<LittleEndian>(self.ref_map_ini_id)?;
        writer.write_u32::<LittleEndian>(self.reserved_header_word)?;
        writer.write_u32::<LittleEndian>(self.monster_block_size)?;
        writer.write_u32::<LittleEndian>(self.npc_block_size)?;
        writer.write_u32::<LittleEndian>(self.unused_map_object_block_size)?;
        writer.write_u32::<LittleEndian>(self.extra_object_block_size)?;
        writer.write_u32::<LittleEndian>(self.number_of_visited_maps)?;
        for map_id in &self.map_ids {
            writer.write_u32::<LittleEndian>(*map_id)?;
        }
        Ok(())
    }
}

/// A cached correspondence between an isometric screen position and a map tile.
///
/// The game rebuilds records of this shape: screen positions
/// advance in 32- and 16-pixel isometric steps, while `map_tile_index` is
/// `map_y * map_width + map_x`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MapViewportCell {
    pub screen_x: u32,
    pub screen_y: u32,
    pub map_x: u32,
    pub map_y: u32,
    pub map_tile_index: u32,
}

/// Rectangle in the game's screen-pixel coordinate system.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct MapViewportRect {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

/// Map coordinate and its precomputed row-major tile index.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct MapTileReference {
    pub map_x: u32,
    pub map_y: u32,
    pub map_tile_index: u32,
}

/// Serialized state of the game's isometric map viewport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapViewportState {
    /// Fixed screen rectangle in which the map is drawn.
    pub viewport_clip_rect: MapViewportRect,
    /// Projected map rectangle translated as the camera scrolls.
    pub map_projection_rect: MapViewportRect,
    /// Eight map-tile references used to constrain and update camera movement.
    pub camera_boundary_tiles: [MapTileReference; 8],
    /// Cached screen-to-map lookup cells.
    pub cells: Vec<MapViewportCell>,
    /// Active smooth-scroll direction: `-1`=idle, `0`=up, `1`=up-right,
    /// `2`=right, `3`=down-right, `4`=down, `5`=down-left, `6`=left,
    /// `7`=up-left.
    pub scroll_direction: i32,
    /// Accumulated horizontal sub-tile offset during smooth scrolling.
    pub smooth_scroll_offset_x: u32,
    /// Accumulated vertical sub-tile offset during smooth scrolling.
    pub smooth_scroll_offset_y: u32,
    /// Current frame of the smooth-scroll animation.
    pub scroll_animation_frame: u32,
    /// Number of frames in the active smooth-scroll animation.
    pub scroll_animation_frame_count: u32,
}

impl Default for MapViewportState {
    fn default() -> Self {
        Self {
            viewport_clip_rect: MapViewportRect::default(),
            map_projection_rect: MapViewportRect::default(),
            camera_boundary_tiles: [MapTileReference::default(); 8],
            cells: vec![MapViewportCell::default(); MAP_VIEWPORT_CELL_COUNT],
            scroll_direction: -1,
            smooth_scroll_offset_x: 0,
            smooth_scroll_offset_y: 0,
            scroll_animation_frame: 0,
            scroll_animation_frame_count: 0,
        }
    }
}

impl MapViewportState {
    pub fn read_from<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let viewport_clip_rect = read_rect(reader)?;
        let map_projection_rect = read_rect(reader)?;
        let mut camera_boundary_tiles = [MapTileReference::default(); 8];
        for tile in &mut camera_boundary_tiles {
            tile.map_x = reader.read_u32::<LittleEndian>()?;
            tile.map_y = reader.read_u32::<LittleEndian>()?;
            tile.map_tile_index = reader.read_u32::<LittleEndian>()?;
        }
        let mut cells = vec![MapViewportCell::default(); MAP_VIEWPORT_CELL_COUNT];
        for cell in &mut cells {
            cell.screen_x = reader.read_u32::<LittleEndian>()?;
            cell.screen_y = reader.read_u32::<LittleEndian>()?;
            cell.map_x = reader.read_u32::<LittleEndian>()?;
            cell.map_y = reader.read_u32::<LittleEndian>()?;
            cell.map_tile_index = reader.read_u32::<LittleEndian>()?;
        }
        Ok(Self {
            viewport_clip_rect,
            map_projection_rect,
            camera_boundary_tiles,
            cells,
            scroll_direction: reader.read_i32::<LittleEndian>()?,
            smooth_scroll_offset_x: reader.read_u32::<LittleEndian>()?,
            smooth_scroll_offset_y: reader.read_u32::<LittleEndian>()?,
            scroll_animation_frame: reader.read_u32::<LittleEndian>()?,
            scroll_animation_frame_count: reader.read_u32::<LittleEndian>()?,
        })
    }

    pub(crate) fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        if self.cells.len() != MAP_VIEWPORT_CELL_COUNT {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "map viewport has {} cells, expected {MAP_VIEWPORT_CELL_COUNT}",
                    self.cells.len()
                ),
            ));
        }
        write_rect(writer, self.viewport_clip_rect)?;
        write_rect(writer, self.map_projection_rect)?;
        for tile in self.camera_boundary_tiles {
            writer.write_u32::<LittleEndian>(tile.map_x)?;
            writer.write_u32::<LittleEndian>(tile.map_y)?;
            writer.write_u32::<LittleEndian>(tile.map_tile_index)?;
        }
        for cell in &self.cells {
            writer.write_u32::<LittleEndian>(cell.screen_x)?;
            writer.write_u32::<LittleEndian>(cell.screen_y)?;
            writer.write_u32::<LittleEndian>(cell.map_x)?;
            writer.write_u32::<LittleEndian>(cell.map_y)?;
            writer.write_u32::<LittleEndian>(cell.map_tile_index)?;
        }
        writer.write_i32::<LittleEndian>(self.scroll_direction)?;
        writer.write_u32::<LittleEndian>(self.smooth_scroll_offset_x)?;
        writer.write_u32::<LittleEndian>(self.smooth_scroll_offset_y)?;
        writer.write_u32::<LittleEndian>(self.scroll_animation_frame)?;
        writer.write_u32::<LittleEndian>(self.scroll_animation_frame_count)?;
        Ok(())
    }

    /// Serialize this state for inspection in the raw-hex viewer.
    ///
    /// Unlike [`Self::write_to`], this retains every supplied cell so the
    /// viewer can show malformed in-memory data before serialization rejects it.
    pub fn raw_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(148 + self.cells.len() * 20);
        append_rect(&mut bytes, self.viewport_clip_rect);
        append_rect(&mut bytes, self.map_projection_rect);
        for tile in self.camera_boundary_tiles {
            bytes.extend_from_slice(&tile.map_x.to_le_bytes());
            bytes.extend_from_slice(&tile.map_y.to_le_bytes());
            bytes.extend_from_slice(&tile.map_tile_index.to_le_bytes());
        }
        for cell in &self.cells {
            bytes.extend_from_slice(&cell.screen_x.to_le_bytes());
            bytes.extend_from_slice(&cell.screen_y.to_le_bytes());
            bytes.extend_from_slice(&cell.map_x.to_le_bytes());
            bytes.extend_from_slice(&cell.map_y.to_le_bytes());
            bytes.extend_from_slice(&cell.map_tile_index.to_le_bytes());
        }
        bytes.extend_from_slice(&self.scroll_direction.to_le_bytes());
        bytes.extend_from_slice(&self.smooth_scroll_offset_x.to_le_bytes());
        bytes.extend_from_slice(&self.smooth_scroll_offset_y.to_le_bytes());
        bytes.extend_from_slice(&self.scroll_animation_frame.to_le_bytes());
        bytes.extend_from_slice(&self.scroll_animation_frame_count.to_le_bytes());
        bytes
    }
}

fn read_rect<R: Read>(reader: &mut R) -> std::io::Result<MapViewportRect> {
    Ok(MapViewportRect {
        left: reader.read_u32::<LittleEndian>()?,
        top: reader.read_u32::<LittleEndian>()?,
        right: reader.read_u32::<LittleEndian>()?,
        bottom: reader.read_u32::<LittleEndian>()?,
    })
}

fn write_rect<W: Write>(writer: &mut W, rect: MapViewportRect) -> std::io::Result<()> {
    writer.write_u32::<LittleEndian>(rect.left)?;
    writer.write_u32::<LittleEndian>(rect.top)?;
    writer.write_u32::<LittleEndian>(rect.right)?;
    writer.write_u32::<LittleEndian>(rect.bottom)
}

fn append_rect(bytes: &mut Vec<u8>, rect: MapViewportRect) {
    bytes.extend_from_slice(&rect.left.to_le_bytes());
    bytes.extend_from_slice(&rect.top.to_le_bytes());
    bytes.extend_from_slice(&rect.right.to_le_bytes());
    bytes.extend_from_slice(&rect.bottom.to_le_bytes());
}
