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

pub mod database;
pub mod model;
pub mod reader;
pub mod render;
pub mod sprite_loader;
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
use std::io::{BufReader, Result, Seek, SeekFrom};
use std::path::Path;

use crate::sprite::SequenceInfo;
use crate::tileset;

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
pub fn read_map_data(reader: &mut BufReader<File>) -> Result<MapData> {
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
    reader.seek(SeekFrom::End(skip.try_into().unwrap()))?;

    let events = read_events_block(reader, tiled_map_width, tiled_map_height)?;
    let (gtl_tiles, collisions) =
        read_tiles_and_access_block(reader, tiled_map_width, tiled_map_height)?;

    let mut btl_tiles = HashMap::new();
    let current_pos = reader.seek(SeekFrom::Current(0))?;
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
pub fn extract(
    input_map_file: &Path,
    input_btl_file: &Path,
    input_gtl_file: &Path,
    output_path: &Path,
    save_map_sprites: &bool,
) -> Result<()> {
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
pub fn extract_sprites(input_map_file: &Path, output_path: &Path) -> Result<()> {
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
pub fn import_to_database(database_path: &Path, map_path: &Path) -> Result<()> {
    use rusqlite::Connection;
    let mut conn = Connection::open(database_path)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    let file = File::open(map_path)?;
    let mut reader = BufReader::new(file);
    let map_data = read_map_data(&mut reader)?;
    let map_id = map_path.file_stem().unwrap().to_str().unwrap();

    save_to_db(&mut conn, map_id, &map_data)
}

/// Writes map data to the SQLite database.
pub fn save_to_db(conn: &mut rusqlite::Connection, map_id: &str, data: &MapData) -> Result<()> {
    println!("Saving map tiles for {}...", map_id);
    crate::database::save_map_tiles(
        conn,
        map_id,
        &data.gtl_tiles,
        &data.btl_tiles,
        &data.collisions,
        &data.events,
        data.model.tiled_map_width,
        data.model.tiled_map_height,
    )
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    crate::database::save_map_objects(conn, map_id, &data.tiled_infos)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    crate::database::save_map_sprites(conn, map_id, &data.sprite_blocks)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    crate::database::save_map_metadata(conn, map_id, &data.model)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    Ok(())
}
