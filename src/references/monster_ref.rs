use std::io::{BufReader, BufWriter};
use std::{fs::File, path::Path};

use crate::references::enums::ItemTypeId;
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
// | Record Size: 56 bytes (14 × i32)      |
// +--------------------------------------+
// | [Header]                            |
// | - record_count: i32                  |
// +--------------------------------------+
// | [Record 1]                           |
// | - file_id: i32                       |
// | - mon_id: i32 (monster type ID)      |
// | - pos_x: i32 (tile X coordinate)     |
// | - pos_y: i32 (tile Y coordinate)     |
// | - padding: 5 × i32                   |
// | - loot1_item_id: u8                   |
// | - loot1_item_type: u8                |
// | - padding: 2 × u8                    |
// | - loot2_item_id: u8                   |
// | - loot2_item_type: u8                |
// | - padding: 2 × u8                    |
// | - loot3_item_id: u8                   |
// | - loot3_item_type: u8                |
// | - padding: 2 × u8                    |
// | - padding: 2 × i32                   |
// +--------------------------------------+
// | [Record 2]                           |
// | ... (same structure) ...             |
// +--------------------------------------+
//
// MONSTER TYPE IDS:
// - Links to Monster.db entries
//
// FILE PURPOSE:
// Defines monster placements on specific maps with
// exact coordinates and loot drop configurations.
// Used for populating dungeons and areas with enemies
// and their associated rewards.
//
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterRef {
    /// Record index relative to the Mondun struct array.
    pub index: i32,
    pub file_id: i32,
    pub mon_id: i32,
    pub pos_x: i32,
    pub pos_y: i32,
    pub padding1: i32,
    pub padding2: i32,
    pub padding3: i32,
    pub padding4: i32,
    pub padding5: i32,
    pub loot1_item_id: u8,
    pub loot1_item_type: ItemTypeId,
    pub padding6: u8,
    pub padding7: u8,
    pub loot2_item_id: u8,
    pub loot2_item_type: ItemTypeId,
    pub padding8: u8,
    pub padding9: u8,
    pub loot3_item_id: u8,
    pub loot3_item_type: ItemTypeId,
    pub padding10: u8,
    pub padding11: u8,
    pub padding12: i32,
    pub padding13: i32,
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

            let padding1 = reader.read_i32::<LittleEndian>()?;
            let padding2 = reader.read_i32::<LittleEndian>()?;
            let padding3 = reader.read_i32::<LittleEndian>()?;
            let padding4 = reader.read_i32::<LittleEndian>()?;
            let padding5 = reader.read_i32::<LittleEndian>()?;

            let loot1_item_id = reader.read_u8()?;
            let loot1_item_type_raw = reader.read_u8()?;
            let padding6 = reader.read_u8()?;
            let padding7 = reader.read_u8()?;

            let loot2_item_id = reader.read_u8()?;
            let loot2_item_type_raw = reader.read_u8()?;
            let padding8 = reader.read_u8()?;
            let padding9 = reader.read_u8()?;

            let loot3_item_id = reader.read_u8()?;
            let loot3_item_type_raw = reader.read_u8()?;
            let padding10 = reader.read_u8()?;
            let padding11 = reader.read_u8()?;

            let padding12 = reader.read_i32::<LittleEndian>()?;
            let padding13 = reader.read_i32::<LittleEndian>()?;

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
                padding5,
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

            writer.write_i32::<LittleEndian>(record.padding1)?;
            writer.write_i32::<LittleEndian>(record.padding2)?;
            writer.write_i32::<LittleEndian>(record.padding3)?;
            writer.write_i32::<LittleEndian>(record.padding4)?;
            writer.write_i32::<LittleEndian>(record.padding5)?;

            writer.write_u8(record.loot1_item_id)?;
            writer.write_u8(u8::from(record.loot1_item_type))?;
            writer.write_u8(record.padding6)?;
            writer.write_u8(record.padding7)?;

            writer.write_u8(record.loot2_item_id)?;
            writer.write_u8(u8::from(record.loot2_item_type))?;
            writer.write_u8(record.padding8)?;
            writer.write_u8(record.padding9)?;

            writer.write_u8(record.loot3_item_id)?;
            writer.write_u8(u8::from(record.loot3_item_type))?;
            writer.write_u8(record.padding10)?;
            writer.write_u8(record.padding11)?;

            writer.write_i32::<LittleEndian>(record.padding12)?; // 1 or 0
            writer.write_i32::<LittleEndian>(record.padding13)?;
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
    monster_refs: &Vec<MonsterRef>,
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
                monster_ref.padding1,
                monster_ref.padding2,
                monster_ref.padding3,
                monster_ref.padding4,
                monster_ref.padding5,
                monster_ref.loot1_item_id,
                u8::from(monster_ref.loot1_item_type),
                monster_ref.padding6,
                monster_ref.padding7,
                monster_ref.loot2_item_id,
                u8::from(monster_ref.loot2_item_type),
                monster_ref.padding8,
                monster_ref.padding9,
                monster_ref.loot3_item_id,
                u8::from(monster_ref.loot3_item_type),
                monster_ref.padding10,
                monster_ref.padding11,
                monster_ref.padding12,
                monster_ref.padding13,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
