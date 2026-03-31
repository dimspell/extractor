/// Binary block readers for the `.map` file format.
///
/// The `.map` file is laid out as a sequence of distinct blocks:
/// 1. Map model header (width × height)
/// 2. First block  – unknown, skipped
/// 3. Second block – unknown, skipped
/// 4. Sprite block – internal embedded sprites (sequence headers)
/// 5. Sprite info block – placement records for embedded sprites
/// 6. Tiled objects block – building/object tile stacks
/// 7. Event block – per-tile event trigger IDs (read from end of file)
/// 8. Tile & access block – GTL tile IDs + collision flags
/// 9. Roof block – BTL tile IDs (optional, only if data remains)
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Result, Seek, SeekFrom};

use crate::sprite;
use crate::sprite::SequenceInfo;

use super::types::{Coords, EventBlock, SpriteInfoBlock, TiledObjectInfo};

// --------------------------------------------------------------------------
// Unknown blocks (skipped on read, not persisted)
// --------------------------------------------------------------------------

pub fn first_block(reader: &mut BufReader<File>) -> Result<()> {
    let multiplier = reader.read_i32::<LittleEndian>()?;
    let size = reader.read_i32::<LittleEndian>()?;
    reader.seek(SeekFrom::Start(8))?;
    let skip: i64 = (multiplier * size * 4).into();
    reader.seek(SeekFrom::Current(skip))?;
    Ok(())
}

pub fn second_block(reader: &mut BufReader<File>) -> Result<()> {
    let size = reader.read_i32::<LittleEndian>()?;
    let skip: i64 = (size * 2).into();
    reader.seek(SeekFrom::Current(skip))?;
    Ok(())
}

// --------------------------------------------------------------------------
// Sprite block – embedded sprites stored inside the map file
// --------------------------------------------------------------------------

pub fn sprite_block(reader: &mut BufReader<File>) -> Result<Vec<SequenceInfo>> {
    let sprite_count = reader.read_i32::<LittleEndian>()?;
    let mut sprites = vec![];
    for _ in 0..sprite_count {
        let image_stamp = reader.read_i32::<LittleEndian>()?;
        let image_offset: i32 = if image_stamp == 6 {
            1904
        } else if image_stamp == 9 {
            2996
        } else {
            unimplemented!("Unexpected image-stamp {image_stamp}");
        };

        reader.seek(SeekFrom::Current(264))?;

        let info = sprite::get_sequence_info(reader)?;
        let info_offset = info.sequence_end_position;
        sprites.push(info);
        reader.seek(SeekFrom::Start(info_offset))?;

        let image_offset: i64 = image_offset.into();
        reader.seek(SeekFrom::Current(image_offset))?;
    }
    Ok(sprites)
}

// --------------------------------------------------------------------------
// Sprite info block – pixel placements for each embedded sprite
// --------------------------------------------------------------------------

pub fn sprite_info_block(
    reader: &mut BufReader<File>,
    sprites: &[SequenceInfo],
) -> Result<Vec<SpriteInfoBlock>> {
    let count = reader.read_i32::<LittleEndian>()?;
    let mut info = Vec::with_capacity(count.try_into().unwrap());

    for _ in 0..count {
        let sprite_id = reader.read_i32::<LittleEndian>()?;
        reader.read_i32::<LittleEndian>()?; // unknown
        reader.read_i32::<LittleEndian>()?; // unknown
        let _sprite_bottom_right_x = reader.read_i32::<LittleEndian>()?;
        let _sprite_bottom_right_y = reader.read_i32::<LittleEndian>()?;
        let sprite_x = reader.read_i32::<LittleEndian>()?;
        let sprite_y = reader.read_i32::<LittleEndian>()?;

        let sprite_id: usize = sprite_id.try_into().unwrap();
        let skip = (sprites[sprite_id].frame_count - 1) * 6 * 4;
        reader.seek(SeekFrom::Current(skip.into()))?;

        info.push(SpriteInfoBlock {
            sprite_id,
            sprite_x,
            sprite_y,
        });
    }
    Ok(info)
}

// --------------------------------------------------------------------------
// Tiled objects block – buildings/objects composed of BTL tile stacks
// --------------------------------------------------------------------------

pub fn tiled_objects_block(reader: &mut BufReader<File>) -> Result<Vec<TiledObjectInfo>> {
    let bundles_count = reader.read_i32::<LittleEndian>()?;
    let _number1 = reader.read_i32::<LittleEndian>()?;

    let mut infos: Vec<TiledObjectInfo> = Vec::with_capacity(bundles_count.unsigned_abs() as usize);
    for _ in 0..bundles_count {
        reader.seek(SeekFrom::Current(264))?;

        let _s8 = reader.read_i32::<LittleEndian>()?;
        let _s0_1 = reader.read_i32::<LittleEndian>()?;
        let _s1 = reader.read_i32::<LittleEndian>()?;
        let _s0_2 = reader.read_i32::<LittleEndian>()?;

        let _v1 = reader.read_i32::<LittleEndian>()?;
        let _v2 = reader.read_i32::<LittleEndian>()?;
        let _v3 = reader.read_i32::<LittleEndian>()?;
        let _v4 = reader.read_i32::<LittleEndian>()?;
        let x = reader.read_i32::<LittleEndian>()?;
        let y = reader.read_i32::<LittleEndian>()?;
        let _v7 = reader.read_i32::<LittleEndian>()?;
        let _v8 = reader.read_i32::<LittleEndian>()?;

        let c1 = reader.read_i32::<LittleEndian>()?;
        let c2 = reader.read_i32::<LittleEndian>()?;
        let c3 = reader.read_i32::<LittleEndian>()?;

        let mut ids: Vec<i16> = vec![];
        for _ in 0..c3 {
            ids.push(reader.read_i16::<LittleEndian>()?);
        }

        infos.push(TiledObjectInfo { ids, x, y });

        reader.seek(SeekFrom::Current(84))?;
        let skip: i64 = ((c1 + c2 + c3) * 4).into();
        reader.seek(SeekFrom::Current(skip))?;
    }

    // Align past the bundle-end sentinel
    let back_pos = 20;
    reader.seek(SeekFrom::Current(-back_pos))?;
    let mut last_pos = 0u8;
    for _ in 0..back_pos {
        let v: u8 = reader.read_u8()?;
        if v == 1 {
            last_pos = v;
        }
    }
    let to_undo: i64 = back_pos;
    reader.seek(SeekFrom::Current(to_undo))?;
    let to_undo: i64 = last_pos.into();
    reader.seek(SeekFrom::Current(-to_undo - 4))?;

    Ok(infos)
}

// --------------------------------------------------------------------------
// Event block – per-tile event trigger IDs (located near end of file)
// --------------------------------------------------------------------------

pub fn read_events_block(
    reader: &mut BufReader<File>,
    tiled_map_width: i32,
    tiled_map_height: i32,
) -> Result<HashMap<Coords, EventBlock>> {
    let mut blocks = HashMap::new();

    for y in 0..tiled_map_height {
        for x in 0..tiled_map_width {
            let event_id = reader.read_i16::<LittleEndian>()?;
            let unknown_value = reader.read_i16::<LittleEndian>()?;
            blocks.insert(
                (x, y),
                EventBlock {
                    x,
                    y,
                    event_id,
                    unknown_value,
                },
            );
        }
    }
    Ok(blocks)
}

// --------------------------------------------------------------------------
// Tile & access block – GTL tile IDs and collision flags
// --------------------------------------------------------------------------

pub fn read_tiles_and_access_block(
    reader: &mut BufReader<File>,
    tiled_map_width: i32,
    tiled_map_height: i32,
) -> Result<(HashMap<Coords, i32>, HashMap<Coords, bool>)> {
    let mut gtl_tiles = HashMap::new();
    let mut collisions = HashMap::new();

    for y in 0..tiled_map_height {
        for x in 0..tiled_map_width {
            let coords: Coords = (x, y);
            let value = reader.read_i32::<LittleEndian>()?;
            let gtl_tile_id = value >> 10;
            let collision = (gtl_tile_id & 0x1) == 1;
            gtl_tiles.insert(coords, gtl_tile_id);
            collisions.insert(coords, collision);
        }
    }
    Ok((gtl_tiles, collisions))
}

// --------------------------------------------------------------------------
// Roof tile block – BTL tile IDs for building roofs
// --------------------------------------------------------------------------

pub fn read_roof_tiles(
    reader: &mut BufReader<File>,
    tiled_map_width: i32,
    tiled_map_height: i32,
) -> Result<HashMap<Coords, i32>> {
    let mut btl_tiles = HashMap::new();

    for y in 0..tiled_map_height {
        for x in 0..tiled_map_width {
            let btl_tile_id = reader.read_i16::<LittleEndian>()?;
            let some_flag = reader.read_i16::<LittleEndian>()?;
            let coords: Coords = (x, y);

            if btl_tile_id > 0 {
                if some_flag > 0 {
                    println!("ReadRoofTiles TODO: {x:?}:{y:?} {btl_tile_id:?} {some_flag}");
                } else {
                    btl_tiles.insert(coords, btl_tile_id.into());
                }
            }
        }
    }
    Ok(btl_tiles)
}
