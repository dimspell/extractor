use byteorder::{LittleEndian, ReadBytesExt};
use serde::Serialize;
use std::fs::File;
use std::io::{BufReader, Result};
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct PartyLevelRecord {
    pub id: u32,
    pub field2: u32,
    pub field3: u32,
    pub field4_a: u16,
    pub field4_b: u16,
    pub field5: u32,
    pub field6: u32,
    pub field7: u32,
    pub field8_a: u16,
    pub field8_b: u16,
}

#[derive(Debug, Serialize)]
pub struct PartyLevelNpc {
    pub npc_index: usize,
    pub records: Vec<PartyLevelRecord>,
}

pub fn read_party_level_db(source_path: &Path) -> Result<Vec<PartyLevelNpc>> {
    let file = File::open(source_path)?;
    let mut reader = BufReader::new(file);
    let mut npcs = Vec::new();

    // The file is 5760 bytes. Based on reverse engineering:
    // 8 NPCs * 720 bytes = 5760 bytes.
    // Each 720 byte block is 20 sub-blocks of 36 bytes.
    // Each 36 byte sub-block starts with a 4-byte sentinel followed by 8 u32 data fields.

    for npc_index in 0..8 {
        let mut records = Vec::new();
        for _block_idx in 0..20 {
            let _sentinel = reader.read_u32::<LittleEndian>()?;

            // Each block has 8 u32 values
            let id = reader.read_u32::<LittleEndian>()?;
            let f2 = reader.read_u32::<LittleEndian>()?;
            let f3 = reader.read_u32::<LittleEndian>()?;
            let f4_a = reader.read_u16::<LittleEndian>()?;
            let f4_b = reader.read_u16::<LittleEndian>()?;

            let f5 = reader.read_u32::<LittleEndian>()?;
            let f6 = reader.read_u32::<LittleEndian>()?;
            let f7 = reader.read_u32::<LittleEndian>()?;
            let f8_a = reader.read_u16::<LittleEndian>()?;
            let f8_b = reader.read_u16::<LittleEndian>()?;

            records.push(PartyLevelRecord {
                id,
                field2: f2,
                field3: f3,
                field4_a: f4_a,
                field4_b: f4_b,
                field5: f5,
                field6: f6,
                field7: f7,
                field8_a: f8_a,
                field8_b: f8_b,
            });
        }
        npcs.push(PartyLevelNpc { npc_index, records });
    }

    Ok(npcs)
}
