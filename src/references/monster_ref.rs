use std::io::BufReader;
use std::{fs::File, path::Path};

use crate::references::references::read_mapper;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MonsterRef {
    pub index: i32,
    pub file_id: i32,
    pub mon_id: i32,
    pub pos_x: i32,
    pub pos_y: i32,
    pub loot1_item_id: u8,
    pub loot1_item_type: u8,
    pub loot2_item_id: u8,
    pub loot2_item_type: u8,
    pub loot3_item_id: u8,
    pub loot3_item_type: u8,
}

pub fn read_monster_ref(source_path: &Path) -> std::io::Result<Vec<MonsterRef>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 4;
    const PROPERTY_ITEM_SIZE: i32 = 14 * 4;
    // const FILLER: u8 = 0;

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
        let loot1_item_type = reader.read_u8()?;
        reader.read_u8()?;
        reader.read_u8()?;

        let loot2_item_id = reader.read_u8()?;
        let loot2_item_type = reader.read_u8()?;
        reader.read_u8()?;
        reader.read_u8()?;

        let loot3_item_id = reader.read_u8()?;
        let loot3_item_type = reader.read_u8()?;
        reader.read_u8()?;
        reader.read_u8()?;

        reader.read_i32::<LittleEndian>()?; // 1 or 0
        reader.read_i32::<LittleEndian>()?;

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
