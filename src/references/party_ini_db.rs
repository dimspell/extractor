use crate::references::extractor::Extractor;
use dispel_macros::{Extractor, Localizable};
use rusqlite::{params, Connection, Result as DbResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// PartyIni.db - Party Member Initial Configurations
///
/// Stores initial metadata and starting configurations for party members.
///
/// Reads file: `NpcInGame/PrtIni.db`
///
/// # Binary Format
///
/// - **Encoding**: Little-endian for all numeric values
/// - **Text Encoding**: UTF-8 for `name` (20 bytes, null-padded)
/// - **Record Size**: 28 bytes (20 + 4 + 4)
/// - **Header**: None; fixed 8 records (party size limit)
///
/// ```text
/// +--------------------------------------+
/// | PartyIni.db - Party Initial Config|
/// +--------------------------------------+
/// | Encoding: Binary (Little-Endian)     |
/// | Text Encoding: UTF-8                 |
/// | Record Size: 28 bytes               |
/// | Fixed: 8 records (party size)      |
/// +--------------------------------------+
/// | [Record 1] - 28 bytes               |
/// | - name: 20 bytes (UTF-8, null-pad) |
/// | - unknown1: u8                       |
/// | - unknown2: u8                       |
/// | - unknown3: u8                       |
/// | - unknown4: u8                       |
/// | - unknown5: u16                      |
/// | - unknown6: u16                      |
/// +--------------------------------------+
/// | [Record 2]                           |
/// | ... (same structure) ...             |
/// +--------------------------------------+
/// | ... (8 total records)                |
/// +--------------------------------------+
/// ```
///
/// # Field Categories
///
/// - **Identification**: `name` (20 bytes, UTF-8, null-padded)
/// - **Unknown Fields**: `unknown1-6` (need investigation)
///
/// # Special Values
///
/// - `name`: 20 bytes max, null-padded (UTF-8)
/// - `unknown1-4`: u8 fields, observed as 0
/// - `unknown5-6`: u16 fields, observed as 0
///
/// # File Purpose
///
/// Defines initial party member configurations
/// with names and unknown parameters. Used for
/// party initialization and character setup.
#[derive(Debug, Clone, Serialize, Default, Deserialize, Extractor, Localizable)]
#[extractor(counter_size = 0, property_item_size = 28)]
pub struct PartyIniNpc {
    /// Null-terminated root character identifier string.
    #[extractor(string(encoding = "UTF-8", size = 20))]
    #[translatable(encoding = "WINDOWS_1250", max_bytes = 20)]
    pub name: String,
    #[extractor(primitive(type = "u8"))]
    pub unknown1: u8,
    #[extractor(primitive(type = "u8"))]
    pub unknown2: u8,
    #[extractor(primitive(type = "u8"))]
    pub unknown3: u8,
    #[extractor(primitive(type = "u8"))]
    pub unknown4: u8,
    #[extractor(primitive(type = "u16"))]
    pub unknown5: u16,
    #[extractor(primitive(type = "u16"))]
    pub unknown6: u16,
}

pub fn read_party_ini_db(source_path: &Path) -> std::io::Result<Vec<PartyIniNpc>> {
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
        let names = [
            "Alice", "Bob", "Carol", "Dave", "Eve", "Frank", "Grace", "Hank",
        ];
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

    #[test]
    fn serialize_round_trip() {
        let names = ["Hero", "Mage", "Warrior", "Rogue", "", "", "", ""];
        let data = eight_records(&names);
        let mut c = Cursor::new(&data[..]);
        let records = PartyIniNpc::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        PartyIniNpc::to_writer(&records, &mut out).unwrap();
        assert_eq!(out, data);
    }
}
