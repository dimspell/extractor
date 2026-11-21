//! SQLite persistence for parsed `.map` data.
//!
//! This module owns the *write side* of map data: a [`crate::map::MapData`]
//! value is converted into rows in the map tables registered in the schema
//! manifest. This module owns the write-side API; [`crate::map`] owns parsing
//! and extraction and re-exports it.
//!
//! Every writer uses the INSERT statements from [`crate::database::insert_sql`]
//! and commits each logical group in one transaction, so a failed import never
//! leaves a half-written map behind.

use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};
use image::RgbaImage;
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder, Rgba};
use rusqlite::{Connection, Result as DbResult, params};

use super::insert_sql;
use crate::map::MapData;
use crate::map::model::MapModel;
use crate::map::types::{
    Coords, EventBlock, SpriteInfoBlock, TiledBundle, TiledObjectInfo, TiledObjectRef,
    btl_overlay_draws_in_dark_map,
};
use crate::sprite::{SequenceInfo, rgb16_565_produce_color};

/// Imports a `.map` file into the SQLite database.
///
/// Parsing stays in [`crate::map::read_map_data`]; this function only wires the
/// parser to [`save_to_db`].
pub fn import_to_database(database_path: &Path, map_path: &Path) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::BufReader;

    let mut conn =
        Connection::open(database_path).map_err(|e| std::io::Error::other(e.to_string()))?;

    let file = File::open(map_path)?;
    let mut reader = BufReader::new(file);
    let map_data = crate::map::read_map_data(&mut reader)?;
    let map_id = map_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("map");

    save_to_db(&mut conn, map_id, &map_data, &mut reader)
        .map_err(|e| std::io::Error::other(e.to_string()))
}

/// Writes parsed map data to the SQLite database.
///
/// `reader` must still be positioned at the map file: embedded sprite frames
/// are decoded from their on-disk RGB565 pixel data and stored as PNG blobs.
/// It is generic over `Read + Seek` so callers can supply a buffered file and
/// tests can supply an in-memory cursor.
pub fn save_to_db<R: Read + Seek>(
    conn: &mut Connection,
    map_id: &str,
    data: &MapData,
    reader: &mut R,
) -> DbResult<()> {
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
    save_map_bundles(conn, map_id, &data.tiled_bundles)?;

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

    {
        let mut stmt = tx.prepare(insert_sql("map_tiles"))?;

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
                let under_roof_marker = events
                    .get(&coords)
                    .map(|e| e.is_under_roof_marker())
                    .unwrap_or_default();
                let no_spell_cast_zone = events
                    .get(&coords)
                    .map(|e| e.is_no_spell_cast_zone())
                    .unwrap_or_default();
                let object_id = object_ids.get(&coords).copied();
                let raw_ref = access_ref_words.get(&coords).copied().unwrap_or(0);
                let btl_overlay_draws_in_dark_map = btl_overlay_draws_in_dark_map(raw_ref);
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
                    under_roof_marker,
                    no_spell_cast_zone,
                    btl_overlay_draws_in_dark_map,
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
        let mut stmt = tx.prepare(insert_sql("map_objects"))?;
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
        let mut stmt = tx.prepare(insert_sql("map_sprites"))?;
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

pub fn save_map_sprite_frames<R: Read + Seek>(
    conn: &mut Connection,
    map_id: &str,
    internal_sprites: &[SequenceInfo],
    reader: &mut R,
) -> DbResult<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(insert_sql("map_sprite_frames"))?;
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
        insert_sql("map_metadata"),
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
        let mut stmt = tx.prepare(insert_sql("map_overlay_modes"))?;
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
        let mut stmt = tx.prepare(insert_sql("map_sprite_sequences"))?;
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
    refs: &[TiledObjectRef],
) -> DbResult<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(insert_sql("map_object_refs"))?;
        for (ref_index, r) in refs.iter().enumerate() {
            stmt.execute(params![map_id, ref_index as i32, r.word0, r.word1])?;
        }
    }
    tx.commit()?;
    Ok(())
}

/// Persists the typed tiled-bundle trees into the six `map_bundle*` tables.
///
/// All rows go in one transaction; a failure anywhere rolls the whole map's
/// bundle export back. Row counts are derivable from the tree: one
/// `map_bundles` row per bundle, one record/item/entry row per node, and one
/// id row per `(entry, seq)`.
pub fn save_map_bundles(
    conn: &mut Connection,
    map_id: &str,
    bundles: &[TiledBundle],
) -> DbResult<()> {
    let tx = conn.transaction()?;
    {
        let mut bundle_stmt = tx.prepare(insert_sql("map_bundles"))?;
        let mut record_stmt = tx.prepare(insert_sql("map_bundle_records"))?;
        let mut item_stmt = tx.prepare(insert_sql("map_bundle_items"))?;
        let mut entry_stmt = tx.prepare(insert_sql("map_bundle_entries"))?;
        let mut entry_id_stmt = tx.prepare(insert_sql("map_bundle_entry_ids"))?;
        let mut flag_stmt = tx.prepare(insert_sql("map_bundle_flags"))?;

        for (bundle_index, bundle) in bundles.iter().enumerate() {
            bundle_stmt.execute(params![map_id, bundle_index as i32])?;

            for (level_index, flags) in bundle.level_flags.iter().enumerate() {
                flag_stmt.execute(params![
                    map_id,
                    bundle_index as i32,
                    level_index as i32,
                    flags.flag_a,
                    flags.flag_b
                ])?;
            }

            for (record_index, record) in bundle.records.iter().enumerate() {
                record_stmt.execute(params![
                    map_id,
                    bundle_index as i32,
                    record_index as i32,
                    record.field_04,
                    record.body,
                    record.items.len() as i32,
                ])?;

                for (item_index, item) in record.items.iter().enumerate() {
                    item_stmt.execute(params![
                        map_id,
                        bundle_index as i32,
                        record_index as i32,
                        item_index as i32,
                        item.type_flag,
                        item.field_14,
                        item.entries.len() as i32,
                    ])?;

                    for (entry_index, entry) in item.entries.iter().enumerate() {
                        entry_stmt.execute(params![
                            map_id,
                            bundle_index as i32,
                            record_index as i32,
                            item_index as i32,
                            entry_index as i32,
                            entry.bound_x,
                            entry.bound_y,
                            entry.bound_right,
                            entry.bound_bottom,
                            entry.anchor_x,
                            entry.anchor_y,
                            entry.draw_x,
                            entry.draw_y,
                            entry.grid_width,
                            entry.grid_height,
                            entry.stored_cell_count,
                            entry.extra_payload,
                        ])?;

                        for (seq, &id) in entry.ids.iter().enumerate() {
                            entry_id_stmt.execute(params![
                                map_id,
                                bundle_index as i32,
                                record_index as i32,
                                item_index as i32,
                                entry_index as i32,
                                seq as i32,
                                id as i32,
                            ])?;
                        }
                    }
                }
            }
        }
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sprite::SequenceInfo;

    /// A complete, tiny map built in memory — no game fixtures required.
    fn synthetic_map_data() -> MapData {
        let mut events = HashMap::new();
        // Event word 0x0040_0013: id 19, tile marked, under-roof / no-spell gate.
        events.insert(
            (1, 1),
            EventBlock {
                x: 1,
                y: 1,
                word: 0x0040_0013,
            },
        );

        let mut access_ref_words = HashMap::new();
        // Overlay ref 5 + shadow level 100 + a light-source flag.
        access_ref_words.insert((1, 1), (100 << 15) | 0xC000_0000 | 5);

        let mut gtl_tiles = HashMap::new();
        gtl_tiles.insert((0, 0), 7);
        let mut btl_tiles = HashMap::new();
        btl_tiles.insert((1, 1), 9);
        let mut collisions = HashMap::new();
        collisions.insert((0, 1), true);
        let mut object_ids = HashMap::new();
        object_ids.insert((1, 0), 4);

        MapData {
            model: MapModel {
                tiled_map_width: 2,
                tiled_map_height: 2,
                map_width_in_pixels: 128,
                map_height_in_pixels: 64,
                map_non_occluded_start_x: 16,
                map_non_occluded_start_y: 8,
                occluded_map_in_pixels_width: 96,
                occluded_map_in_pixels_height: 48,
                ..MapModel::default()
            },
            gtl_tiles,
            btl_tiles,
            overlay_modes: vec![0x0101, 0x0002],
            access_ref_words,
            collisions,
            events,
            object_ids,
            tiled_infos: vec![TiledObjectInfo {
                ids: vec![3, 11],
                x: 10,
                y: 20,
            }],
            internal_sprites: vec![SequenceInfo {
                sequence_start_position: 32,
                sequence_end_position: 64,
                frame_count: 0,
                frame_infos: Vec::new(),
            }],
            sprite_blocks: vec![SpriteInfoBlock {
                sprite_id: 0,
                sprite_x: 5,
                sprite_y: 6,
                sprite_bottom_right_y: 38,
                bbox_left: 5,
                bbox_top: 6,
                bbox_right: 21,
            }],
            internal_sprite_stamps: vec![6],
            tiled_object_refs: vec![TiledObjectRef { word0: 0, word1: 1 }],
            tiled_bundles: vec![TiledBundle {
                records: Vec::new(),
                level_flags: vec![],
            }],
        }
    }

    #[test]
    fn test_map_persist_query_round_trip() {
        let mut conn = Connection::open_in_memory().expect("in-memory database");
        super::super::bootstrap(&conn).expect("bootstrap schema");

        let data = synthetic_map_data();
        // No frame pixel data is needed for this synthetic map, so an empty
        // in-memory cursor stands in for the .map file reader.
        let mut reader = Cursor::new(Vec::new());
        save_to_db(&mut conn, "synth", &data, &mut reader).expect("save synthetic map");

        // Metadata (tile count is only used to place the coordinates).
        let (width, height): (i32, i32) = conn
            .query_row(
                "SELECT tiled_width, tiled_height FROM map_metadata WHERE map_id = ?1",
                ["synth"],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("query metadata");
        assert_eq!((width, height), (2, 2));

        // Dense tile grid: 2×2 rows written, shifted into the file's stored
        // coordinate space by width/2 / height/2.
        let tile_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM map_tiles WHERE map_id = ?1",
                ["synth"],
                |row| row.get(0),
            )
            .expect("count tiles");
        assert_eq!(tile_count, 4);

        let tile = |x: i32, y: i32| -> (i32, i32, bool, i64, i64, bool, i64) {
            conn.query_row(
                "SELECT gtl_tile_id, btl_tile_id, collision, event_word, \
                 access_ref_word, under_roof_marker, shadow_level \
                 FROM map_tiles WHERE map_id = ?1 AND x = ?2 AND y = ?3",
                params!["synth", x, y],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                    ))
                },
            )
            .expect("query tile")
        };
        assert_eq!(tile(-1, -1), (7, 0, false, 0, 0, false, 0));
        assert_eq!(
            tile(0, 0),
            (
                0,
                9,
                false,
                0x0040_0013,
                (100 << 15) | 0xC000_0000 | 5,
                true,
                100
            )
        );
        assert!(tile(-1, 0).2, "collision flag at file coords (0,1)");

        let (obj_id, stack, btl): (i32, i32, i32) = conn
            .query_row(
                "SELECT object_index, stack_order, btl_tile_id FROM map_objects \
                 WHERE map_id = ?1 ORDER BY stack_order",
                ["synth"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("query first object tile");
        assert_eq!((obj_id, stack, btl), (0, 0, 3));

        let sprite: (i32, i32, i32, i32) = conn
            .query_row(
                "SELECT x, y, internal_sprite_id, bottom_right_y FROM map_sprites WHERE map_id = ?1",
                ["synth"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("query sprite");
        assert_eq!(sprite, (5, 6, 0, 38));

        let (sequence_count, bundle_count, ref_count): (i64, i64, i64) = conn
            .query_row(
                "SELECT (SELECT COUNT(*) FROM map_sprite_sequences WHERE map_id = ?1), \
                        (SELECT COUNT(*) FROM map_bundles WHERE map_id = ?1), \
                        (SELECT COUNT(*) FROM map_object_refs WHERE map_id = ?1)",
                ["synth"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("query grouped tables");
        assert_eq!(sequence_count, 1);
        assert_eq!(bundle_count, 1);
        assert_eq!(ref_count, 1);
    }
}
