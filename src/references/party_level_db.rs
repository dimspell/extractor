use byteorder::{LittleEndian, ReadBytesExt};
use serde::Serialize;
use std::fs::File;
use std::io::{BufReader, Result};
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct PartyLevelRecord {
    /// Potentially ID or Type
    pub id: u32,
    /// Potentially Level or Stat category
    pub field2: u32,
    /// Potentially value or modifier
    pub field3: u32,
    /// Potentially value or modifier
    pub field4: u32,
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

            // Each block has 8 u32 values, which we interpret as two 4-u32 records.
            // This matches the "record-oriented" nature mentioned in the docs.
            for _r in 0..2 {
                let id = reader.read_u32::<LittleEndian>()?;
                let f2 = reader.read_u32::<LittleEndian>()?;
                let f3 = reader.read_u32::<LittleEndian>()?;
                let f4 = reader.read_u32::<LittleEndian>()?;

                records.push(PartyLevelRecord {
                    id,
                    field2: f2,
                    field3: f3,
                    field4: f4,
                });
            }
        }
        npcs.push(PartyLevelNpc { npc_index, records });
    }

    Ok(npcs)
}
