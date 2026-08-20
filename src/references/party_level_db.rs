use crate::references::extractor::Extractor;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rusqlite::{Connection, Result as DbResult, params};
use serde::{Deserialize, Serialize};
use std::io::{Read, Result, Seek, Write};
use std::path::Path;

/// One 36-byte (`0x24`) level-progression entry from `PrtLevel.db`.
///
/// The game addresses entries as `party_slot * 0x2d0 + (level - 1) * 0x24`.
/// The three action IDs and all reserved bytes are retained so editing a stat
/// cannot silently change a party member's available actions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PartyLevelRecord {
    /// Derived from the zero-based record position; it is not stored on disk.
    pub level: u32,
    /// First magic-spell ID available at this level (`0xff` means absent).
    pub magic_spell_id_1: u8,
    /// Second magic-spell ID available at this level (`0xff` means absent).
    pub magic_spell_id_2: u8,
    /// Third magic-spell ID available at this level (`0xff` means absent).
    pub magic_spell_id_3: u8,
    /// Alignment/reserved byte at offset `0x03`.
    pub reserved_0x03: u8,
    pub strength: u32,
    pub constitution: u32,
    pub wisdom: u32,
    pub health_points: u16,
    pub mana_points: u16,
    /// Agility at offset `0x14`; the following three bytes are reserved.
    pub agility: u8,
    pub reserved_0x15: u8,
    pub reserved_0x16: u8,
    pub reserved_0x17: u8,
    /// Attack-related stat at offset `0x18`; the following three bytes are reserved.
    pub attack: u8,
    pub reserved_0x19: u8,
    pub reserved_0x1a: u8,
    pub reserved_0x1b: u8,
    /// Shared weapon-skill/proficiency level used in weapon calculations.
    pub weapon_skill_level: u32,
    /// Percentage threshold used after level 10 to trigger a tactical action.
    pub tactical_action_chance: u32,
}

/// Fixed table of 20 progression entries for one of the eight party slots.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PartyLevelNpc {
    pub npc_index: usize,
    pub records: Vec<PartyLevelRecord>,
}

/// Stub Extractor so `GenericEditorState<PartyLevelRecord>` can be used in the GUI.
/// The actual binary format is embedded inside `PartyLevelNpc::parse`.
impl Extractor for PartyLevelRecord {
    fn parse<R: Read + Seek>(_reader: &mut R, _len: u64) -> std::io::Result<Vec<Self>> {
        Ok(Vec::new())
    }

    fn to_writer<W: Write>(_records: &[Self], _writer: &mut W) -> std::io::Result<()> {
        Ok(())
    }
}

impl Extractor for PartyLevelNpc {
    fn parse<R: Read + Seek>(reader: &mut R, _len: u64) -> Result<Vec<Self>> {
        let mut npcs = Vec::with_capacity(8);
        for npc_index in 0..8 {
            let mut records = Vec::with_capacity(20);
            for level_index in 0..20 {
                records.push(PartyLevelRecord {
                    level: level_index + 1,
                    magic_spell_id_1: reader.read_u8()?,
                    magic_spell_id_2: reader.read_u8()?,
                    magic_spell_id_3: reader.read_u8()?,
                    reserved_0x03: reader.read_u8()?,
                    strength: reader.read_u32::<LittleEndian>()?,
                    constitution: reader.read_u32::<LittleEndian>()?,
                    wisdom: reader.read_u32::<LittleEndian>()?,
                    health_points: reader.read_u16::<LittleEndian>()?,
                    mana_points: reader.read_u16::<LittleEndian>()?,
                    agility: reader.read_u8()?,
                    reserved_0x15: reader.read_u8()?,
                    reserved_0x16: reader.read_u8()?,
                    reserved_0x17: reader.read_u8()?,
                    attack: reader.read_u8()?,
                    reserved_0x19: reader.read_u8()?,
                    reserved_0x1a: reader.read_u8()?,
                    reserved_0x1b: reader.read_u8()?,
                    weapon_skill_level: reader.read_u32::<LittleEndian>()?,
                    tactical_action_chance: reader.read_u32::<LittleEndian>()?,
                });
            }
            npcs.push(PartyLevelNpc { npc_index, records });
        }
        Ok(npcs)
    }

    fn to_writer<W: Write>(npcs: &[Self], writer: &mut W) -> std::io::Result<()> {
        for npc in npcs {
            for record in &npc.records {
                writer.write_u8(record.magic_spell_id_1)?;
                writer.write_u8(record.magic_spell_id_2)?;
                writer.write_u8(record.magic_spell_id_3)?;
                writer.write_u8(record.reserved_0x03)?;
                writer.write_u32::<LittleEndian>(record.strength)?;
                writer.write_u32::<LittleEndian>(record.constitution)?;
                writer.write_u32::<LittleEndian>(record.wisdom)?;
                writer.write_u16::<LittleEndian>(record.health_points)?;
                writer.write_u16::<LittleEndian>(record.mana_points)?;
                writer.write_u8(record.agility)?;
                writer.write_u8(record.reserved_0x15)?;
                writer.write_u8(record.reserved_0x16)?;
                writer.write_u8(record.reserved_0x17)?;
                writer.write_u8(record.attack)?;
                writer.write_u8(record.reserved_0x19)?;
                writer.write_u8(record.reserved_0x1a)?;
                writer.write_u8(record.reserved_0x1b)?;
                writer.write_u32::<LittleEndian>(record.weapon_skill_level)?;
                writer.write_u32::<LittleEndian>(record.tactical_action_chance)?;
            }
        }
        Ok(())
    }
}

pub fn read_party_level_db(source_path: &Path) -> Result<Vec<PartyLevelNpc>> {
    PartyLevelNpc::read_file(source_path)
}

pub fn save_party_levels(conn: &mut Connection, npcs: &[PartyLevelNpc]) -> DbResult<()> {
    let tx = conn.transaction()?;
    let mut stmt = tx.prepare(include_str!("../queries/insert_party_level.sql"))?;
    for npc in npcs {
        for record in &npc.records {
            stmt.execute(params![
                npc.npc_index as i32,
                record.level as i32,
                record.magic_spell_id_1 as i32,
                record.magic_spell_id_2 as i32,
                record.magic_spell_id_3 as i32,
                record.reserved_0x03 as i32,
                record.strength as i64,
                record.constitution as i64,
                record.wisdom as i64,
                record.health_points as i32,
                record.mana_points as i32,
                record.agility as i32,
                record.reserved_0x15 as i32,
                record.reserved_0x16 as i32,
                record.reserved_0x17 as i32,
                record.attack as i32,
                record.reserved_0x19 as i32,
                record.reserved_0x1a as i32,
                record.reserved_0x1b as i32,
                record.weapon_skill_level as i64,
                record.tactical_action_chance as i64,
            ])?;
        }
    }
    drop(stmt);
    tx.commit()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn level_block() -> [u8; 36] {
        let mut buf = [0u8; 36];
        buf[0..4].copy_from_slice(&[3, 1, 0xff, 0]);
        buf[4..8].copy_from_slice(&100u32.to_le_bytes());
        buf[16..18].copy_from_slice(&50u16.to_le_bytes());
        buf[20..24].copy_from_slice(&[30, 7, 8, 9]);
        buf[24..28].copy_from_slice(&[15, 10, 11, 12]);
        buf[28..32].copy_from_slice(&4u32.to_le_bytes());
        buf[32..36].copy_from_slice(&35u32.to_le_bytes());
        buf
    }

    fn full_file() -> Vec<u8> {
        let block = level_block();
        std::iter::repeat_n(block, 160).flatten().collect()
    }

    #[test]
    fn parse_all_npcs_and_levels() {
        let data = full_file();
        let npcs = PartyLevelNpc::parse(&mut Cursor::new(&data), 5760).unwrap();
        assert_eq!(npcs.len(), 8);
        assert_eq!(npcs[0].records.len(), 20);
        assert_eq!(npcs[0].records[0].level, 1);
        assert_eq!(npcs[0].records[19].level, 20);
    }

    #[test]
    fn parse_preserves_action_ids_and_nonzero_reserved_bytes() {
        let data = full_file();
        let records = PartyLevelNpc::parse(&mut Cursor::new(&data), data.len() as u64).unwrap();
        let record = &records[0].records[0];
        assert_eq!(
            [
                record.magic_spell_id_1,
                record.magic_spell_id_2,
                record.magic_spell_id_3,
            ],
            [3, 1, 0xff]
        );
        assert_eq!(
            [
                record.reserved_0x15,
                record.reserved_0x16,
                record.reserved_0x17
            ],
            [7, 8, 9]
        );
        assert_eq!(
            [
                record.reserved_0x19,
                record.reserved_0x1a,
                record.reserved_0x1b
            ],
            [10, 11, 12]
        );
        assert_eq!(record.weapon_skill_level, 4);
        assert_eq!(record.tactical_action_chance, 35);
    }

    #[test]
    fn serialize_round_trip() {
        let data = full_file();
        let records = PartyLevelNpc::parse(&mut Cursor::new(&data), data.len() as u64).unwrap();
        let mut out = Vec::new();
        PartyLevelNpc::to_writer(&records, &mut out).unwrap();
        assert_eq!(out, data);
    }
}
