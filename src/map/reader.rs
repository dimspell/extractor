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

use super::types::{
    BundleEntry, BundleItem, BundleRecord, Coords, EventBlock, SpriteInfoBlock, StackLevelFlags,
    TiledBundle, TiledObjectInfo, TiledObjectRef,
};

// --------------------------------------------------------------------------
// Skipped blocks (parsed by the format but not persisted in MapData)
// --------------------------------------------------------------------------

/// Tiled-object ref records: `count` (i32) followed by `(count - 1)` records
/// of 2 × i32 each.
///
/// Both words are constant across ALL shipped maps (`word0 == 0`,
/// `word1 == 1` in all 99,104 records probed); their semantics are unknown.
/// The earlier "linear tile index into the end grids" interpretation was
/// wrong. See [`TiledObjectRef`].
///
/// Verified against cat1/cat3/dun01/map1/catp fixtures: reading `(count-1)`
/// pairs lands exactly on the overlay table's size field, while `count*8`
/// lands on `0x01010101` garbage.
///
/// A count below 1 cannot be honored (the format has no negative-record
/// semantics) and yields a clear error instead of a silent negative seek.
pub fn read_tiled_object_refs(reader: &mut BufReader<File>) -> Result<Vec<TiledObjectRef>> {
    let count = reader.read_i32::<LittleEndian>()?;
    if count < 1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid tiled-object ref count {count}"),
        ));
    }
    let mut refs = Vec::with_capacity((count - 1).max(0) as usize);
    for _ in 0..(count - 1) {
        let word0 = reader.read_i32::<LittleEndian>()?;
        let word1 = reader.read_i32::<LittleEndian>()?;
        refs.push(TiledObjectRef { word0, word1 });
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
// Each bundle contains records, items, and entries. An entry contains four
// bounding-box words, seven position and size words, tile IDs, and optional
// extra bytes. The number of level-flag pairs equals the grid height of the
// first entry.
//
// The three end grids follow immediately after the last bundle's flags —
// the format has no end-of-block sentinel. The previous empirical parser's
// "sentinel alignment scan" is gone; any decode error now surfaces as an
// explicit `Err` instead of being papered over by a resync hack.
// --------------------------------------------------------------------------

fn read_bundle_entry(reader: &mut BufReader<File>, type_flag: i32) -> Result<BundleEntry> {
    let bound_x = reader.read_i32::<LittleEndian>()?;
    let bound_y = reader.read_i32::<LittleEndian>()?;
    let bound_right = reader.read_i32::<LittleEndian>()?;
    let bound_bottom = reader.read_i32::<LittleEndian>()?;

    let anchor_x = reader.read_i32::<LittleEndian>()?;
    let anchor_y = reader.read_i32::<LittleEndian>()?;
    let draw_x = reader.read_i32::<LittleEndian>()?;
    let draw_y = reader.read_i32::<LittleEndian>()?;
    let grid_width = reader.read_i32::<LittleEndian>()?;
    let grid_height = reader.read_i32::<LittleEndian>()?;
    let stored_cell_count = reader.read_i32::<LittleEndian>()?;

    // stored_cell_count sizes both trailing arrays of the entry. Reject
    // absurd counts early so that a malformed file returns an error.
    if !(0..=100_000).contains(&stored_cell_count) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid tiled-bundle entry id count {stored_cell_count}"),
        ));
    }

    let mut ids = Vec::with_capacity(stored_cell_count as usize);
    for _ in 0..stored_cell_count {
        ids.push(reader.read_u16::<LittleEndian>()?);
    }

    let extra_payload = if type_flag == 1 {
        let mut bytes = vec![0u8; stored_cell_count as usize];
        reader.read_exact(&mut bytes)?;
        Some(bytes)
    } else {
        None
    };

    Ok(BundleEntry {
        bound_x,
        bound_y,
        bound_right,
        bound_bottom,
        anchor_x,
        anchor_y,
        draw_x,
        draw_y,
        grid_width,
        grid_height,
        stored_cell_count,
        ids,
        extra_payload,
    })
}

/// Returns `(infos, bundles)` — one [`TiledObjectInfo`] per bundle, same
/// order as the returned [`TiledBundle`]s.
pub fn tiled_objects_block(
    reader: &mut BufReader<File>,
) -> Result<(Vec<TiledObjectInfo>, Vec<TiledBundle>)> {
    let bundles_count = reader.read_i32::<LittleEndian>()?;
    if bundles_count < 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid tiled-bundle count {bundles_count}"),
        ));
    }

    let mut infos = Vec::with_capacity(bundles_count as usize);
    let mut bundles = Vec::with_capacity(bundles_count as usize);

    for _bundle_index in 0..bundles_count {
        let record_count = reader.read_i32::<LittleEndian>()?;
        if !(0..=10_000).contains(&record_count) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid tiled-bundle record count {record_count}"),
            ));
        }

        let mut records = Vec::with_capacity(record_count as usize);
        for _record_index in 0..record_count {
            let field_04 = reader.read_i32::<LittleEndian>()?;
            let mut body = [0u8; 260];
            reader.read_exact(&mut body)?;

            let item_count = reader.read_i32::<LittleEndian>()?;
            if !(0..=10_000).contains(&item_count) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid tiled-bundle item count {item_count}"),
                ));
            }

            let mut items = Vec::with_capacity(item_count as usize);
            for _item_index in 0..item_count {
                let type_flag = reader.read_i32::<LittleEndian>()?;
                let entry_count = reader.read_i32::<LittleEndian>()?;
                let field_14 = reader.read_i32::<LittleEndian>()?;
                if !(0..=10_000).contains(&entry_count) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Invalid tiled-bundle entry count {entry_count}"),
                    ));
                }

                let mut entries = Vec::with_capacity(entry_count as usize);
                for _entry_index in 0..entry_count {
                    entries.push(read_bundle_entry(reader, type_flag)?);
                }
                items.push(BundleItem {
                    type_flag,
                    field_14,
                    entries,
                });
            }
            records.push(BundleRecord {
                field_04,
                body,
                items,
            });
        }

        // The first entry sets the number of flag pairs.
        let n = records
            .first()
            .and_then(|r| r.items.first())
            .and_then(|i| i.entries.first())
            .map(|e| e.grid_height)
            .unwrap_or(0);
        if n < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid tiled-bundle flag count {n}"),
            ));
        }
        let mut level_flags = Vec::with_capacity(n as usize);
        for _ in 0..n {
            let flag_a = reader.read_i32::<LittleEndian>()?;
            let flag_b = reader.read_i32::<LittleEndian>()?;
            level_flags.push(StackLevelFlags { flag_a, flag_b });
        }
        let bundle = TiledBundle {
            records,
            level_flags,
        };

        if let Some(info) = bundle.info() {
            infos.push(info);
        }
        bundles.push(bundle);
    }

    Ok((infos, bundles))
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
