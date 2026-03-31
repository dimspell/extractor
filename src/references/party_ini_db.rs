use crate::references::extractor::Extractor;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rusqlite::{params, Connection, Result as DbResult};
use serde::Serialize;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Result, Write};
use std::path::Path;

// ===========================================================================
// PRTINI.DB FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | PrtIni.db - Party NPC Metadata       |
// +--------------------------------------+
// | Encoding: Binary (Little-Endian)     |
// | Record Size: 28 bytes                |
// | Fixed 8 records (224 bytes total)    |
// +--------------------------------------+
// | [Record 1]                           |
// | - name: 20 bytes (null-terminated)    |
// | - flags: u16                         |
// | - kind: u16                          |
// | - value: u32                         |
// +--------------------------------------+
// | [Record 2]                           |
// | ... (same structure) ...             |
// +--------------------------------------+
// | ... (8 total records) ...            |
// +--------------------------------------+
//
// FIELD DEFINITIONS:
// - name: Character name (20 bytes max)
// - flags: Behavior and status flags
// - kind: Character class/type
// - value: Associated numeric value
//
// SPECIAL VALUES:
// - Fixed 8 records (party size limit)
// - 224-byte total file size
// - Null-terminated names
//
// FILE PURPOSE:
// Stores metadata for party NPCs including names,
// classes, and status flags. Used for party management
// and character initialization.
//
// ===========================================================================

#[derive(Debug, Serialize)]
pub struct PartyIniNpc {
    /// Null-terminated root character identifier string.
    pub name: String,
    /// Binary metadata governing operational behavior.
    pub flags: u16,
    /// Role specialization tag or ID class parameter.
    pub kind: u16,
    /// Sub-identifier linking variables together.
    pub value: u32,
}

/// Stores initial metadata and starting configurations for party members.
///
/// Reads file: `NpcInGame/PrtIni.db`
/// # File Format: `NpcInGame/PrtIni.db`
///
/// Binary file, little-endian.  Starts with a 4-byte i32 record count.
/// Each record:
/// - `name`  : null-terminated string (variable length up to buffer)
/// - `flags` : u16
/// - `kind`  : u16
/// - `value` : u32
impl Extractor for PartyIniNpc {
    fn read_file(source_path: &Path) -> Result<Vec<Self>> {
        let file = File::open(source_path)?;
        let mut reader = BufReader::new(file);
        let mut npcs = Vec::new();

        // The file is 224 bytes, which is 8 NPCs * 28 bytes each.
        // Each 28 byte record consists of:
        // - name: 20 bytes (null-terminated ASCII)
        // - flags: u16
        // - kind: u16
        // - value: u32
        for _ in 0..8 {
            let mut name_bytes = [0u8; 20];
            reader.read_exact(&mut name_bytes)?;

            // Find the first null byte to terminate the string
            let name = name_bytes
                .split(|&b| b == 0)
                .next()
                .map(|b| String::from_utf8_lossy(b).to_string())
                .unwrap_or_default();

            let flags = reader.read_u16::<LittleEndian>()?;
            let kind = reader.read_u16::<LittleEndian>()?;
            let value = reader.read_u32::<LittleEndian>()?;

            npcs.push(PartyIniNpc {
                name,
                flags,
                kind,
                value,
            });
        }

        Ok(npcs)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> Result<()> {
        let file = File::create(dest_path)?;
        let mut writer = BufWriter::new(file);

        for record in records {
            let mut name_bytes = [0u8; 20];
            let name_bytes_val = record.name.as_bytes();
            let len = std::cmp::min(name_bytes_val.len(), 19); // Keep at least one \0 if string is long
            name_bytes[..len].copy_from_slice(&name_bytes_val[..len]);

            writer.write_all(&name_bytes)?;
            writer.write_u16::<LittleEndian>(record.flags)?;
            writer.write_u16::<LittleEndian>(record.kind)?;
            writer.write_u32::<LittleEndian>(record.value)?;
        }

        Ok(())
    }
}

pub fn read_party_ini_db(source_path: &Path) -> Result<Vec<PartyIniNpc>> {
    PartyIniNpc::read_file(source_path)
}

pub fn save_party_inis(conn: &mut Connection, npcs: &Vec<PartyIniNpc>) -> DbResult<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_party_ini.sql"))?;
        for (idx, npc) in npcs.iter().enumerate() {
            stmt.execute(params![
                idx as i32,
                npc.name,
                npc.flags as i32,
                npc.kind as i32,
                npc.value as i32,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
