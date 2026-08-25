// Map module – public API surface
//
// The former monolithic `map.rs` has been split into focused sub-modules:
//
//  types.rs        – Coords, EventBlock, SpriteInfoBlock, TiledObjectInfo,
//                    coordinate constants and helpers
//  model.rs        – MapModel struct and geometry parser (read_map_model)
//  reader.rs       – Binary block readers for the native .map file format
//  render.rs       – Isometric rendering pipeline (ground / objects / roofs,
//                    sprite on bitmap)
//  sprite_loader.rs– LoadedSpriteFrame, load_sprite_frames, plot_entity_sprite
//  tileset.rs      – Tileset extraction, tile plotting, and atlas generation

// ===========================================================================
// DISPEL GAME MAP FILE FORMAT (.MAP)
// ===========================================================================
//
// ASCII Diagram of File Structure:
//
// +------------------------------+
// | MAP FILE HEADER (12 bytes)  |
// | - Width in chunks (i32)     |
// | - Height in chunks (i32)    |
// | - Border count (i32)        |
// |  (always 2 in practice)     |
// +------------------------------+
// | FIRST BLOCK (variable)      |
// | - Count (i32)               |
// | - Data: (count-1)*8 bytes   |
// |  (count-1 records of 2×i32; |
// |   value2 = linear tile index|
// |   into the three end grids, |
// |   used by tiled-object     |
// |   rendering)               |
// +------------------------------+
// | SECOND BLOCK (variable)      |
// | - Size (i32)                 |
// | - Data: size*2 (u16 table)   |
// |  (lookup table for the end-  |
// |   grid BTL overlay refs:     |
// |  hi byte = draw-enable flag, |
// |  lo byte = transparency mode)|
// +------------------------------+
// | SPRITE BLOCK                 |
// | - Sprite count (i32)         |
// | For each sprite:            |
// |   - Image stamp (i32)       |
// |   - 264 bytes metadata       |
// |   - Sequence info           |
// |   - Pixel data              |
// +------------------------------+
// | SPRITE INFO BLOCK           |
// | - Placement count (i32)      |
// | For each placement:         |
// |   - Sprite ID (i32)         |
// |   - Frame-0 bbox in map px  |
// |     {left, top, right,      |
// |      bottom} + dup {x, y}   |
// |   - (frame_count-1)*24 bytes|
// +------------------------------+
// | TILED OBJECTS BLOCK         |
// | - Bundle count (i32)        |
// | For each bundle:            |
// |   - 264 bytes metadata      |
// |   - Control words + params  |
// |   - Anchor (x,y) map px     |
// |   - BTL tile stack IDs      |
// |   - Building definition    |
// +------------------------------+
// | ... (file continues) ...    |
// +------------------------------+
// | EVENT GRID (near end)       |
// | For each tile: packed u32   |
// |   bits 0-13  event id       |
// |   bit  22    tile marked    |
// |   remainder  unmapped       |
// +------------------------------+
// | TILE & ACCESS GRID          |
// | For each tile: packed u32   |
// |   bit 0     collision       |
// |   bits 1-9  object slot id  |
// |   bits 10-24 GTL tile index |
// +------------------------------+
// | ACCESS-REF GRID ("roof")    |
// | For each tile: packed u32   |
// |   bits 0-14  overlay id →   |
// |              second block   |
// |   bits 15-29 shadow level   |
// |   bits 30-31 light flags    |
// +------------------------------+
//
// COORDINATE SYSTEM:
// - Chunk-based: 1 chunk = 25×25 tiles
// - Isometric coordinates: (x,y) tile positions
// - Tile size: 32×32 pixels
// - Offsets: TILE_HORIZONTAL_OFFSET_HALF=32, TILE_HEIGHT_HALF=16
//
// FILE SIZE CALCULATION:
// Total size = header + blocks + width×height×12 (three packed-u32 grids)
//
pub mod fogdata;
pub mod model;
pub mod reader;
pub mod render;
pub mod sprite_loader;
pub mod tileset;
pub mod tmx;
pub mod types;
pub mod writer;

// ── Re-export the entire public surface so external code needs no changes ──
pub use self::fogdata::{FogData, save_to_db as save_fog_to_db};
pub use model::{MapModel, read_map_model};
pub use render::{EntityRenderInfo, ExternalEntities, LayerToggles};
pub use types::{
    Coords, EventBlock, SpriteInfoBlock, TILE_HEIGHT_HALF, TILE_HORIZONTAL_OFFSET_HALF,
    TILE_PIXEL_NUMBER, TILE_WIDTH_HALF, TiledObjectInfo, convert_map_coords_to_image_coords,
};

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor, Seek, SeekFrom};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};
use image::RgbaImage;
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder, Rgba};

use crate::sprite::{SequenceInfo, rgb16_565_produce_color};
use rusqlite::{Connection, Result as DbResult, params};
use serde::{Deserialize, Serialize};

/// IO Result type for file operations
type IoResult<T> = std::io::Result<T>;

use reader::{
    read_events_block, read_overlay_id_table, read_roof_tiles, read_tiled_object_refs,
    read_tiles_and_access_block, sprite_block, sprite_info_block, tiled_objects_block,
};
use render::{MapRenderConfig, render_map};
use types::TiledObjectMetadata;

// --------------------------------------------------------------------------
// MapData – the in-memory representation of a parsed .map file
// --------------------------------------------------------------------------

pub struct MapData {
    pub model: MapModel,
    pub gtl_tiles: HashMap<Coords, i32>,
    pub btl_tiles: HashMap<Coords, i32>,
    /// Per-overlay-id `{lo: transparency mode, hi: draw-enable}` table from
    /// the map file; drives roof rendering (see `render::plot_roofs`).
    pub overlay_modes: Vec<u16>,
    /// Raw packed u32 words of the access-ref grid ("roof" block), preserved
    /// so saving keeps shadow levels and light flags intact. Overlay refs are
    /// patched from `btl_tiles` on write.
    pub access_ref_words: HashMap<Coords, u32>,
    pub collisions: HashMap<Coords, bool>,
    pub events: HashMap<Coords, EventBlock>,
    pub object_ids: HashMap<Coords, i32>,
    pub tiled_infos: Vec<TiledObjectInfo>,
    pub internal_sprites: Vec<SequenceInfo>,
    pub sprite_blocks: Vec<SpriteInfoBlock>,
    /// Image stamp per embedded sprite (6 or 9), parallel to
    /// `internal_sprites` — same length, same order.
    pub internal_sprite_stamps: Vec<i32>,
    /// First-block tiled-object ref records `(value0, value1)` in file order.
    pub tiled_object_refs: Vec<(i32, i32)>,
    /// Per-bundle retained metadata, parallel to `tiled_infos` — same length,
    /// same order.
    pub tiled_object_metadata: Vec<TiledObjectMetadata>,
}

/// JSON-serializable representation of map data.
/// Converts HashMap-based fields to arrays for JSON compatibility.
#[derive(Serialize, Deserialize)]
pub struct MapDataJson {
    pub metadata: MapMetadataJson,
    pub gtl_tiles: Vec<TileEntryJson>,
    pub btl_tiles: Vec<TileEntryJson>,
    pub collisions: Vec<CollisionEntryJson>,
    pub events: Vec<EventEntryJson>,
    pub object_ids: Vec<TileEntryJson>,
    pub tiled_objects: Vec<TiledObjectJson>,
    pub sprites: Vec<SpritePlacementJson>,
    pub internal_sprites: Vec<InternalSpriteJson>,
}

#[derive(Serialize, Deserialize)]
pub struct MapMetadataJson {
    pub chunk_width: i32,
    pub chunk_height: i32,
    pub tiled_width: i32,
    pub tiled_height: i32,
    pub map_width_in_pixels: i32,
    pub map_height_in_pixels: i32,
    pub non_occluded_start_x: i32,
    pub non_occluded_start_y: i32,
    pub occluded_width: i32,
    pub occluded_height: i32,
}

#[derive(Serialize, Deserialize)]
pub struct TileEntryJson {
    pub x: i32,
    pub y: i32,
    pub tile_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct CollisionEntryJson {
    pub x: i32,
    pub y: i32,
    pub blocked: bool,
}

#[derive(Serialize, Deserialize)]
pub struct EventEntryJson {
    pub x: i32,
    pub y: i32,
    pub event_id: i16,
}

#[derive(Serialize, Deserialize)]
pub struct TiledObjectJson {
    pub index: usize,
    pub x: i32,
    pub y: i32,
    pub tile_ids: Vec<i16>,
}

#[derive(Serialize, Deserialize)]
pub struct SpritePlacementJson {
    pub index: usize,
    pub sprite_id: usize,
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize)]
pub struct InternalSpriteJson {
    pub index: usize,
    pub image_stamp: i32,
    pub frame_count: usize,
    pub frames: Vec<SpriteFrameJson>,
}

#[derive(Serialize, Deserialize)]
pub struct SpriteFrameJson {
    pub width: i32,
    pub height: i32,
    pub origin_x: i32,
    pub origin_y: i32,
}

impl MapData {
    /// Iterate tiles carrying a light level, yielding `(coords, level)` with
    /// levels in `1..=199`.
    ///
    /// The level lives in bits 15–29 of the tile's access-ref word and
    /// selects a brightness pattern from `ExtraInGame/fogdata.dat` (higher
    /// levels are generally brighter, not darker). Level 0 tiles are skipped
    /// here — on maps flagged Dark in AllMap.ini they render fully black;
    /// see [`render::plot_shadows`]. Levels ≥ 200 fall outside the lighting
    /// pass and are skipped as well.
    pub fn shadow_levels(&self) -> impl Iterator<Item = (Coords, u8)> + '_ {
        self.access_ref_words.iter().filter_map(|(&coords, &word)| {
            let level = (word >> 15) & 0x7FFF;
            (level != 0 && level < 200).then_some((coords, level as u8))
        })
    }

    /// Convert MapData to JSON-serializable format.
    pub fn to_json(&self) -> MapDataJson {
        MapDataJson {
            metadata: MapMetadataJson {
                chunk_width: (self.model.tiled_map_width + 1) / 25,
                chunk_height: (self.model.tiled_map_height + 1) / 25,
                tiled_width: self.model.tiled_map_width,
                tiled_height: self.model.tiled_map_height,
                map_width_in_pixels: self.model.map_width_in_pixels,
                map_height_in_pixels: self.model.map_height_in_pixels,
                non_occluded_start_x: self.model.map_non_occluded_start_x,
                non_occluded_start_y: self.model.map_non_occluded_start_y,
                occluded_width: self.model.occluded_map_in_pixels_width,
                occluded_height: self.model.occluded_map_in_pixels_height,
            },
            gtl_tiles: self
                .gtl_tiles
                .iter()
                .map(|(&(x, y), &tile_id)| TileEntryJson { x, y, tile_id })
                .collect(),
            btl_tiles: self
                .btl_tiles
                .iter()
                .map(|(&(x, y), &tile_id)| TileEntryJson { x, y, tile_id })
                .collect(),
            collisions: self
                .collisions
                .iter()
                .map(|(&(x, y), &blocked)| CollisionEntryJson { x, y, blocked })
                .collect(),
            events: self
                .events
                .iter()
                .map(|(&(x, y), event)| EventEntryJson {
                    x,
                    y,
                    event_id: event.event_id() as i16,
                })
                .collect(),
            object_ids: self
                .object_ids
                .iter()
                .map(|(&(x, y), &tile_id)| TileEntryJson { x, y, tile_id })
                .collect(),
            tiled_objects: self
                .tiled_infos
                .iter()
                .enumerate()
                .map(|(index, obj)| TiledObjectJson {
                    index,
                    x: obj.x,
                    y: obj.y,
                    tile_ids: obj.ids.clone(),
                })
                .collect(),
            sprites: self
                .sprite_blocks
                .iter()
                .enumerate()
                .map(|(index, sp)| SpritePlacementJson {
                    index,
                    sprite_id: sp.sprite_id,
                    x: sp.sprite_x,
                    y: sp.sprite_y,
                })
                .collect(),
            internal_sprites: self
                .internal_sprites
                .iter()
                .enumerate()
                .map(|(index, seq)| InternalSpriteJson {
                    index,
                    image_stamp: 0,
                    frame_count: seq.frame_infos.len(),
                    frames: seq
                        .frame_infos
                        .iter()
                        .map(|f| SpriteFrameJson {
                            width: f.width,
                            height: f.height,
                            origin_x: f.origin_x,
                            origin_y: f.origin_y,
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

// --------------------------------------------------------------------------
// Top-level .map file parser
// --------------------------------------------------------------------------

/// Reads a complete `.map` file and returns all its data.
///
/// This is the core parsing function that understands the complete Dispel .MAP file format.
/// It reads all blocks sequentially, handling the isometric coordinate system and
/// converting binary data into structured Rust types.
///
/// # Arguments
/// * `reader` - Buffered file reader positioned at the start of a .MAP file
///
/// # Returns
/// Result containing MapData structure with all parsed components, or I/O/parsing errors
///
/// # Parsing Process
/// The function reads these blocks in order:
/// 1. Map model header to determine dimensions
/// 2. Object-ref records + overlay-id table (skipped; see
///    `reader::skip_tiled_object_refs` / `reader::skip_overlay_id_table`)
/// 3. Sprite block with embedded animation sequences
/// 4. Sprite placement information (frame bounding boxes in map pixels)
/// 5. Tiled objects (building definitions)
/// 6. Event grid (packed u32 per tile, read from end of file)
/// 7. Tile & access grid (GTL index + access bits per tile)
/// 8. Access-ref grid (BTL overlay ref + shadow level per tile)
///
/// The parser handles the isometric coordinate system and converts tile coordinates
/// to the internal (x,y) format used throughout the codebase.
///
/// # Coordinate System
/// Uses a chunk-based system where:
/// - 1 chunk = 25×25 tiles
/// - Tiles are 32×32 pixels with isometric offsets
/// - Coordinates use (x,y) tile positions
/// - Conversion to pixels uses TILE_HORIZONTAL_OFFSET_HALF (32) and TILE_HEIGHT_HALF (16)
pub fn read_map_data(reader: &mut BufReader<File>) -> IoResult<MapData> {
    let file_len = reader.get_ref().metadata()?.len();
    let map_model = read_map_model(reader)?;
    let tiled_map_width = map_model.tiled_map_width;
    let tiled_map_height = map_model.tiled_map_height;

    // First-block refs are now read (previously skipped) — see
    // `reader::read_tiled_object_refs`.
    let tiled_object_refs = read_tiled_object_refs(reader)?;
    let overlay_modes = read_overlay_id_table(reader)?;

    let (internal_sprites, internal_sprite_stamps) = sprite_block(reader)?;
    let sprite_blocks = sprite_info_block(reader, &internal_sprites)?;
    let (tiled_infos, tiled_object_metadata) = tiled_objects_block(reader)?;

    // Event and tile blocks live at the end of the file
    // Calculate expected size for the three end blocks
    let expected_end_blocks_size = tiled_map_height * tiled_map_width * 4 * 3;

    // Validate that we have enough data for the end blocks
    if file_len < expected_end_blocks_size as u64 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "File too small for expected map dimensions. File size: {}, Expected end blocks: {}",
                file_len, expected_end_blocks_size
            ),
        ));
    }

    let skip = -(expected_end_blocks_size as i64);
    reader.seek(SeekFrom::End(skip))?;

    let events = read_events_block(reader, tiled_map_width, tiled_map_height)?;
    let (gtl_tiles, collisions, object_ids) =
        read_tiles_and_access_block(reader, tiled_map_width, tiled_map_height)?;

    let mut btl_tiles = HashMap::new();
    let mut access_ref_words = HashMap::new();
    let current_pos = reader.stream_position()?;
    let remaining_bytes = file_len - current_pos;
    let expected_roof_size = (tiled_map_width * tiled_map_height * 4) as u64;

    // Only read the access-ref grid if we have exactly the expected amount of
    // data remaining
    if remaining_bytes >= expected_roof_size {
        let (refs, words) = read_roof_tiles(reader, tiled_map_width, tiled_map_height)?;
        btl_tiles = refs;
        access_ref_words = words;
        let _ = &overlay_modes;
    } else if remaining_bytes > 0 {
        // If there are remaining bytes but not enough for a full grid,
        // this might indicate a different file structure or corruption
        eprintln!(
            "Warning: Found {} bytes after tile blocks, expected {} for the access-ref grid. Skipping.",
            remaining_bytes, expected_roof_size
        );
    }

    Ok(MapData {
        model: map_model,
        gtl_tiles,
        btl_tiles,
        overlay_modes,
        access_ref_words,
        collisions,
        events,
        object_ids,
        tiled_infos,
        internal_sprites,
        sprite_blocks,
        internal_sprite_stamps,
        tiled_object_refs,
        tiled_object_metadata,
    })
}

// --------------------------------------------------------------------------
// CLI commands
// --------------------------------------------------------------------------

/// Renders a map from binary files to a PNG image.
///
/// This function processes the complete Dispel game map file format, which contains:
/// - Map geometry and dimensions in the header
/// - Embedded sprites and their placement information
/// - Tiled objects (buildings made from stacked BTL tiles)
/// - Event triggers tied to specific map coordinates
/// - Ground tiles (GTL) with collision data
/// - Building/roof tiles (BTL) for structures
///
/// The map uses an isometric coordinate system with 25×25 tile chunks and
/// stores data in distinct blocks that are read sequentially from the file.
///
/// # Arguments
/// * `input_map_file` - Path to the .MAP file containing map geometry and objects
/// * `input_btl_file` - Path to the .BTL file containing building/roof tileset
/// * `input_gtl_file` - Path to the .GTL file containing ground tileset
/// * `output_path` - Path where the rendered PNG will be saved
/// * `save_map_sprites` - Whether to extract embedded sprites to separate files
///
/// # Returns
/// Result containing any I/O or parsing errors that may occur
///
/// # Map File Structure
/// The .MAP file format consists of these main blocks:
/// 1. Header: Map dimensions in chunks (25-tile units)
/// 2. Object-ref records + overlay-id table: skipped during processing
/// 3. Sprite block: Embedded sprite sequences and metadata
/// 4. Sprite info: Frame bounding boxes for placed sprites
/// 5. Tiled objects: Building definitions using stacked tiles
/// 6. Event grid: Packed u32 per tile (event id + flags)
/// 7. Tile & access: Packed u32 per tile (GTL index + access bits)
/// 8. Access-ref grid: BTL overlay refs and shadow levels per tile
///
/// Coordinates use an isometric system where each tile is 32×32 pixels,
/// with special offsets for proper isometric rendering.
pub fn extract(
    input_map_file: &Path,
    input_btl_file: &Path,
    input_gtl_file: &Path,
    output_path: &Path,
    save_map_sprites: bool,
    game_path: Option<&Path>,
    toggles: LayerToggles,
) -> IoResult<()> {
    let file = File::open(input_map_file)?;
    let mut reader = BufReader::new(file);
    let map_data = read_map_data(&mut reader)?;
    let map_id = input_map_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("map");

    if save_map_sprites {
        for (i, sprite) in map_data.internal_sprites.iter().enumerate() {
            crate::sprite::save_sequence(&mut reader, &sprite.frame_infos, i as i32, map_id)?;
        }
    }

    let btl_tileset = tileset::extract(input_btl_file)?;
    let gtl_tileset = tileset::extract(input_gtl_file)?;

    render_map(MapRenderConfig {
        reader: &mut reader,
        output_path,
        data: &map_data,
        gtl_tileset: &gtl_tileset,
        btl_tileset: &btl_tileset,
        map_id,
        game_path,
        toggles,
        lights: &[],
    })
}

/// Extracts all internal sprites from a map file to separate PNGs.
///
/// This function focuses on the sprite-related blocks within the .MAP file:
/// - Sprite block: Contains embedded sprite sequences with animation frames
/// - Sprite info block: Contains placement coordinates for each sprite
///
/// The sprites are stored as sequences with metadata including frame count,
/// animation timing, and pixel data. Each sprite has an associated placement
/// record that specifies its exact position on the map.
///
/// # Arguments
/// * `input_map_file` - Path to the .MAP file containing embedded sprites
/// * `output_path` - Directory where individual sprite PNGs will be saved
///
/// # Returns
/// Result containing any I/O or parsing errors
///
/// # Sprite Data Structure
/// Each sprite in the map consists of:
/// - Image stamp (6 or 9) determining data layout
/// - 264 bytes of metadata
/// - Sequence info with frame count and positions
/// - Pixel data for each animation frame
///
/// Sprites are extracted with their original animation sequences preserved,
/// allowing for proper reconstruction of the observed animations.
pub fn extract_sprites(input_map_file: &Path, output_path: &Path) -> IoResult<()> {
    let file = File::open(input_map_file)?;
    let mut reader = BufReader::new(file);
    let map_data = read_map_data(&mut reader)?;
    let map_id = input_map_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("map");

    std::fs::create_dir_all(output_path)?;
    let output_dir_str = output_path.to_str().unwrap_or("out");

    for (i, sprite) in map_data.internal_sprites.iter().enumerate() {
        let prefix = format!("{}/{}", output_dir_str, map_id);
        crate::sprite::save_sequence(&mut reader, &sprite.frame_infos, i as i32, &prefix)?;
    }
    Ok(())
}

/// Imports a `.map` file into the SQLite database.
///
/// This function parses the complete .MAP file structure and saves all
/// components to a structured database format for later retrieval and rendering.
///
/// The database import preserves the hierarchical structure of the map:
/// - Map metadata (dimensions, computed pixel sizes)
/// - Tile layers (ground GTL tiles and building BTL tiles)
/// - Collision data for pathfinding and game logic
/// - Event triggers for interactive elements
/// - Object placements (buildings made from tile stacks)
/// - Sprite information for dynamic elements
///
/// # Arguments
/// * `database_path` - Path to the SQLite database file
/// * `map_path` - Path to the .MAP file to import
///
/// # Returns
/// Result containing any I/O, parsing, or database errors
///
/// # Database Schema
/// The function creates or updates these database tables:
/// - map_metadata: Map dimensions and computed sizes
/// - map_tiles: Ground and building tiles with coordinates
/// - map_collisions: Tile collision flags
/// - map_events: Event trigger information
/// - map_objects: Tiled object definitions
/// - map_sprites: Sprite placement and sequence data
///
/// This allows for efficient querying and rendering of map components
/// without needing to re-parse the binary format each time.
pub fn import_to_database(database_path: &Path, map_path: &Path) -> IoResult<()> {
    use rusqlite::Connection;
    let mut conn =
        Connection::open(database_path).map_err(|e| std::io::Error::other(e.to_string()))?;

    let file = File::open(map_path)?;
    let mut reader = BufReader::new(file);
    let map_data = read_map_data(&mut reader)?;
    let map_id = map_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("map");

    save_to_db(&mut conn, map_id, &map_data, &mut reader)
        .map_err(|e| std::io::Error::other(e.to_string()))
}

/// Writes map data to the SQLite database.
///
/// This low-level function takes parsed MapData and persists it to the database.
/// It handles the conversion from in-memory structures to the relational format.
///
/// # Arguments
/// * `conn` - Active SQLite database connection
/// * `map_id` - Identifier for the map (e.g., "cat1", "dun01")
/// * `data` - Parsed MapData structure containing all map components
///
/// # Returns
/// Result containing any database operation errors
///
/// # Data Conversion Process
/// The function converts these in-memory structures to database records:
/// - MapModel → map_metadata table
/// - GTL/BTL tiles → map_tiles table with layer distinction
/// - Collisions → map_collisions table with boolean flags
/// - Events → map_events table with trigger IDs
/// - TiledObjectInfo → map_objects table with tile stacks
/// - SpriteInfoBlock → map_sprites table with positions
///
/// This creates a complete, queryable representation of the original
/// binary map file in a relational database format.
pub fn save_to_db(
    conn: &mut rusqlite::Connection,
    map_id: &str,
    data: &MapData,
    reader: &mut BufReader<File>,
) -> DbResult<()> {
    println!("Saving map tiles for {}...", map_id);
    save_map_tiles(SaveMapTilesParams {
        conn,
        map_id,
        gtl_tiles: &data.gtl_tiles,
        btl_tiles: &data.btl_tiles,
        collisions: &data.collisions,
        events: &data.events,
        access_ref_words: &data.access_ref_words,
        object_ids: &data.object_ids,
        width: data.model.tiled_map_width,
        height: data.model.tiled_map_height,
    })?;

    save_map_objects(conn, map_id, &data.tiled_infos)?;

    save_map_sprites(conn, map_id, &data.sprite_blocks)?;

    save_map_sprite_frames(conn, map_id, &data.internal_sprites, reader)?;

    save_map_metadata(conn, map_id, &data.model)?;

    save_map_overlay_modes(conn, map_id, &data.overlay_modes)?;

    save_map_sprite_sequences(conn, map_id, &data.internal_sprites)?;

    save_map_object_refs(conn, map_id, &data.tiled_object_refs)?;

    save_map_object_metadata(conn, map_id, &data.tiled_infos, &data.tiled_object_metadata)?;

    Ok(())
}

pub struct SaveMapTilesParams<'a> {
    pub conn: &'a mut Connection,
    pub map_id: &'a str,
    pub gtl_tiles: &'a HashMap<Coords, i32>,
    pub btl_tiles: &'a HashMap<Coords, i32>,
    pub collisions: &'a HashMap<Coords, bool>,
    pub events: &'a HashMap<Coords, EventBlock>,
    pub access_ref_words: &'a HashMap<Coords, u32>,
    pub object_ids: &'a HashMap<Coords, i32>,
    pub width: i32,
    pub height: i32,
}

pub fn save_map_tiles(params: SaveMapTilesParams) -> DbResult<()> {
    let conn = params.conn;
    let map_id = params.map_id;
    let gtl_tiles = params.gtl_tiles;
    let btl_tiles = params.btl_tiles;
    let collisions = params.collisions;
    let events = params.events;
    let access_ref_words = params.access_ref_words;
    let object_ids = params.object_ids;
    let width = params.width;
    let height = params.height;

    let tx = conn.transaction()?;

    let offset_x = width / 2;
    let offset_y = height / 2;

    println!(
        "Inserting map tiles for map {}, width {}, height {}",
        map_id, width, height
    );

    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_map_tile.sql"))?;

        for y in 0..height {
            for x in 0..width {
                let coords = (x, y);
                let gtl_id = gtl_tiles.get(&coords).cloned().unwrap_or(0);
                let btl_id = btl_tiles.get(&coords).cloned().unwrap_or(0);
                let collision = collisions.get(&coords).cloned().unwrap_or(false);
                let event_id = events.get(&coords).map(|e| e.event_id()).unwrap_or(0);

                // Dense grid: every (x, y) is written so access/shadow data of
                // otherwise "empty" tiles survives the round-trip.
                let event_word = events
                    .get(&coords)
                    .map(|e| e.word as i64)
                    .unwrap_or_default();
                let marked = events
                    .get(&coords)
                    .map(|e| e.is_tile_marked())
                    .unwrap_or_default();
                let object_id = object_ids.get(&coords).copied();
                let raw_ref = access_ref_words.get(&coords).copied().unwrap_or(0);
                let shadow_level = ((raw_ref >> 15) & 0x7FFF) as i64;
                let light_flags = ((raw_ref >> 30) & 0x3) as i64;
                let access_ref_word = raw_ref as i64;

                stmt.execute(params![
                    map_id,
                    x - offset_x,
                    y - offset_y,
                    gtl_id,
                    btl_id,
                    collision,
                    event_id as i32,
                    event_word,
                    marked,
                    object_id,
                    shadow_level,
                    light_flags,
                    access_ref_word,
                ])?;
            }
        }
    }

    tx.commit()?;
    Ok(())
}

pub fn save_map_objects(
    conn: &mut Connection,
    map_id: &str,
    tiled_infos: &[TiledObjectInfo],
) -> DbResult<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_map_object.sql"))?;
        for (obj_idx, info) in tiled_infos.iter().enumerate() {
            for (stack_order, btl_id) in info.ids.iter().enumerate() {
                stmt.execute(params![
                    map_id,
                    obj_idx as i32,
                    info.x,
                    info.y,
                    *btl_id as i32,
                    stack_order as i32,
                ])?;
            }
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn save_map_sprites(
    conn: &mut Connection,
    map_id: &str,
    sprite_blocks: &[SpriteInfoBlock],
) -> DbResult<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_map_sprite.sql"))?;
        for (sprite_idx, block) in sprite_blocks.iter().enumerate() {
            stmt.execute(params![
                map_id,
                sprite_idx as i32,
                block.sprite_x,
                block.sprite_y,
                block.sprite_id as i32,
                block.sprite_bottom_right_y,
                block.bbox_left,
                block.bbox_top,
                block.bbox_right,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn save_map_sprite_frames(
    conn: &mut Connection,
    map_id: &str,
    internal_sprites: &[SequenceInfo],
    reader: &mut BufReader<File>,
) -> DbResult<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_map_sprite_frame.sql"))?;
        for (seq_idx, seq) in internal_sprites.iter().enumerate() {
            for (frame_idx, frame) in seq.frame_infos.iter().enumerate() {
                // Seek to this frame's RGB565 pixel data in the .map file
                reader
                    .seek(SeekFrom::Start(frame.image_start_position))
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                // Decode RGB565 → RGBA image
                let mut rgba = RgbaImage::new(frame.width as u32, frame.height as u32);
                for y in 0..frame.height {
                    for x in 0..frame.width {
                        let pixel = reader
                            .read_u16::<LittleEndian>()
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                        if pixel == 0 {
                            continue; // transparent
                        }
                        let color = rgb16_565_produce_color(pixel);
                        rgba.put_pixel(x as u32, y as u32, Rgba([color.r, color.g, color.b, 255]));
                    }
                }

                // Encode frame as PNG blob
                let mut png_buf = Vec::new();
                let encoder = PngEncoder::new(Cursor::new(&mut png_buf));
                encoder
                    .write_image(
                        rgba.as_raw(),
                        frame.width as u32,
                        frame.height as u32,
                        ColorType::Rgba8,
                    )
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                stmt.execute(params![
                    map_id,
                    seq_idx as i32,
                    frame_idx as i32,
                    png_buf,
                    frame.width,
                    frame.height,
                    frame.origin_x,
                    frame.origin_y,
                    frame.size_bytes,
                    frame.image_start_position as i64,
                ])?;
            }
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn save_map_metadata(conn: &mut Connection, map_id: &str, model: &MapModel) -> DbResult<()> {
    conn.execute(
        include_str!("../queries/insert_map_metadata.sql"),
        params![
            map_id,
            model.tiled_map_width,
            model.tiled_map_height,
            model.map_width_in_pixels,
            model.map_height_in_pixels,
            model.map_non_occluded_start_x,
            model.map_non_occluded_start_y,
            model.occluded_map_in_pixels_width,
            model.occluded_map_in_pixels_height,
        ],
    )?;

    Ok(())
}

/// Persists the overlay-id lookup table: one row per u16 entry, with the
/// low byte decoded as transparency mode and the high byte as draw-enable.
pub fn save_map_overlay_modes(
    conn: &mut Connection,
    map_id: &str,
    overlay_modes: &[u16],
) -> DbResult<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_map_overlay_mode.sql"))?;
        for (index, &raw) in overlay_modes.iter().enumerate() {
            stmt.execute(params![
                map_id,
                index as i32,
                raw as i32,
                (raw & 0xFF) as i32,
                (raw >> 8) as i32,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

/// Persists sequence-level info for each embedded internal sprite
/// (file offsets + frame count).
pub fn save_map_sprite_sequences(
    conn: &mut Connection,
    map_id: &str,
    internal_sprites: &[SequenceInfo],
) -> DbResult<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_map_sprite_sequence.sql"))?;
        for (seq_idx, seq) in internal_sprites.iter().enumerate() {
            stmt.execute(params![
                map_id,
                seq_idx as i32,
                seq.sequence_start_position as i64,
                seq.sequence_end_position as i64,
                seq.frame_count,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

/// Persists the first-block tiled-object ref records in file order.
pub fn save_map_object_refs(
    conn: &mut Connection,
    map_id: &str,
    refs: &[(i32, i32)],
) -> DbResult<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_map_object_ref.sql"))?;
        for (ref_index, &(value0, value1)) in refs.iter().enumerate() {
            stmt.execute(params![map_id, ref_index as i32, value0, value1])?;
        }
    }
    tx.commit()?;
    Ok(())
}

/// Persists per-bundle retained metadata from the tiled objects block.
///
/// Defensive guard: if `metadata.len() != tiled_infos.len()` (should not
/// happen — both come from one parse pass), whatever exists is still written
/// indexed positionally and the export continues.
pub fn save_map_object_metadata(
    conn: &mut Connection,
    map_id: &str,
    _tiled_infos: &[TiledObjectInfo],
    metadata: &[TiledObjectMetadata],
) -> DbResult<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_map_object_metadata.sql"))?;
        for (obj_idx, meta) in metadata.iter().enumerate() {
            stmt.execute(params![
                map_id,
                obj_idx as i32,
                meta.metadata_blob,
                meta.control_0,
                meta.control_1,
                meta.control_2,
                meta.control_3,
                meta.param_0,
                meta.param_1,
                meta.param_2,
                meta.param_3,
                meta.param_4,
                meta.param_5,
                meta.extra_count_a,
                meta.extra_count_b,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod shadow_tests {
    use super::*;

    #[test]
    fn test_shadow_levels_decodes_bits_15_to_29() {
        let mut data = MapData {
            model: MapModel::default(),
            gtl_tiles: HashMap::new(),
            btl_tiles: HashMap::new(),
            overlay_modes: Vec::new(),
            access_ref_words: HashMap::new(),
            collisions: HashMap::new(),
            events: HashMap::new(),
            object_ids: HashMap::new(),
            tiled_infos: Vec::new(),
            internal_sprites: Vec::new(),
            sprite_blocks: Vec::new(),
            internal_sprite_stamps: Vec::new(),
            tiled_object_refs: Vec::new(),
            tiled_object_metadata: Vec::new(),
        };

        // No shadows yet
        assert_eq!(data.shadow_levels().count(), 0);

        // Overlay ref 5 with shadow level 100, plus light-flag bits set high
        data.access_ref_words
            .insert((1, 0), (100 << 15) | 5 | 0xC000_0000);
        // Level 0 → not yielded
        data.access_ref_words.insert((2, 0), 7);
        // Level 200 → not a darkness level, skipped
        data.access_ref_words.insert((3, 0), 200 << 15);
        // Level 199 → maximum valid darkness
        data.access_ref_words.insert((4, 0), 199 << 15);

        let mut levels: Vec<(Coords, u8)> = data.shadow_levels().collect();
        levels.sort();
        assert_eq!(levels, vec![((1, 0), 100), ((4, 0), 199)]);
    }
}

#[cfg(test)]
mod render_placement_tests;

#[cfg(test)]
mod db_export_tests {
    use super::*;
    use rusqlite::Connection;

    const CAT1_MAP: &str = "fixtures/Dispel/Map/cat1.map";

    fn parse_fixture() -> (MapData, BufReader<File>) {
        let file = File::open(CAT1_MAP).expect("cat1.map fixture must exist");
        let mut reader = BufReader::new(file);
        let data = read_map_data(&mut reader).expect("parsing cat1.map");
        // Rewind to start so save_map_sprite_frames can re-read pixel data.
        reader.seek(SeekFrom::Start(0)).unwrap();
        (data, reader)
    }

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch(include_str!("../queries/create_table_map_tiles.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../queries/create_table_map_objects.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../queries/create_table_map_sprites.sql"))
            .unwrap();
        conn.execute_batch(include_str!(
            "../queries/create_table_map_sprite_frames.sql"
        ))
        .unwrap();
        conn.execute_batch(include_str!("../queries/create_table_map_metadata.sql"))
            .unwrap();
        conn.execute_batch(include_str!(
            "../queries/create_table_map_overlay_modes.sql"
        ))
        .unwrap();
        conn.execute_batch(include_str!(
            "../queries/create_table_map_sprite_sequences.sql"
        ))
        .unwrap();
        conn.execute_batch(include_str!("../queries/create_table_map_object_refs.sql"))
            .unwrap();
        conn.execute_batch(include_str!(
            "../queries/create_table_map_object_metadata.sql"
        ))
        .unwrap();
        conn
    }

    fn count(conn: &Connection, sql: &str) -> i64 {
        conn.query_row(sql, [], |row| row.get(0)).unwrap()
    }

    #[test]
    fn test_db_export_overlay_modes_match_parsed_table() {
        let (data, mut reader) = parse_fixture();
        let mut conn = setup_db();
        save_to_db(&mut conn, "cat1", &data, &mut reader).unwrap();

        assert_eq!(
            count(&conn, "SELECT COUNT(*) FROM map_overlay_modes"),
            data.overlay_modes.len() as i64
        );

        // Spot-check: decoded lo/hi bytes must match the raw u16.
        for idx in [
            0usize,
            data.overlay_modes.len() / 2,
            data.overlay_modes.len() - 1,
        ] {
            let raw = data.overlay_modes[idx];
            let (mode, lo, hi): (i64, i64, i64) = conn
                .query_row(
                    "SELECT mode, transparency_mode, draw_enable FROM map_overlay_modes \
                     WHERE overlay_index = ?1",
                    [idx as i32],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .unwrap();
            assert_eq!(mode, raw as i64);
            assert_eq!(lo, (raw & 0xFF) as i64);
            assert_eq!(hi, (raw >> 8) as i64);
        }
    }

    #[test]
    fn test_db_export_tiles_cover_full_grid_with_decoded_fields() {
        let (data, mut reader) = parse_fixture();
        let mut conn = setup_db();
        save_to_db(&mut conn, "cat1", &data, &mut reader).unwrap();

        let w = data.model.tiled_map_width;
        let h = data.model.tiled_map_height;
        let offset_x = w / 2;
        let offset_y = h / 2;

        assert_eq!(
            count(&conn, "SELECT COUNT(*) FROM map_tiles"),
            (w as i64) * (h as i64),
            "dense grid: every (x, y) must have a row"
        );

        // Sample a deterministic spread of tiles.
        for &(sx, sy) in &[
            (0i32, 0i32),
            (w / 3, h / 3),
            (w - 1, h - 1),
            (0, h - 1),
            (w - 1, 0),
        ] {
            let coords = (sx, sy);
            let tile_row: [Option<i64>; 6] = conn
                .query_row(
                    "SELECT event_word, marked, object_id, shadow_level, light_flags, \
                     access_ref_word FROM map_tiles WHERE x = ?1 AND y = ?2",
                    params![sx - offset_x, sy - offset_y],
                    |row| {
                        Ok([
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                            row.get(5)?,
                        ])
                    },
                )
                .unwrap_or([None, None, None, None, None, None]);
            // unwrap_or would swallow a missing row; require presence:
            if tile_row[0].is_none() && !data.events.contains_key(&coords) {
                panic!("missing tile row at {:?}", coords);
            }

            let event_word = data.events.get(&coords).map(|e| e.word as i64).unwrap_or(0);
            let marked = data
                .events
                .get(&coords)
                .map(|e| e.is_tile_marked())
                .unwrap_or(false);
            let raw_ref = data.access_ref_words.get(&coords).copied().unwrap_or(0);
            let expected_object = data.object_ids.get(&coords).map(|&v| v as i64);

            assert_eq!(
                tile_row[0].unwrap(),
                event_word,
                "event_word at {:?}",
                coords
            );
            assert_eq!(tile_row[1].unwrap() != 0, marked, "marked at {:?}", coords);
            assert_eq!(tile_row[2], expected_object, "object_id at {:?}", coords);
            assert_eq!(
                tile_row[3].unwrap(),
                ((raw_ref >> 15) & 0x7FFF) as i64,
                "shadow_level at {:?}",
                coords
            );
            assert_eq!(
                tile_row[4].unwrap(),
                ((raw_ref >> 30) & 0x3) as i64,
                "light_flags at {:?}",
                coords
            );
            assert_eq!(
                tile_row[5].unwrap(),
                raw_ref as i64,
                "access_ref_word at {:?}",
                coords
            );
        }
    }

    #[test]
    fn test_db_export_object_id_null_exactly_where_absent() {
        let (data, mut reader) = parse_fixture();
        let mut conn = setup_db();
        save_to_db(&mut conn, "cat1", &data, &mut reader).unwrap();

        // Sampled check: every sampled coordinate's NULL-ness matches the map.
        let w = data.model.tiled_map_width;
        let h = data.model.tiled_map_height;
        for &(sx, sy) in &[(0i32, 0i32), (w / 2, h / 2), (w - 1, h - 1)] {
            let coords = (sx, sy);
            let stored: Option<Option<i64>> = conn
                .query_row(
                    "SELECT object_id FROM map_tiles WHERE x = ?1 AND y = ?2",
                    params![sx - w / 2, sy - h / 2],
                    |row| row.get::<_, Option<i64>>(0).map(Some),
                )
                .ok()
                .flatten();
            let stored = stored.expect("tile row must exist");
            assert_eq!(
                stored.as_ref().copied(),
                data.object_ids.get(&coords).map(|&v| v as i64),
                "object_id NULL pattern at {:?}",
                coords
            );
        }
    }

    #[test]
    fn test_db_export_sprites_carry_bottom_right_y_and_bbox() {
        let (data, mut reader) = parse_fixture();
        let mut conn = setup_db();
        save_to_db(&mut conn, "cat1", &data, &mut reader).unwrap();

        for sprite_idx in 0..data.sprite_blocks.len().min(5) {
            let block = &data.sprite_blocks[sprite_idx];
            let (bry, l, t, r): (i64, Option<i64>, Option<i64>, Option<i64>) = conn
                .query_row(
                    "SELECT bottom_right_y, bbox_left, bbox_top, bbox_right FROM map_sprites \
                     WHERE sprite_id = ?1",
                    [sprite_idx as i32],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                )
                .unwrap();
            assert_eq!(bry, block.sprite_bottom_right_y as i64);
            assert_eq!(l, Some(block.bbox_left as i64));
            assert_eq!(t, Some(block.bbox_top as i64));
            assert_eq!(r, Some(block.bbox_right as i64));
        }
    }

    #[test]
    fn test_db_export_sprite_sequences_and_frames_match_parsed_data() {
        let (data, mut reader) = parse_fixture();
        let mut conn = setup_db();
        save_to_db(&mut conn, "cat1", &data, &mut reader).unwrap();

        assert_eq!(
            count(&conn, "SELECT COUNT(*) FROM map_sprite_sequences"),
            data.internal_sprites.len() as i64
        );

        'outer: for (seq_idx, seq) in data.internal_sprites.iter().enumerate() {
            for (frame_idx, frame) in seq.frame_infos.iter().enumerate() {
                let (size_bytes, start_pos, png_len): (i64, i64, i64) = conn
                    .query_row(
                        "SELECT size_bytes, image_start_position, LENGTH(png_blob) \
                         FROM map_sprite_frames WHERE internal_sprite_id = ?1 AND frame_index = ?2",
                        params![seq_idx as i32, frame_idx as i32],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                    )
                    .unwrap();
                assert_eq!(size_bytes, frame.size_bytes);
                assert_eq!(start_pos, frame.image_start_position as i64);
                assert!(png_len > 0, "png_blob must be non-empty");
                if seq_idx >= 2 {
                    break 'outer; // sample enough sequences only
                }
            }
        }
    }

    #[test]
    fn test_db_export_object_refs_and_metadata_match_parsed_data() {
        let (data, mut reader) = parse_fixture();
        let mut conn = setup_db();
        save_to_db(&mut conn, "cat1", &data, &mut reader).unwrap();

        assert_eq!(
            count(&conn, "SELECT COUNT(*) FROM map_object_refs"),
            data.tiled_object_refs.len() as i64
        );
        assert_eq!(
            count(&conn, "SELECT COUNT(*) FROM map_object_metadata"),
            data.tiled_infos.len() as i64,
            "one metadata row per tiled object bundle"
        );
    }

    #[test]
    fn test_db_export_internal_sprite_stamps_are_known_values() {
        let (data, _reader) = parse_fixture();
        assert_eq!(
            data.internal_sprite_stamps.len(),
            data.internal_sprites.len(),
            "stamps parallel to internal_sprites"
        );
        for stamp in &data.internal_sprite_stamps {
            assert!(*stamp == 6 || *stamp == 9, "unexpected stamp {stamp}");
        }
    }
}
