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
use std::io::{BufReader, Read, Result, Seek, SeekFrom};

use crate::sprite;
use crate::sprite::SequenceInfo;

use super::types::{Coords, EventBlock, SpriteInfoBlock, TiledObjectInfo, TiledObjectMetadata};

// --------------------------------------------------------------------------
// Skipped blocks (parsed by the format but not persisted in MapData)
// --------------------------------------------------------------------------

/// Tiled-object ref records: `count` (i32) followed by `(count - 1)` records
/// of 2 × i32 each (`value1`, `value2`).
///
/// `value2` is a linear tile index (`y * stride + x`) into the three end
/// grids — tiled-object rendering and access checks resolve tiles
/// through it.
///
/// Verified against cat1/cat3/dun01/map1/catp fixtures: skipping `(count-1)*8`
/// lands exactly on the overlay table's size field, while `count*8` lands on
/// `0x01010101` garbage. The previous `multiplier*size*4` skip only worked by
/// coincidence (`8 + 2*count*4 == 16 + (count-1)*8`).
/// Reads (instead of skipping) the tiled-object ref records: `count` (i32)
/// followed by `(count - 1)` records of 2 × i32 each (`value1`, `value2`).
///
/// `value2` is a linear tile index (`y * stride + x`) into the three end
/// grids — tiled-object rendering and access checks resolve tiles
/// through it.
///
/// Verified against cat1/cat3/dun01/map1/catp fixtures: reading `(count-1)`
/// pairs lands exactly on the overlay table's size field, while `count*8`
/// lands on `0x01010101` garbage. The previous `multiplier*size*4` skip only
/// worked by coincidence (`8 + 2*count*4 == 16 + (count-1)*8`).
///
/// A count below 1 cannot be honored (the format has no negative-record
/// semantics) and yields a clear error instead of a silent negative seek.
pub fn read_tiled_object_refs(reader: &mut BufReader<File>) -> Result<Vec<(i32, i32)>> {
    let count = reader.read_i32::<LittleEndian>()?;
    if count < 1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid tiled-object ref count {count}"),
        ));
    }
    let mut refs = Vec::with_capacity((count - 1).max(0) as usize);
    for _ in 0..(count - 1) {
        let value0 = reader.read_i32::<LittleEndian>()?;
        let value1 = reader.read_i32::<LittleEndian>()?;
        refs.push((value0, value1));
    }
    Ok(refs)
}

/// Overlay-id lookup table: `size` (i32) followed by `size` u16 entries.
///
/// Indexed by the Access-Ref grid's low 15 bits. Each entry packs
/// `{low byte: transparency mode, high byte: draw-enable}` for the BTL
/// overlay tile with that id.
///
/// Semantics: a high byte of 0 hides the overlay entirely; a low byte of 0
/// blits the tile diamond opaquely, any other value blits it skipping black
/// (0,0,0) pixels.
pub fn read_overlay_id_table(reader: &mut BufReader<File>) -> Result<Vec<u16>> {
    let size = reader.read_i32::<LittleEndian>()?;
    let mut table = Vec::with_capacity(size as usize);
    for _ in 0..size {
        table.push(reader.read_u16::<LittleEndian>()?);
    }
    Ok(table)
}

// --------------------------------------------------------------------------
// Sprite block – embedded sprites stored inside the map file
// --------------------------------------------------------------------------

/// Returns `(sequences, image_stamps)` — one stamp per sequence, same order
/// (stamps are always 6 or 9; anything else is a parse error).
pub fn sprite_block(reader: &mut BufReader<File>) -> Result<(Vec<SequenceInfo>, Vec<i32>)> {
    let sprite_count = reader.read_i32::<LittleEndian>()?;
    let mut sprites = vec![];
    let mut stamps = Vec::with_capacity(sprite_count.unsigned_abs() as usize);
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
        stamps.push(image_stamp);

        reader.seek(SeekFrom::Current(264))?;

        let info = sprite::get_sequence_info(reader)?;
        let info_offset = info.sequence_end_position;
        sprites.push(info);
        reader.seek(SeekFrom::Start(info_offset))?;

        let image_offset: i64 = image_offset.into();
        reader.seek(SeekFrom::Current(image_offset))?;
    }
    Ok((sprites, stamps))
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
        // Frame-0 record (24 bytes): bbox {left, top, right, bottom} in map
        // pixels + duplicated anchor. left/top are re-read as x/y below;
        // right is unused (== left + frame width).
        let bbox_left = reader.read_i32::<LittleEndian>()?;
        let bbox_top = reader.read_i32::<LittleEndian>()?;
        let bbox_right = reader.read_i32::<LittleEndian>()?;
        let sprite_bottom_right_y = reader.read_i32::<LittleEndian>()?;
        let sprite_x = reader.read_i32::<LittleEndian>()?;
        let sprite_y = reader.read_i32::<LittleEndian>()?;
        // In known maps left==sprite_x and top==sprite_y; they are stored
        // in separate per-frame arrays so they may diverge in mods.

        let sprite_id: usize = sprite_id.try_into().unwrap_or(0);
        let skip = (sprites[sprite_id].frame_count - 1) * 6 * 4;
        reader.seek(SeekFrom::Current(skip.into()))?;

        info.push(SpriteInfoBlock {
            sprite_id,
            sprite_x,
            sprite_y,
            sprite_bottom_right_y,
            bbox_left,
            bbox_top,
            bbox_right,
        });
    }
    Ok(info)
}

// --------------------------------------------------------------------------
// Tiled objects block – building definitions composed of BTL tile stacks
//
// Each bundle is a building placed on the map. The file parser reads, per
// bundle: a sub-record count, then per sub-record {i32 stamp, 260-byte name/
// metadata block, item count, items}; each item carries seven i32 fields plus
// a u16 array (tile-stack ids) and an optional byte array when the item's
// first i32 == 1. The Rust reader below flattens this byte stream with an
// empirically fitted layout that round-trips all known maps.
// --------------------------------------------------------------------------

/// Returns `(infos, metadata)` — one [`TiledObjectMetadata`] per bundle, same
/// order as the returned [`TiledObjectInfo`]s.
pub fn tiled_objects_block(
    reader: &mut BufReader<File>,
) -> Result<(Vec<TiledObjectInfo>, Vec<TiledObjectMetadata>)> {
    let bundles_count = reader.read_i32::<LittleEndian>()?;
    // Sub-record count inside the bundle: sub-records of {stamp, 260-byte
    // metadata, items} follow at this point.
    let _sub_record_count = reader.read_i32::<LittleEndian>()?;

    let mut infos: Vec<TiledObjectInfo> = Vec::with_capacity(bundles_count.unsigned_abs() as usize);
    let mut metadata: Vec<TiledObjectMetadata> =
        Vec::with_capacity(bundles_count.unsigned_abs() as usize);
    for _ in 0..bundles_count {
        // 264 bytes: stamp i32 + metadata block (building name/definition).
        // Read verbatim (stamp included) so the DB export is lossless.
        let mut metadata_blob = vec![0u8; 264];
        reader.read_exact(&mut metadata_blob)?;

        // Control words — always observed as (8, 0, 1, 0) in known maps.
        let control_0 = reader.read_i32::<LittleEndian>()?;
        let control_1 = reader.read_i32::<LittleEndian>()?;
        let control_2 = reader.read_i32::<LittleEndian>()?;
        let control_3 = reader.read_i32::<LittleEndian>()?;

        // Unmapped parameters preceding the stack anchor.
        let param_0 = reader.read_i32::<LittleEndian>()?;
        let param_1 = reader.read_i32::<LittleEndian>()?;
        let param_2 = reader.read_i32::<LittleEndian>()?;
        let param_3 = reader.read_i32::<LittleEndian>()?;

        // Anchor of the building stack in map-local pixel coordinates.
        let x = reader.read_i32::<LittleEndian>()?;
        let y = reader.read_i32::<LittleEndian>()?;

        // Unmapped parameters following the anchor.
        let param_4 = reader.read_i32::<LittleEndian>()?;
        let param_5 = reader.read_i32::<LittleEndian>()?;

        // Trailing counts. Only `tile_stack_len` BTL tile ids follow here;
        // the other two counts gate data covered by the skip below.
        let extra_count_a = reader.read_i32::<LittleEndian>()?;
        let extra_count_b = reader.read_i32::<LittleEndian>()?;
        let tile_stack_len = reader.read_i32::<LittleEndian>()?;

        // BTL tile stack for this building, top → bottom.
        let mut ids: Vec<i16> = vec![];
        for _ in 0..tile_stack_len {
            ids.push(reader.read_i16::<LittleEndian>()?);
        }

        infos.push(TiledObjectInfo { ids, x, y });
        metadata.push(TiledObjectMetadata {
            metadata_blob,
            control_0,
            control_1,
            control_2,
            control_3,
            param_0,
            param_1,
            param_2,
            param_3,
            param_4,
            param_5,
            extra_count_a,
            extra_count_b,
        });

        reader.seek(SeekFrom::Current(84))?;
        let skip: i64 = ((extra_count_a + extra_count_b + tile_stack_len) * 4).into();
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

    Ok((infos, metadata))
}

// --------------------------------------------------------------------------
// Event block – per-tile packed u32 (located near end of file)
//
// Low 14 bits hold an event/transition id; observed behavior: hovering such
// a tile shows a name resolved through the Map.ini/AllMap.ini tables (ids < 70 are
// treated as valid). Bit 22 of the word ("tile marked / entity occupies")
// monster chase logic treats marked tiles as blocked; see
// [`EventBlock::is_tile_marked`].
// --------------------------------------------------------------------------

pub fn read_events_block(
    reader: &mut BufReader<File>,
    tiled_map_width: i32,
    tiled_map_height: i32,
) -> Result<HashMap<Coords, EventBlock>> {
    let mut blocks = HashMap::new();

    for y in 0..tiled_map_height {
        for x in 0..tiled_map_width {
            // One packed u32 per tile; see [`EventBlock`] for the bit layout.
            let word = reader.read_u32::<LittleEndian>()?;
            blocks.insert((x, y), EventBlock { x, y, word });
        }
    }
    Ok(blocks)
}

// --------------------------------------------------------------------------
// Tile & access block – one packed u32 per tile
//
// Bit layout (verified against known game data):
//   bits 0        – blocked/collision flag
//   bits 1–9      – map-object slot id (0–511); non-zero marks an
//                   interactive object dispatched to a subsystem handler
//   bits 10–24    – GTL ground-tile index: (word >> 10) & 0x7FFF.
//                   The renderer blits from gtl_base + index * 0x800
//                   (2048 B = one 32×32 RGB565 tile in the .gtl file).
//   bits 25–31    – unused (always 0 in known maps)
//
// Access-modifying script commands rewrite exactly bits 0–9,
// preserving the tile index — this whole field is called "access".
// Known maps only use bit 0; bits 1–9 are set at runtime.
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
// Access-ref grid ("roof") – one packed u32 per tile
//
//   bits 0–14   BTL overlay ref → indexes the u16 table read by
//               `skip_overlay_id_table` ({lo byte: transparency mode,
//               hi byte: draw-enable}); pixels at btl_base + id * 0x800
//   bit 15      bleeds into the signed-i16 view of the ref (negative ⇒ skip)
//   bits 15–29  light level (0–199) selecting a brightness pattern from
//               ExtraInGame/fogdata.dat; applied by the shadow renderer on
//               maps flagged Dark in AllMap.ini (level 0 → blacked out)
//   bits 30–31  light-source flags (entities' light raises the level, max wins)
// --------------------------------------------------------------------------

/// Returns `(btl_overlay_refs, raw_words)` — the parsed overlay refs plus the
/// untouched packed words so the writer can preserve shadow/light bits.
pub fn read_roof_tiles(
    reader: &mut BufReader<File>,
    tiled_map_width: i32,
    tiled_map_height: i32,
) -> Result<(HashMap<Coords, i32>, HashMap<Coords, u32>)> {
    let mut btl_tiles = HashMap::new();
    let mut access_ref_words = HashMap::new();

    for y in 0..tiled_map_height {
        for x in 0..tiled_map_width {
            let word = reader.read_u32::<LittleEndian>()?;
            let coords: Coords = (x, y);
            access_ref_words.insert(coords, word);

            // Overlay ref in bits 0–14. Bit 15 belongs to the shadow level,
            // NOT the ref — mask it off instead of sign-dropping.
            let btl_tile_id = (word & 0x7FFF) as i32;
            if btl_tile_id > 0 {
                btl_tiles.insert(coords, btl_tile_id);
            }
        }
    }
    Ok((btl_tiles, access_ref_words))
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
        let decoded_object = (value >> 1) & 0x1FF;

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
        // Known-file case: object_id=0, high bits 0
        let gtl_tile_id: i32 = 42;
        let object_id: i32 = 0;
        let blocked = false;
        let value = ((gtl_tile_id & 0x7FFF) << 10) | ((object_id & 0x1FF) << 1) | (blocked as i32);

        let decoded_gtl = (value >> 10) & 0x7FFF;
        let decoded_blocked = (value & 0x1) == 1;
        let decoded_object = (value >> 1) & 0x1FF;

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
        let decoded_object = (value >> 1) & 0x1FF;

        assert_eq!(decoded_gtl, 0x7FFF);
        assert_eq!(decoded_object, 0x1FF);
        assert!(decoded_blocked);

        let repacked = ((decoded_gtl & 0x7FFF) << 10)
            | ((decoded_object & 0x1FF) << 1)
            | (decoded_blocked as i32);
        assert_eq!(repacked, value);
    }
}
