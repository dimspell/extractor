use crate::references::extractor::Extractor;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rusqlite::{Connection, Result as DbResult, params};
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Result, Seek, Write};
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
    /// Accuracy rate; the following three bytes are reserved.
    pub accuracy: u8,
    pub reserved_0x15: u8,
    pub reserved_0x16: u8,
    pub reserved_0x17: u8,
    /// Dodge rate; the following three bytes are reserved.
    pub dodge: u8,
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

/// Size of one on-disk progression row, matching `PARTY_LEVEL_RECORD.record_size`.
const RECORD_SIZE: u64 = 36;

/// Read one 36-byte progression row in on-disk field order. The derived
/// `level` field is left at zero; callers assign it from the row's position.
fn parse_record<R: Read>(reader: &mut R) -> Result<PartyLevelRecord> {
    Ok(PartyLevelRecord {
        level: 0,
        magic_spell_id_1: reader.read_u8()?,
        magic_spell_id_2: reader.read_u8()?,
        magic_spell_id_3: reader.read_u8()?,
        reserved_0x03: reader.read_u8()?,
        strength: reader.read_u32::<LittleEndian>()?,
        constitution: reader.read_u32::<LittleEndian>()?,
        wisdom: reader.read_u32::<LittleEndian>()?,
        health_points: reader.read_u16::<LittleEndian>()?,
        mana_points: reader.read_u16::<LittleEndian>()?,
        accuracy: reader.read_u8()?,
        reserved_0x15: reader.read_u8()?,
        reserved_0x16: reader.read_u8()?,
        reserved_0x17: reader.read_u8()?,
        dodge: reader.read_u8()?,
        reserved_0x19: reader.read_u8()?,
        reserved_0x1a: reader.read_u8()?,
        reserved_0x1b: reader.read_u8()?,
        weapon_skill_level: reader.read_u32::<LittleEndian>()?,
        tactical_action_chance: reader.read_u32::<LittleEndian>()?,
    })
}

/// Write one 36-byte progression row in on-disk field order. The derived
/// `level` field is not serialized.
fn write_record<W: Write>(record: &PartyLevelRecord, writer: &mut W) -> Result<()> {
    writer.write_u8(record.magic_spell_id_1)?;
    writer.write_u8(record.magic_spell_id_2)?;
    writer.write_u8(record.magic_spell_id_3)?;
    writer.write_u8(record.reserved_0x03)?;
    writer.write_u32::<LittleEndian>(record.strength)?;
    writer.write_u32::<LittleEndian>(record.constitution)?;
    writer.write_u32::<LittleEndian>(record.wisdom)?;
    writer.write_u16::<LittleEndian>(record.health_points)?;
    writer.write_u16::<LittleEndian>(record.mana_points)?;
    writer.write_u8(record.accuracy)?;
    writer.write_u8(record.reserved_0x15)?;
    writer.write_u8(record.reserved_0x16)?;
    writer.write_u8(record.reserved_0x17)?;
    writer.write_u8(record.dodge)?;
    writer.write_u8(record.reserved_0x19)?;
    writer.write_u8(record.reserved_0x1a)?;
    writer.write_u8(record.reserved_0x1b)?;
    writer.write_u32::<LittleEndian>(record.weapon_skill_level)?;
    writer.write_u32::<LittleEndian>(record.tactical_action_chance)?;
    Ok(())
}

impl Extractor for PartyLevelRecord {
    fn parse<R: Read + Seek>(reader: &mut R, len: u64) -> Result<Vec<Self>> {
        let count = len / RECORD_SIZE;
        let mut records = Vec::with_capacity(count as usize);
        for index in 0..count {
            let mut record = parse_record(reader)?;
            record.level = (index + 1) as u32;
            records.push(record);
        }
        let trailing = len % RECORD_SIZE;
        if trailing != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "party level data has {trailing} trailing bytes after {count} complete records"
                ),
            ));
        }
        Ok(records)
    }

    fn to_writer<W: Write>(records: &[Self], writer: &mut W) -> Result<()> {
        for record in records {
            write_record(record, writer)?;
        }
        Ok(())
    }
}

impl Extractor for PartyLevelNpc {
    fn parse<R: Read + Seek>(reader: &mut R, _len: u64) -> Result<Vec<Self>> {
        let mut npcs = Vec::with_capacity(8);
        for npc_index in 0..8 {
            let mut records = Vec::with_capacity(20);
            for level_index in 0..20 {
                let mut record = parse_record(reader)?;
                record.level = level_index + 1;
                records.push(record);
            }
            npcs.push(PartyLevelNpc { npc_index, records });
        }
        Ok(npcs)
    }

    fn to_writer<W: Write>(npcs: &[Self], writer: &mut W) -> Result<()> {
        for npc in npcs {
            for record in &npc.records {
                write_record(record, writer)?;
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
    let mut stmt = tx.prepare(crate::database::insert_sql("party_levels"))?;
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
                record.accuracy as i32,
                record.reserved_0x15 as i32,
                record.reserved_0x16 as i32,
                record.reserved_0x17 as i32,
                record.dodge as i32,
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

    #[test]
    fn parse_single_record_reads_all_fields_and_derives_level() {
        let block = level_block();
        let records =
            PartyLevelRecord::parse(&mut Cursor::new(&block), block.len() as u64).unwrap();
        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record.level, 1);
        assert_eq!(record.magic_spell_id_1, 3);
        assert_eq!(record.magic_spell_id_2, 1);
        assert_eq!(record.magic_spell_id_3, 0xff);
        assert_eq!(record.reserved_0x03, 0);
        assert_eq!(record.strength, 100);
        assert_eq!(record.constitution, 0);
        assert_eq!(record.wisdom, 0);
        assert_eq!(record.health_points, 50);
        assert_eq!(record.mana_points, 0);
        assert_eq!(record.accuracy, 30);
        assert_eq!(record.reserved_0x15, 7);
        assert_eq!(record.reserved_0x16, 8);
        assert_eq!(record.reserved_0x17, 9);
        assert_eq!(record.dodge, 15);
        assert_eq!(record.reserved_0x19, 10);
        assert_eq!(record.reserved_0x1a, 11);
        assert_eq!(record.reserved_0x1b, 12);
        assert_eq!(record.weapon_skill_level, 4);
        assert_eq!(record.tactical_action_chance, 35);
    }

    #[test]
    fn parse_two_records_numbers_levels_from_one() {
        let block = level_block();
        let data: Vec<u8> = block.into_iter().chain(block).collect();
        let records = PartyLevelRecord::parse(&mut Cursor::new(&data), data.len() as u64).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].level, 1);
        assert_eq!(records[1].level, 2);
    }

    #[test]
    fn parse_truncated_trailing_bytes_is_an_error() {
        let data = full_file();
        let truncated = &data[..data.len() - 1];
        let err = PartyLevelRecord::parse(&mut Cursor::new(truncated), truncated.len() as u64)
            .expect_err("an incomplete trailing row must not be dropped silently");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);

        let short = [0u8; 5];
        let err = PartyLevelRecord::parse(&mut Cursor::new(&short), short.len() as u64)
            .expect_err("an incomplete single row must not be dropped silently");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn record_to_writer_reproduces_exact_bytes_and_ignores_level() {
        let block = level_block();
        let mut records =
            PartyLevelRecord::parse(&mut Cursor::new(&block), block.len() as u64).unwrap();
        let mut out = Vec::new();
        PartyLevelRecord::to_writer(&records, &mut out).unwrap();
        assert_eq!(out, block);

        records[0].level = 99;
        let mut out = Vec::new();
        PartyLevelRecord::to_writer(&records, &mut out).unwrap();
        assert_eq!(out, block, "level is derived and must not affect bytes");
    }

    #[test]
    fn read_file_loads_records() {
        let block = level_block();
        let data: Vec<u8> = block.into_iter().chain(block).collect();
        let path = std::env::temp_dir().join(format!(
            "dispel_party_level_record_test_{}_{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path, &data).unwrap();
        let records = PartyLevelRecord::read_file(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].level, 1);
        assert_eq!(records[1].level, 2);
    }
}
