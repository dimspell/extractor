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
/// The three record-size fields are 329, 349, and 200 in known saves; they
/// match the monster, NPC, and extra-object record sizes in the map section.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostMapsData {
    /// Terminator after the final map section. Known saves store zero.
    pub map_section_terminator: u32,
    /// Save-format version, observed as 1.45.
    pub game_version: f32,
    /// Unknown header value. Preserve it verbatim.
    pub unknown_header_value_1: u32,
    /// ID reference in AllMap.ini.
    pub all_map_ini_id: u32,
    /// ID reference in Ref/Map.ini.
    pub ref_map_ini_id: u32,
    /// Size of a MonsterRecord in the map section.
    pub monster_block_size: u32,
    /// Size of an NpcRecord in the map section.
    pub npc_block_size: u32,
    /// Unknown header value. Preserve it verbatim.
    pub unknown_header_value_2: u32,
    /// Size of an ExtraObjectRecord in the map section.
    pub extra_object_block_size: u32,
    /// Number of visited maps, which must match the preceding map section.
    pub number_of_visited_maps: u32,
    /// IDs of the visited maps.
    pub map_ids: Vec<u32>,
    /// Fixed-size serialized isometric map viewport state.
    pub map_viewport_state: MapViewportState,
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

/// Serialized state of the game's isometric map viewport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapViewportState {
    /// Four viewport/render-bound values from object offsets `0x68..=0x74`.
    pub render_bounds: [u32; 4],
    /// Four viewport-bound values from object offsets `0x44..=0x50`.
    pub viewport_bounds: [u32; 4],
    /// Geometry values from object offsets `0xB4..=0x110`.
    pub geometry: [u32; 24],
    /// Cached screen-to-map lookup cells.
    pub cells: Vec<MapViewportCell>,
    /// Value at object offset `0x11C`; initialized to `-1` by the game.
    pub selected_tile_index: u32,
    /// Two renderer-global values written between object fields.
    pub renderer_global_state: [u32; 2],
    /// Values at object offsets `0x114` and `0x118`.
    pub runtime_state: [u32; 2],
}

impl Default for MapViewportState {
    fn default() -> Self {
        Self {
            render_bounds: [0; 4],
            viewport_bounds: [0; 4],
            geometry: [0; 24],
            cells: vec![MapViewportCell::default(); MAP_VIEWPORT_CELL_COUNT],
            selected_tile_index: u32::MAX,
            renderer_global_state: [0; 2],
            runtime_state: [0; 2],
        }
    }
}

impl MapViewportState {
    pub fn read_from<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut state = Self::default();
        for value in &mut state.render_bounds {
            *value = reader.read_u32::<LittleEndian>()?;
        }
        for value in &mut state.viewport_bounds {
            *value = reader.read_u32::<LittleEndian>()?;
        }
        for value in &mut state.geometry {
            *value = reader.read_u32::<LittleEndian>()?;
        }
        for cell in &mut state.cells {
            cell.screen_x = reader.read_u32::<LittleEndian>()?;
            cell.screen_y = reader.read_u32::<LittleEndian>()?;
            cell.map_x = reader.read_u32::<LittleEndian>()?;
            cell.map_y = reader.read_u32::<LittleEndian>()?;
            cell.map_tile_index = reader.read_u32::<LittleEndian>()?;
        }
        state.selected_tile_index = reader.read_u32::<LittleEndian>()?;
        for value in &mut state.renderer_global_state {
            *value = reader.read_u32::<LittleEndian>()?;
        }
        for value in &mut state.runtime_state {
            *value = reader.read_u32::<LittleEndian>()?;
        }
        Ok(state)
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
        for value in self.render_bounds {
            writer.write_u32::<LittleEndian>(value)?;
        }
        for value in self.viewport_bounds {
            writer.write_u32::<LittleEndian>(value)?;
        }
        for value in self.geometry {
            writer.write_u32::<LittleEndian>(value)?;
        }
        for cell in &self.cells {
            writer.write_u32::<LittleEndian>(cell.screen_x)?;
            writer.write_u32::<LittleEndian>(cell.screen_y)?;
            writer.write_u32::<LittleEndian>(cell.map_x)?;
            writer.write_u32::<LittleEndian>(cell.map_y)?;
            writer.write_u32::<LittleEndian>(cell.map_tile_index)?;
        }
        writer.write_u32::<LittleEndian>(self.selected_tile_index)?;
        for value in self.renderer_global_state {
            writer.write_u32::<LittleEndian>(value)?;
        }
        for value in self.runtime_state {
            writer.write_u32::<LittleEndian>(value)?;
        }
        Ok(())
    }

    /// Serialize this state for inspection in the raw-hex viewer.
    ///
    /// Unlike [`Self::write_to`], this retains every supplied cell so the
    /// viewer can show malformed in-memory data before serialization rejects it.
    pub fn raw_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(148 + self.cells.len() * 20);
        for value in self.render_bounds {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in self.viewport_bounds {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in self.geometry {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for cell in &self.cells {
            bytes.extend_from_slice(&cell.screen_x.to_le_bytes());
            bytes.extend_from_slice(&cell.screen_y.to_le_bytes());
            bytes.extend_from_slice(&cell.map_x.to_le_bytes());
            bytes.extend_from_slice(&cell.map_y.to_le_bytes());
            bytes.extend_from_slice(&cell.map_tile_index.to_le_bytes());
        }
        bytes.extend_from_slice(&self.selected_tile_index.to_le_bytes());
        for value in self.renderer_global_state {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in self.runtime_state {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes
    }
}
