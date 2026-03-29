// Map module – public API surface
//
// The former monolithic `map.rs` has been split into focused sub-modules:
//
//  types.rs        – Coords, EventBlock, SpriteInfoBlock, TiledObjectInfo,
//                    coordinate constants and helpers
//  model.rs        – MapModel struct and geometry parser (read_map_model)
//  reader.rs       – Binary block readers for the native .map file format
//  render.rs       – Isometric rendering pipeline (ground / objects / roofs,
//                    sprite on bitmap, atlas tile blitter)
//  sprite_loader.rs– LoadedSpriteFrame, load_sprite_frames, plot_entity_sprite
//  database.rs     – render_from_database + entity overlay helpers
//  tileset.rs      – Tileset extraction, tile plotting, and atlas generation

// ===========================================================================
// DISPEL GAME MAP FILE FORMAT (.MAP)
// ===========================================================================
//
// ASCII Diagram of File Structure:
//
// +------------------------------+
// | MAP FILE HEADER (8 bytes)   |
// | - Width in chunks (i32)     |
// | - Height in chunks (i32)    |
// +------------------------------+
// | FIRST BLOCK (variable)      |
// | - Multiplier (i32)           |
// | - Size (i32)                 |
// | - Data: multiplier*size*4    |
// |  (unknown purpose, skipped)  |
// +------------------------------+
// | SECOND BLOCK (variable)      |
// | - Size (i32)                 |
// | - Data: size*2               |
// |  (unknown purpose, skipped)  |
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
// |   - Position data           |
// |   - Frame count             |
// +------------------------------+
// | TILED OBJECTS BLOCK         |
// | - Bundle count (i32)        |
// | For each bundle:            |
// |   - 264 bytes metadata      |
// |   - Coordinates (x,y)      |
// |   - Tile stack IDs         |
// |   - Building definition    |
// +------------------------------+
// | ... (file continues) ...    |
// +------------------------------+
// | EVENT BLOCK (near end)      |
// | For each tile (width×height):|
// |   - Event ID (i16)          |
// |   - Unknown (i16)           |
// +------------------------------+
// | TILE & ACCESS BLOCK         |
// | For each tile (width×height):|
// |   - GTL tile ID (i32)      |
// |   - Collision flag         |
// +------------------------------+
// | ROOF TILE BLOCK (optional)   |
// | For each tile (width×height):|
// |   - BTL tile ID (i16)      |
// |   - Flags (i16)            |
// +------------------------------+
//
// COORDINATE SYSTEM:
// - Chunk-based: 1 chunk = 25×25 tiles
// - Isometric coordinates: (x,y) tile positions
// - Tile size: 32×32 pixels
// - Offsets: TILE_HORIZONTAL_OFFSET_HALF=32, TILE_HEIGHT_HALF=16
//
// FILE SIZE CALCULATION:
// Total size = header + blocks + (width×height×(2+4+2)) + optional roof data
//
pub mod database;
pub mod model;
pub mod reader;
pub mod render;
pub mod sprite_loader;
pub mod tileset;
pub mod types;

// ── Re-export the entire public surface so external code needs no changes ──
pub use database::render_from_database;
pub use model::{read_map_model, MapModel};
pub use types::{
    convert_map_coords_to_image_coords, Coords, EventBlock, SpriteInfoBlock, TiledObjectInfo,
    TILE_HEIGHT_HALF, TILE_HORIZONTAL_OFFSET_HALF, TILE_PIXEL_NUMBER, TILE_WIDTH_HALF,
};

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use std::path::Path;

use crate::sprite::SequenceInfo;
use rusqlite::{params, Connection, Result as DbResult};

/// IO Result type for file operations
type IoResult<T> = std::io::Result<T>;

use reader::{
    first_block, read_events_block, read_roof_tiles, read_tiles_and_access_block, second_block,
    sprite_block, sprite_info_block, tiled_objects_block,
};
use render::render_map;

// --------------------------------------------------------------------------
// MapData – the in-memory representation of a parsed .map file
// --------------------------------------------------------------------------

pub struct MapData {
    pub model: MapModel,
    pub gtl_tiles: HashMap<Coords, i32>,
    pub btl_tiles: HashMap<Coords, i32>,
    pub collisions: HashMap<Coords, bool>,
    pub events: HashMap<Coords, EventBlock>,
    pub tiled_infos: Vec<TiledObjectInfo>,
    pub internal_sprites: Vec<SequenceInfo>,
    pub sprite_blocks: Vec<SpriteInfoBlock>,
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
/// 2. Unknown blocks (skipped)
/// 3. Sprite block with embedded animation sequences
/// 4. Sprite placement information
/// 5. Tiled objects (building definitions)
/// 6. Event triggers (read from end of file)
/// 7. Ground tiles and collision data
/// 8. Optional roof/building tiles
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

    first_block(reader)?;
    second_block(reader)?;

    let internal_sprites = sprite_block(reader)?;
    let sprite_blocks = sprite_info_block(reader, &internal_sprites)?;
    let tiled_infos = tiled_objects_block(reader)?;

    // Event and tile blocks live at the end of the file
    let skip = -(tiled_map_height * tiled_map_width * 4 * 3);
    reader.seek(SeekFrom::End(skip.into()))?;

    let events = read_events_block(reader, tiled_map_width, tiled_map_height)?;
    let (gtl_tiles, collisions) =
        read_tiles_and_access_block(reader, tiled_map_width, tiled_map_height)?;

    let mut btl_tiles = HashMap::new();
    let current_pos = reader.stream_position()?;
    if current_pos + (tiled_map_width * tiled_map_height * 4) as u64 <= file_len {
        btl_tiles = read_roof_tiles(reader, tiled_map_width, tiled_map_height)?;
    }

    Ok(MapData {
        model: map_model,
        gtl_tiles,
        btl_tiles,
        collisions,
        events,
        tiled_infos,
        internal_sprites,
        sprite_blocks,
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
/// 2. Unknown blocks: Skipped during processing
/// 3. Sprite block: Embedded sprite sequences and metadata
/// 4. Sprite info: Position data for placed sprites
/// 5. Tiled objects: Building definitions using stacked tiles
/// 6. Event block: Per-tile event trigger IDs
/// 7. Tile & access: Ground tiles with collision flags
/// 8. Roof tiles: Optional building/roof tile layer
///
/// Coordinates use an isometric system where each tile is 32×32 pixels,
/// with special offsets for proper isometric rendering.
pub fn extract(
    input_map_file: &Path,
    input_btl_file: &Path,
    input_gtl_file: &Path,
    output_path: &Path,
    save_map_sprites: &bool,
) -> IoResult<()> {
    let file = File::open(input_map_file)?;
    let mut reader = BufReader::new(file);
    let map_data = read_map_data(&mut reader)?;
    let map_id = input_map_file.file_stem().unwrap().to_str().unwrap();

    if *save_map_sprites {
        for (i, sprite) in map_data.internal_sprites.iter().enumerate() {
            crate::sprite::save_sequence(
                &mut reader,
                &sprite.frame_infos,
                i as i32,
                &map_id.to_string(),
            )?;
        }
    }

    let btl_tileset = tileset::extract(input_btl_file)?;
    let gtl_tileset = tileset::extract(input_gtl_file)?;

    render_map(
        &mut reader,
        output_path,
        &map_data,
        true,
        &gtl_tileset,
        &btl_tileset,
        map_id,
    )
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
/// allowing for proper reconstruction of in-game animations.
pub fn extract_sprites(input_map_file: &Path, output_path: &Path) -> IoResult<()> {
    let file = File::open(input_map_file)?;
    let mut reader = BufReader::new(file);
    let map_data = read_map_data(&mut reader)?;
    let map_id = input_map_file.file_stem().unwrap().to_str().unwrap();

    std::fs::create_dir_all(output_path)?;
    let output_dir_str = output_path.to_str().unwrap();

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
    let mut conn = Connection::open(database_path)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    let file = File::open(map_path)?;
    let mut reader = BufReader::new(file);
    let map_data = read_map_data(&mut reader)?;
    let map_id = map_path.file_stem().unwrap().to_str().unwrap();

    save_to_db(&mut conn, map_id, &map_data)
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
pub fn save_to_db(conn: &mut rusqlite::Connection, map_id: &str, data: &MapData) -> DbResult<()> {
    println!("Saving map tiles for {}...", map_id);
    save_map_tiles(
        conn,
        map_id,
        &data.gtl_tiles,
        &data.btl_tiles,
        &data.collisions,
        &data.events,
        data.model.tiled_map_width,
        data.model.tiled_map_height,
    )?;

    save_map_objects(conn, map_id, &data.tiled_infos)?;

    save_map_sprites(conn, map_id, &data.sprite_blocks)?;

    save_map_metadata(conn, map_id, &data.model)?;

    Ok(())
}

pub fn save_map_tiles(
    conn: &mut Connection,
    map_id: &str,
    gtl_tiles: &HashMap<Coords, i32>,
    btl_tiles: &HashMap<Coords, i32>,
    collisions: &HashMap<Coords, bool>,
    events: &HashMap<Coords, EventBlock>,
    width: i32,
    height: i32,
) -> DbResult<()> {
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
                let event_id = events.get(&coords).map(|e| e.event_id).unwrap_or(0);

                if gtl_id == 0 && btl_id == 0 && !collision && event_id == 0 {
                    continue;
                }

                stmt.execute(params![
                    map_id,
                    x - offset_x,
                    y - offset_y,
                    gtl_id,
                    btl_id,
                    collision,
                    event_id as i32,
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
    tiled_infos: &Vec<TiledObjectInfo>,
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
    sprite_blocks: &Vec<SpriteInfoBlock>,
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
            ])?;
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
