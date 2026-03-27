use std::io::{BufReader, BufWriter};
use std::{fs::File, path::Path};

use crate::references::references::{read_mapper, Extractor};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use crate::references::enums::ItemTypeId;

#[derive(Debug, Serialize)]
pub struct MonsterRef {
    /// Record index relative to the Mondun struct array.
    pub index: i32,
    pub file_id: i32,
    pub mon_id: i32,
    pub pos_x: i32,
    pub pos_y: i32,
    pub loot1_item_id: u8,
    pub loot1_item_type: ItemTypeId,
    pub loot2_item_id: u8,
    pub loot2_item_type: ItemTypeId,
    pub loot3_item_id: u8,
    pub loot3_item_type: ItemTypeId,

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

            reader.read_i32::<LittleEndian>()?;
            reader.read_i32::<LittleEndian>()?;
            reader.read_i32::<LittleEndian>()?;
            reader.read_i32::<LittleEndian>()?;
            reader.read_i32::<LittleEndian>()?;

            let loot1_item_id = reader.read_u8()?;
            let loot1_item_type_raw = reader.read_u8()?;
            reader.read_u8()?;
            reader.read_u8()?;

            let loot2_item_id = reader.read_u8()?;
            let loot2_item_type_raw = reader.read_u8()?;
            reader.read_u8()?;
            reader.read_u8()?;

            let loot3_item_id = reader.read_u8()?;
            let loot3_item_type_raw = reader.read_u8()?;
            reader.read_u8()?;
            reader.read_u8()?;

            reader.read_i32::<LittleEndian>()?; // 1 or 0
            reader.read_i32::<LittleEndian>()?;

            let loot1_item_type = ItemTypeId::from_u8(loot1_item_type_raw).unwrap_or(ItemTypeId::Unknown);
            let loot2_item_type = ItemTypeId::from_u8(loot2_item_type_raw).unwrap_or(ItemTypeId::Unknown);
            let loot3_item_type = ItemTypeId::from_u8(loot3_item_type_raw).unwrap_or(ItemTypeId::Unknown);

            refs.push(MonsterRef {
                index: i,
                file_id,
                mon_id,
                pos_x,
                pos_y,
                loot1_item_id,
                loot1_item_type,
                loot2_item_id,
                loot2_item_type,
                loot3_item_id,
                loot3_item_type,
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

            writer.write_i32::<LittleEndian>(0)?;
            writer.write_i32::<LittleEndian>(0)?;
            writer.write_i32::<LittleEndian>(0)?;
            writer.write_i32::<LittleEndian>(0)?;
            writer.write_i32::<LittleEndian>(0)?;

            writer.write_u8(record.loot1_item_id)?;
            writer.write_u8(u8::from(record.loot1_item_type))?;
            writer.write_u8(0)?;
            writer.write_u8(0)?;

            writer.write_u8(record.loot2_item_id)?;
            writer.write_u8(u8::from(record.loot2_item_type))?;
            writer.write_u8(0)?;
            writer.write_u8(0)?;

            writer.write_u8(record.loot3_item_id)?;
            writer.write_u8(u8::from(record.loot3_item_type))?;
            writer.write_u8(0)?;
            writer.write_u8(0)?;

            writer.write_i32::<LittleEndian>(0)?;
            writer.write_i32::<LittleEndian>(0)?;
        }
        Ok(())
    }
}

pub fn read_monster_ref(source_path: &Path) -> std::io::Result<Vec<MonsterRef>> {
    MonsterRef::read_file(source_path)
}
