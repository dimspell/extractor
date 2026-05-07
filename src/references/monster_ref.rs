use std::path::Path;

use crate::references::enums::{BooleanFlag, ByteFlag, ItemTypeId, TriStateFlag};
use crate::references::extractor::Extractor;
use dispel_macros::{Extractor, RecordPatcher};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

/// MonsterRef.ref - Monster Placements on Maps
///
/// Stores specific placements and configurations for monsters on a given map.
///
/// Reads file: `MonsterInGame/Mondun01.ref` (and other map-specific `.ref` files)
///
/// # Binary Format
///
/// - **Encoding**: Little-endian for all numeric values
/// - **Record Size**: 56 bytes (11 × i32 + 12 × u8)
/// - **Header**: 4-byte i32 record count, followed by records
///
/// ```text
/// +--------------------------------------+
/// | MonsterRef.ref - Monster Placements  |
/// +--------------------------------------+
/// | Encoding: Binary (Little-Endian)     |
/// | Record Size: 56 bytes               |
/// | Header: 4-byte record count          |
/// +--------------------------------------+
/// | [Header]                             |
/// | - record_count: i32                  |
/// +--------------------------------------+
/// | [Record 1] - 56 bytes               |
/// | - file_id: i32                       |
/// | - mon_id: i32 (-> Monster.db)       |
/// | - pos_x: i32 (tile X)               |
/// | - pos_y: i32 (tile Y)               |
/// | - padding1: i32 (BooleanFlag)       |
/// | - padding2: i32 (BooleanFlag)       |
/// | - padding3: i32                     |
/// | - padding4: i32 (TriStateFlag)      |
/// | - event_id: i32 (-> Event.ini)      |
/// | - loot1_item_id: u8                 |
/// | - loot1_item_type: u8 (ItemTypeId) |
/// | - padding6: u8 (ByteFlag)           |
/// | - padding7: u8 (ByteFlag)           |
/// | - loot2_item_id: u8                 |
/// | - loot2_item_type: u8 (ItemTypeId) |
/// | - padding8: u8 (ByteFlag)           |
/// | - padding9: u8 (ByteFlag)           |
/// | - loot3_item_id: u8                 |
/// | - loot3_item_type: u8 (ItemTypeId) |
/// | - padding10: u8 (ByteFlag)          |
/// | - padding11: u8 (ByteFlag)          |
/// | - padding12: i32 (TriStateFlag)     |
/// | - padding13: i32 (BooleanFlag)      |
/// +--------------------------------------+
/// | [Record 2]                           |
/// | ... (same structure) ...             |
/// +--------------------------------------+
/// ```
///
/// # Field Categories
///
/// - **Identification**: `file_id`, `mon_id` (links to `Monster.db`)
/// - **Position**: `pos_x`, `pos_y` (tile coordinates)
/// - **Event Link**: `event_id` (links to `Event.ini`)
/// - **Loot Drops**: 3 loot slots (`loot1/2/3_item_id` + `item_type`)
/// - **Unknown**: `padding1` through `padding13` (need investigation)
///
/// # Special Values
///
/// - `padding1/padding2`: Usually 0 or 1 (boolean flags)
/// - `padding3`: Always observed as 0
/// - `padding4/padding12`: -1, 0, or 1 (tri-state flags)
/// - `padding6-11`: 0 or 255 (byte flags)
/// - `padding13`: 0 or 1 (boolean flag)
///
/// # File Purpose
///
/// Defines monster placements on specific maps with position,
/// event triggers, and loot configurations. Used by game engine
/// for monster spawning and encounter design.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Extractor, RecordPatcher)]
#[extractor(property_item_size = 56)]
#[patcher(extension = "ref", stem_prefix = "mon")]
pub struct MonsterRef {
    /// Record index relative to the file (0-based).
    #[extractor(index)]
    pub index: i32,
    /// File identifier / record number.
    #[extractor(primitive(type = "i32"))]
    pub file_id: i32,
    /// ID of the monster type from Monster.db.
    #[extractor(primitive(type = "i32"))]
    pub mon_id: i32,
    /// Position on the map (tile X coordinate).
    #[extractor(primitive(type = "i32"))]
    pub pos_x: i32,
    /// Position on the map (tile Y coordinate).
    #[extractor(primitive(type = "i32"))]
    pub pos_y: i32,
    /// Unknown flag (observed values: 0 or 1).
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub padding1: BooleanFlag,
    /// Unknown flag (observed values: 0 or 1).
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub padding2: BooleanFlag,
    /// Unknown flag (observed values: always 0).
    #[extractor(primitive(type = "i32"))]
    pub padding3: i32,
    /// Unknown flag (observed values: -1, 0, or 1).
    #[extractor(enum_from_i32(type = "TriStateFlag"))]
    pub padding4: TriStateFlag,
    /// Event trigger ID, links to Event.ini.
    #[extractor(primitive(type = "i32"))]
    pub event_id: i32,
    /// First loot drop item ID.
    #[extractor(primitive(type = "u8"))]
    pub loot1_item_id: u8,
    /// First loot drop item type.
    #[extractor(enum_from_u8(type = "ItemTypeId"))]
    pub loot1_item_type: ItemTypeId,
    /// Unknown byte (observed values: 0 or 255).
    #[extractor(enum_from_i32_from_u8(type = "ByteFlag"))]
    pub padding6: ByteFlag,
    /// Unknown byte (observed values: 0 or 255).
    #[extractor(enum_from_i32_from_u8(type = "ByteFlag"))]
    pub padding7: ByteFlag,
    /// Second loot drop item ID.
    #[extractor(primitive(type = "u8"))]
    pub loot2_item_id: u8,
    /// Second loot drop item type.
    #[extractor(enum_from_u8(type = "ItemTypeId"))]
    pub loot2_item_type: ItemTypeId,
    /// Unknown byte (observed values: 0 or 255).
    #[extractor(enum_from_i32_from_u8(type = "ByteFlag"))]
    pub padding8: ByteFlag,
    /// Unknown byte (observed values: 0 or 255).
    #[extractor(enum_from_i32_from_u8(type = "ByteFlag"))]
    pub padding9: ByteFlag,
    /// Third loot drop item ID.
    #[extractor(primitive(type = "u8"))]
    pub loot3_item_id: u8,
    /// Third loot drop item type.
    #[extractor(enum_from_u8(type = "ItemTypeId"))]
    pub loot3_item_type: ItemTypeId,
    /// Unknown byte (observed values: 0 or 255).
    #[extractor(enum_from_i32_from_u8(type = "ByteFlag"))]
    pub padding10: ByteFlag,
    /// Unknown byte (observed values: 0 or 255).
    #[extractor(enum_from_i32_from_u8(type = "ByteFlag"))]
    pub padding11: ByteFlag,
    /// Unknown flag (observed values: -1, 0, or 1).
    #[extractor(enum_from_i32(type = "TriStateFlag"))]
    pub padding12: TriStateFlag,
    /// Unknown flag (observed values: 0 or 1).
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub padding13: BooleanFlag,
}

pub fn read_monster_ref(source_path: &Path) -> std::io::Result<Vec<MonsterRef>> {
    MonsterRef::read_file(source_path)
}

pub fn save_monster_refs(
    conn: &mut Connection,
    file_path: &str,
    monster_refs: &[MonsterRef],
) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_monster_ref.sql"))?;
        for monster_ref in monster_refs {
            stmt.execute(params![
                file_path,
                monster_ref.index,
                monster_ref.file_id,
                monster_ref.mon_id,
                monster_ref.pos_x,
                monster_ref.pos_y,
                i32::from(monster_ref.padding1),
                i32::from(monster_ref.padding2),
                monster_ref.padding3,
                i32::from(monster_ref.padding4),
                monster_ref.event_id,
                monster_ref.loot1_item_id,
                u8::from(monster_ref.loot1_item_type),
                u8::from(monster_ref.padding6),
                u8::from(monster_ref.padding7),
                monster_ref.loot2_item_id,
                u8::from(monster_ref.loot2_item_type),
                u8::from(monster_ref.padding8),
                u8::from(monster_ref.padding9),
                monster_ref.loot3_item_id,
                u8::from(monster_ref.loot3_item_type),
                u8::from(monster_ref.padding10),
                u8::from(monster_ref.padding11),
                i32::from(monster_ref.padding12),
                i32::from(monster_ref.padding13),
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn ref_bytes(file_id: i32, mon_id: i32, pos_x: i32, pos_y: i32) -> Vec<u8> {
        // 14 × i32 = 56 bytes; remaining 10 fields are zero
        let mut buf: Vec<u8> = Vec::with_capacity(56);
        for &v in &[file_id, mon_id, pos_x, pos_y, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        buf
    }

    #[test]
    fn parse_one_ref() {
        let rec = ref_bytes(1, 5, 10, 20);
        let mut data = 1i32.to_le_bytes().to_vec(); // header
        data.extend(&rec);
        assert_eq!(data.len(), 60);

        let mut c = Cursor::new(&data[..]);
        let refs = MonsterRef::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].file_id, 1);
        assert_eq!(refs[0].mon_id, 5);
        assert_eq!(refs[0].pos_x, 10);
        assert_eq!(refs[0].pos_y, 20);
    }

    #[test]
    fn parse_two_refs() {
        let mut data = 2i32.to_le_bytes().to_vec();
        data.extend(ref_bytes(1, 2, 3, 4));
        data.extend(ref_bytes(5, 6, 7, 8));

        let mut c = Cursor::new(&data[..]);
        let refs = MonsterRef::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[1].mon_id, 6);
    }

    #[test]
    fn serialize_round_trip() {
        let mut data = 1i32.to_le_bytes().to_vec();
        data.extend(ref_bytes(1, 2, 3, 4));
        let mut c = Cursor::new(&data[..]);
        let records = MonsterRef::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        MonsterRef::to_writer(&records, &mut out).unwrap();
        let mut c2 = Cursor::new(out.as_slice());
        let records2 = MonsterRef::parse(&mut c2, out.len() as u64).unwrap();
        assert_eq!(records.len(), records2.len());
        assert_eq!(records[0].file_id, records2[0].file_id);
        assert_eq!(records[0].mon_id, records2[0].mon_id);
        assert_eq!(records[0].pos_x, records2[0].pos_x);
        assert_eq!(records[0].pos_y, records2[0].pos_y);
    }
}
