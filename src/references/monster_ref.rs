use std::io::{BufReader, BufWriter};
use std::{fs::File, path::Path};

use crate::references::enums::{BooleanFlag, ByteFlag, ItemTypeId, TriStateFlag};
use crate::references::extractor::{read_mapper, Extractor};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

// ===========================================================================
// MONSTERREF.REF FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | MonsterRef.ref - Monster Placements  |
// +--------------------------------------+
// | Encoding: Binary (Little-Endian)     |
// | Header: 4-byte record count          |
// | Record Size: 56 bytes                |
// +--------------------------------------+
// | [Header]                             |
// | - record_count: i32                  |
// +--------------------------------------+
// | [Record]                             |
// | - file_id: i32                       |
// | - mon_id: i32 (monster type ID)      |
// | - pos_x: i32 (tile X coordinate)     |
// | - pos_y: i32 (tile Y coordinate)     |
// | - padding1: i32 (flag: 0 or 1)       |
// | - padding2: i32 (flag: 0 or 1)       |
// | - padding3: i32 (flag: always 0)     |
// | - padding4: i32 (flag: -1/0/1)       |
// | - event_id: i32 (Event.ini link)     |
// | - loot1_item_id: u8                  |
// | - loot1_item_type: u8                |
// | - padding6: u8 (0 or 255)            |
// | - padding7: u8 (0 or 255)            |
// | - loot2_item_id: u8                  |
// | - loot2_item_type: u8                |
// | - padding8: u8 (0 or 255)            |
// | - padding9: u8 (0 or 255)            |
// | - loot3_item_id: u8                  |
// | - loot3_item_type: u8                |
// | - padding10: u8 (0 or 255)           |
// | - padding11: u8 (0 or 255)           |
// | - padding12: i32 (flag: -1/0/1)      |
// | - padding13: i32 (flag: 0 or 1)      |
// +--------------------------------------+
//
// MONSTER TYPE IDS:
// - Links to Monster.db entries
//
// EVENT IDS:
// - Links to Event.ini entries
//
// FILE PURPOSE:
// Defines monster placements on specific maps with
// exact coordinates, event triggers, and loot drop
// configurations. Used for populating dungeons and
// areas with enemies and their associated rewards.
//
// ===========================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MonsterRef {
    /// Record index relative to the file (0-based).
    pub index: i32,
    /// File identifier / record number.
    pub file_id: i32,
    /// ID of the monster type from Monster.db.
    pub mon_id: i32,
    /// Position on the map (tile X coordinate).
    pub pos_x: i32,
    /// Position on the map (tile Y coordinate).
    pub pos_y: i32,
    /// Unknown flag (observed values: 0 or 1).
    pub padding1: BooleanFlag,
    /// Unknown flag (observed values: 0 or 1).
    pub padding2: BooleanFlag,
    /// Unknown flag (observed values: always 0).
    pub padding3: i32,
    /// Unknown flag (observed values: -1, 0, or 1).
    pub padding4: TriStateFlag,
    /// Event trigger ID, links to Event.ini.
    pub event_id: i32,
    /// First loot drop item ID.
    pub loot1_item_id: u8,
    /// First loot drop item type.
    pub loot1_item_type: ItemTypeId,
    /// Unknown byte (observed values: 0 or 255).
    pub padding6: ByteFlag,
    /// Unknown byte (observed values: 0 or 255).
    pub padding7: ByteFlag,
    /// Second loot drop item ID.
    pub loot2_item_id: u8,
    /// Second loot drop item type.
    pub loot2_item_type: ItemTypeId,
    /// Unknown byte (observed values: 0 or 255).
    pub padding8: ByteFlag,
    /// Unknown byte (observed values: 0 or 255).
    pub padding9: ByteFlag,
    /// Third loot drop item ID.
    pub loot3_item_id: u8,
    /// Third loot drop item type.
    pub loot3_item_type: ItemTypeId,
    /// Unknown byte (observed values: 0 or 255).
    pub padding10: ByteFlag,
    /// Unknown byte (observed values: 0 or 255).
    pub padding11: ByteFlag,
    /// Unknown flag (observed values: -1, 0, or 1).
    pub padding12: TriStateFlag,
    /// Unknown flag (observed values: 0 or 1).
    pub padding13: BooleanFlag,
}

/// Stores specific placements and configurations for monsters on a given map.
///
/// Reads file: `MonsterInGame/Mondun01.ref (and other map-specific .ref files)`
/// # File Format: `MonsterInGame/Mondun01.ref` (and other map `.ref` files)
///
/// Binary file, little-endian.  Starts with a 4-byte i32 record count.
/// Each record encodes monster placement and patrol data:
/// - Various i32/u8 positional and flag fields
/// - 4 waypoint pairs (X, Y) as i32
/// - Spawn timing, chasing distance, and AI flags
impl Extractor for MonsterRef {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let file = File::open(source_path)?;

        let metadata = file.metadata()?;
        let file_len = metadata.len();

        let mut reader = BufReader::new(file);

        const COUNTER_SIZE: u8 = 4;
        const PROPERTY_ITEM_SIZE: i32 = 14 * 4;

        let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
        let mut refs: Vec<MonsterRef> = Vec::with_capacity(elements as usize);

        for i in 0..elements {
            let file_id = reader.read_i32::<LittleEndian>()?;
            let mon_id = reader.read_i32::<LittleEndian>()?;
            let pos_x = reader.read_i32::<LittleEndian>()?;
            let pos_y = reader.read_i32::<LittleEndian>()?;

            let padding1 = BooleanFlag::from_i32(reader.read_i32::<LittleEndian>()?)
                .unwrap_or(BooleanFlag::False);
            let padding2 = BooleanFlag::from_i32(reader.read_i32::<LittleEndian>()?)
                .unwrap_or(BooleanFlag::False);
            let padding3 = reader.read_i32::<LittleEndian>()?;
            let padding4 = TriStateFlag::from_i32(reader.read_i32::<LittleEndian>()?)
                .unwrap_or(TriStateFlag::Zero);
            let padding5 = reader.read_i32::<LittleEndian>()?;

            let loot1_item_id = reader.read_u8()?;
            let loot1_item_type_raw = reader.read_u8()?;
            let padding6 = ByteFlag::from_u8(reader.read_u8()?).unwrap_or(ByteFlag::Zero);
            let padding7 = ByteFlag::from_u8(reader.read_u8()?).unwrap_or(ByteFlag::Zero);

            let loot2_item_id = reader.read_u8()?;
            let loot2_item_type_raw = reader.read_u8()?;
            let padding8 = ByteFlag::from_u8(reader.read_u8()?).unwrap_or(ByteFlag::Zero);
            let padding9 = ByteFlag::from_u8(reader.read_u8()?).unwrap_or(ByteFlag::Zero);

            let loot3_item_id = reader.read_u8()?;
            let loot3_item_type_raw = reader.read_u8()?;
            let padding10 = ByteFlag::from_u8(reader.read_u8()?).unwrap_or(ByteFlag::Zero);
            let padding11 = ByteFlag::from_u8(reader.read_u8()?).unwrap_or(ByteFlag::Zero);

            let padding12 = TriStateFlag::from_i32(reader.read_i32::<LittleEndian>()?)
                .unwrap_or(TriStateFlag::Zero);
            let padding13 = BooleanFlag::from_i32(reader.read_i32::<LittleEndian>()?)
                .unwrap_or(BooleanFlag::False);

            let loot1_item_type =
                ItemTypeId::from_u8(loot1_item_type_raw).unwrap_or(ItemTypeId::Weapon);
            let loot2_item_type =
                ItemTypeId::from_u8(loot2_item_type_raw).unwrap_or(ItemTypeId::Weapon);
            let loot3_item_type =
                ItemTypeId::from_u8(loot3_item_type_raw).unwrap_or(ItemTypeId::Weapon);

            refs.push(MonsterRef {
                index: i,
                file_id,
                mon_id,
                pos_x,
                pos_y,
                padding1,
                padding2,
                padding3,
                padding4,
                event_id: padding5,
                loot1_item_id,
                loot1_item_type,
                padding6,
                padding7,
                loot2_item_id,
                loot2_item_type,
                padding8,
                padding9,
                loot3_item_id,
                loot3_item_type,
                padding10,
                padding11,
                padding12,
                padding13,
            })
        }

        Ok(refs)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let file = File::create(dest_path)?;
        let mut writer = BufWriter::new(file);

        let elements = records.len() as i32;
        writer.write_i32::<LittleEndian>(elements)?;

        for record in records {
            writer.write_i32::<LittleEndian>(record.file_id)?;
            writer.write_i32::<LittleEndian>(record.mon_id)?;
            writer.write_i32::<LittleEndian>(record.pos_x)?;
            writer.write_i32::<LittleEndian>(record.pos_y)?;

            writer.write_i32::<LittleEndian>(i32::from(record.padding1))?;
            writer.write_i32::<LittleEndian>(i32::from(record.padding2))?;
            writer.write_i32::<LittleEndian>(record.padding3)?;
            writer.write_i32::<LittleEndian>(i32::from(record.padding4))?;
            writer.write_i32::<LittleEndian>(record.event_id)?;

            writer.write_u8(record.loot1_item_id)?;
            writer.write_u8(u8::from(record.loot1_item_type))?;
            writer.write_u8(u8::from(record.padding6))?;
            writer.write_u8(u8::from(record.padding7))?;

            writer.write_u8(record.loot2_item_id)?;
            writer.write_u8(u8::from(record.loot2_item_type))?;
            writer.write_u8(u8::from(record.padding8))?;
            writer.write_u8(u8::from(record.padding9))?;

            writer.write_u8(record.loot3_item_id)?;
            writer.write_u8(u8::from(record.loot3_item_type))?;
            writer.write_u8(u8::from(record.padding10))?;
            writer.write_u8(u8::from(record.padding11))?;

            writer.write_i32::<LittleEndian>(i32::from(record.padding12))?; // -1, 0, or 1
            writer.write_i32::<LittleEndian>(i32::from(record.padding13))?;
        }
        Ok(())
    }
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
