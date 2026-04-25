use crate::references::extractor::Extractor;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rusqlite::{params, Connection, Result as DbResult};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufWriter, Read, Result, Seek, Write};
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PartyIniNpc {
    /// Null-terminated root character identifier string.
    pub name: String,

    pub unknown1: u8,
    pub unknown2: u8,
    pub unknown3: u8,
    pub unknown4: u8,
    pub unknown5: u16,
    pub unknown6: u16,
}

/// Stores initial metadata and starting configurations for party members.
///
/// Reads file: `NpcInGame/PrtIni.db`
/// # File Format: `NpcInGame/PrtIni.db`
///
/// Binary file, little-endian.  Starts with a 4-byte i32 record count.
/// Each record:
/// - `name`  : null-terminated string (variable length up to buffer)
/// - `data`: 8 bytes
impl Extractor for PartyIniNpc {
    fn parse<R: Read + Seek>(reader: &mut R, _len: u64) -> Result<Vec<Self>> {
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

            let unknown1 = reader.read_u8()?;
            let unknown2 = reader.read_u8()?;
            let unknown3 = reader.read_u8()?;
            let unknown4 = reader.read_u8()?;
            let unknown5 = reader.read_u16::<LittleEndian>()?;
            let unknown6 = reader.read_u16::<LittleEndian>()?;

            npcs.push(PartyIniNpc {
                name,
                unknown1,
                unknown2,
                unknown3,
                unknown4,
                unknown5,
                unknown6,
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
            writer.write_u8(record.unknown1)?;
            writer.write_u8(record.unknown2)?;
            writer.write_u8(record.unknown3)?;
            writer.write_u8(record.unknown4)?;
            writer.write_u16::<LittleEndian>(record.unknown5)?;
            writer.write_u16::<LittleEndian>(record.unknown6)?;
        }

        Ok(())
    }
}

pub fn read_party_ini_db(source_path: &Path) -> Result<Vec<PartyIniNpc>> {
    PartyIniNpc::read_file(source_path)
}

pub fn save_party_inis(conn: &mut Connection, npcs: &[PartyIniNpc]) -> DbResult<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_party_ini.sql"))?;
        for (idx, npc) in npcs.iter().enumerate() {
            stmt.execute(params![
                idx as i32,
                npc.name,
                npc.unknown1 as i32,
                npc.unknown2 as i32,
                npc.unknown3 as i32,
                npc.unknown4 as i32,
                npc.unknown5 as i32,
                npc.unknown6 as i32,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn npc_record(name: &str) -> [u8; 28] {
        let mut buf = [0u8; 28];
        let b = name.as_bytes();
        let n = b.len().min(19);
        buf[..n].copy_from_slice(&b[..n]);
        // bytes 20-27 stay zero (unknown1-6)
        buf
    }

    fn eight_records(names: &[&str; 8]) -> Vec<u8> {
        let mut data = Vec::with_capacity(224);
        for &name in names {
            data.extend_from_slice(&npc_record(name));
        }
        data
    }

    #[test]
    fn parse_all_eight_npcs() {
        let names = ["Alice", "Bob", "Carol", "Dave", "Eve", "Frank", "Grace", "Hank"];
        let data = eight_records(&names);
        assert_eq!(data.len(), 224);

        let mut c = Cursor::new(&data[..]);
        let npcs = PartyIniNpc::parse(&mut c, 224).unwrap();
        assert_eq!(npcs.len(), 8);
        assert_eq!(npcs[0].name, "Alice");
        assert_eq!(npcs[7].name, "Hank");
    }

    #[test]
    fn parse_empty_slots() {
        let names = ["", "", "", "", "", "", "", ""];
        let data = eight_records(&names);
        let mut c = Cursor::new(&data[..]);
        let npcs = PartyIniNpc::parse(&mut c, 224).unwrap();
        assert_eq!(npcs.len(), 8);
        assert!(npcs[0].name.is_empty());
    }
}
