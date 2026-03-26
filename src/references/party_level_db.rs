use crate::references::references::Extractor;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::fs::File;
use std::io::{BufReader, BufWriter, Result};
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct PartyLevelRecord {
    pub level: u32,
    pub strength: u32,
    pub constitution: u32,
    pub wisdom: u32,
    pub health_points: u16,
    pub magic_points: u16,
    pub agility: u32,
    pub attack: u32,
    pub mana_recharge: u32,
    pub defense: u16,
}

#[derive(Debug, Serialize)]
pub struct PartyLevelNpc {
    pub npc_index: usize,
    pub records: Vec<PartyLevelRecord>,
}

impl Extractor for PartyLevelNpc {
    fn read_file(source_path: &Path) -> Result<Vec<Self>> {
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
                let strength = reader.read_u32::<LittleEndian>()?;
                let constitution = reader.read_u32::<LittleEndian>()?;
                let wisdom = reader.read_u32::<LittleEndian>()?;
                let health_points = reader.read_u16::<LittleEndian>()?;
                let magic_points = reader.read_u16::<LittleEndian>()?;

                let agility = reader.read_u32::<LittleEndian>()?;
                let attack = reader.read_u32::<LittleEndian>()?;
                let mana_recharge = reader.read_u32::<LittleEndian>()?;
                let defense = reader.read_u16::<LittleEndian>()?;
                let _ = reader.read_u16::<LittleEndian>()?; // Null byte (\0)

                records.push(PartyLevelRecord {
                    level: _block_idx + 1 as u32,
                    strength,
                    constitution,
                    wisdom,
                    health_points,
                    magic_points,
                    agility,
                    attack,
                    mana_recharge,
                    defense,
                });
            }
            npcs.push(PartyLevelNpc { npc_index, records });
        }

        Ok(npcs)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> Result<()> {
        let file = File::create(dest_path)?;
        let mut writer = BufWriter::new(file);

        for npc in records {
            for record in &npc.records {
                writer.write_u32::<LittleEndian>(0)?; // sentinel

                writer.write_u32::<LittleEndian>(record.strength)?;
                writer.write_u32::<LittleEndian>(record.constitution)?;
                writer.write_u32::<LittleEndian>(record.wisdom)?;
                writer.write_u16::<LittleEndian>(record.health_points)?;
                writer.write_u16::<LittleEndian>(record.magic_points)?;

                writer.write_u32::<LittleEndian>(record.agility)?;
                writer.write_u32::<LittleEndian>(record.attack)?;
                writer.write_u32::<LittleEndian>(record.mana_recharge)?;
                writer.write_u16::<LittleEndian>(record.defense)?;
                writer.write_u16::<LittleEndian>(0)?; // null byte
            }
        }

        Ok(())
    }
}

pub fn read_party_level_db(source_path: &Path) -> Result<Vec<PartyLevelNpc>> {
    PartyLevelNpc::read_file(source_path)
}
