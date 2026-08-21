/// Binary block readers for the `.map` file format.
///
/// The `.map` file is laid out as a sequence of distinct blocks:
/// 1. Map model header (width × height × border count, 3 × i32)
/// 2. First block  – count + (count-1) × 8 bytes of 2 × i32 records, skipped
/// 3. Second block – size + size × 2 bytes of byte pairs, skipped
/// 4. Sprite block – internal embedded sprites (sequence headers)
/// 5. Sprite info block – placement records for embedded sprites
/// 6. Tiled objects block – building/object tile stacks
/// 7. Event block – per-tile packed u32 (low 14 bits: event/transition id)
/// 8. Tile & access block – per-tile packed u32:
///    bits 0–9 = access field (bit 0 collision, bits 1–9 object slot id),
///    bits 10–24 = GTL ground-tile index
/// 9. Access-ref block ("roof") – per-tile packed u32 indexing the second
///    block's u16 table (low 15 bits = table id)
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

/// First block: `count` (i32) followed by `(count - 1)` records of 2 × i32 each.
///
/// Verified against cat1/cat3/dun01/map1/catp fixtures: skipping `(count-1)*8`
/// lands exactly on the second block's size field (974, 3783, 1419, 6643, 353),
/// while `count*8` lands on `0x01010101` garbage. The previous
/// `multiplier*size*4` skip only worked by coincidence
/// (`8 + 2*count*4 == 16 + (count-1)*8`).
pub fn first_block(reader: &mut BufReader<File>) -> Result<()> {
    let count = reader.read_i32::<LittleEndian>()?;
    let skip: i64 = ((count - 1) * 8).into();
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
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unexpected image-stamp {image_stamp}"),
            ));
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
    let mut info = Vec::with_capacity(count.try_into().unwrap_or(0));

    for _ in 0..count {
        let sprite_id = reader.read_i32::<LittleEndian>()?;
        reader.read_i32::<LittleEndian>()?; // unknown
        reader.read_i32::<LittleEndian>()?; // unknown
        let _sprite_bottom_right_x = reader.read_i32::<LittleEndian>()?;
        let sprite_bottom_right_y = reader.read_i32::<LittleEndian>()?;
        let sprite_x = reader.read_i32::<LittleEndian>()?;
        let sprite_y = reader.read_i32::<LittleEndian>()?;

        let sprite_id: usize = sprite_id.try_into().unwrap_or(0);
        let skip = (sprites[sprite_id].frame_count - 1) * 6 * 4;
        reader.seek(SeekFrom::Current(skip.into()))?;

        info.push(SpriteInfoBlock {
            sprite_id,
            sprite_x,
            sprite_y,
            sprite_bottom_right_y,
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
// Event block – per-tile packed u32 (located near end of file)
//
// Low 14 bits hold an event/transition id; in-game, hovering such a tile
// shows a name resolved through the Map.ini/AllMap.ini tables. The binary
// masks with 0x3fff and treats ids < 70 as valid. High bits carry
// parameters on some tiles (semantics not fully mapped yet).
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
                    _unknown_value: unknown_value,
                },
            );
        }
    }
    Ok(blocks)
}

// --------------------------------------------------------------------------
// Tile & access block – one packed u32 per tile
//
// Bit layout (verified against Dispel_145_wm.exe):
//   bits 0        – blocked/collision flag
//   bits 1–9      – map-object slot id (0–511); non-zero marks an
//                   interactive object dispatched to a subsystem handler
//   bits 10–24    – GTL ground-tile index: (word >> 10) & 0x7FFF.
//                   The renderer blits from gtl_base + index * 0x800
//                   (2048 B = one 32×32 RGB565 tile in the .gtl file).
//   bits 25–31    – unused (always 0 in shipped maps)
//
// The game's `setaccess` script command rewrites exactly bits 0–9,
// preserving the tile index — the game calls this whole field "access".
// Shipped maps only use bit 0; bits 1–9 are set at runtime.
// --------------------------------------------------------------------------

/// Parsed result of `read_tiles_and_access_block`: (gtl_tiles, collisions, object_ids).
type TileAccessResult = (
    HashMap<Coords, i32>,
    HashMap<Coords, bool>,
    HashMap<Coords, i32>,
);

pub fn read_tiles_and_access_block(
    reader: &mut BufReader<File>,
    tiled_map_width: i32,
    tiled_map_height: i32,
) -> Result<TileAccessResult> {
    let mut gtl_tiles = HashMap::new();
    let mut collisions = HashMap::new();
    let mut object_ids = HashMap::new();

    for y in 0..tiled_map_height {
        for x in 0..tiled_map_width {
            let coords: Coords = (x, y);
            let value = reader.read_i32::<LittleEndian>()?;
            let gtl_tile_id = (value >> 10) & 0x7FFF;
            let collision = (value & 0x1) == 1;
            let object_id = (value >> 1) & 0x1FF;
            gtl_tiles.insert(coords, gtl_tile_id);
            collisions.insert(coords, collision);
            if object_id != 0 {
                object_ids.insert(coords, object_id);
            }
        }
    }
    Ok((gtl_tiles, collisions, object_ids))
}

// --------------------------------------------------------------------------
// Access-ref block ("roof") – one packed u32 per tile
//
// Bits 0–14 index the second block's u16 table (the table skipped by
// `second_block`); the entry's low byte is a boolean consumed by the game's
// tile-access/occlusion checks (HIDEACCESS / SHOWACCESS debug commands).
// Bits 15+ are flags (rare — e.g. 7 border tiles in cat1.map). Roof visuals
// themselves come from the tiled-objects block, not from this grid.
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
                    println!("ReadRoofTiles TODO: {btl_tile_id:?} {some_flag}");
                }
                btl_tiles.insert(coords, btl_tile_id.into());
            }
        }
    }
    Ok(btl_tiles)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_tile_access_round_trip_synthetic_word() {
        // Encode: gtl=1234, object_id=5, blocked=true
        let gtl_tile_id: i32 = 1234;
        let object_id: i32 = 5;
        let blocked = true;
        let value = ((gtl_tile_id & 0x7FFF) << 10) | ((object_id & 0x1FF) << 1) | (blocked as i32);

        // Decode
        let decoded_gtl = (value >> 10) & 0x7FFF;
        let decoded_blocked = (value & 0x1) == 1;
        let decoded_object = ((value >> 1) & 0x1FF) as i32;

        assert_eq!(decoded_gtl, 1234, "GTL tile id mismatch");
        assert_eq!(decoded_object, 5, "Object id mismatch");
        assert!(decoded_blocked, "Blocked flag mismatch");

        // Re-encode
        let repacked = ((decoded_gtl & 0x7FFF) << 10)
            | ((decoded_object & 0x1FF) << 1)
            | (decoded_blocked as i32);
        assert_eq!(repacked, value, "Repacked value differs from original");
    }

    #[test]
    fn test_tile_access_round_trip_zero_object() {
        // Shipped file case: object_id=0, high bits 0
        let gtl_tile_id: i32 = 42;
        let object_id: i32 = 0;
        let blocked = false;
        let value = ((gtl_tile_id & 0x7FFF) << 10) | ((object_id & 0x1FF) << 1) | (blocked as i32);

        let decoded_gtl = (value >> 10) & 0x7FFF;
        let decoded_blocked = (value & 0x1) == 1;
        let decoded_object = ((value >> 1) & 0x1FF) as i32;

        assert_eq!(decoded_gtl, 42);
        assert_eq!(decoded_object, 0);
        assert!(!decoded_blocked);

        let repacked = ((decoded_gtl & 0x7FFF) << 10)
            | ((decoded_object & 0x1FF) << 1)
            | (decoded_blocked as i32);
        assert_eq!(repacked, value);
    }

    #[test]
    fn test_tile_access_round_trip_max_object_and_gtl() {
        // Max values: gtl=0x7FFF (32767), object_id=0x1FF (511), blocked=true
        let gtl_tile_id: i32 = 0x7FFF;
        let object_id: i32 = 0x1FF;
        let blocked = true;
        let value = ((gtl_tile_id & 0x7FFF) << 10) | ((object_id & 0x1FF) << 1) | (blocked as i32);

        let decoded_gtl = (value >> 10) & 0x7FFF;
        let decoded_blocked = (value & 0x1) == 1;
        let decoded_object = ((value >> 1) & 0x1FF) as i32;

        assert_eq!(decoded_gtl, 0x7FFF);
        assert_eq!(decoded_object, 0x1FF);
        assert!(decoded_blocked);

        let repacked = ((decoded_gtl & 0x7FFF) << 10)
            | ((decoded_object & 0x1FF) << 1)
            | (decoded_blocked as i32);
        assert_eq!(repacked, value);
    }
}
