use crate::references::extractor::Extractor;
use dispel_macros::{Extractor, Localizable, RecordPatcher};
use rusqlite::{Connection, Result as DbResult, params};
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
/// | - reserved_0x14: u8                  |
/// | - class_id: u8                       |
/// | - starting_level: u8                 |
/// | - pathfinding_mode: u8               |
/// | - character_variant: u32             |
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
/// - **Character setup**: class, starting level, navigation mode, and variant
///
/// # Special Values
///
/// - `name`: 20 bytes max, null-padded (UTF-8)
/// - `class_id`: Shipped values are 21 through 24; this selects class-specific
///   runtime behavior and titles.
/// - `pathfinding_mode`: Shipped value is 7; passed to map/path queries.
/// - `character_variant`: Shipped values are 0 or 1; selects one of two variants
///   for each class.
///
/// # File Purpose
///
/// Defines initial party member configurations
/// with names and unknown parameters. Used for
/// party initialization and character setup.
#[derive(Debug, Clone, Serialize, Default, Deserialize, Extractor, Localizable, RecordPatcher)]
#[extractor(counter_size = 0, property_item_size = 28)]
#[patcher(filename = "PrtIni.db")]
pub struct PartyIniNpc {
    /// Null-terminated root character identifier string.
    #[extractor(string(encoding = "UTF-8", size = 20))]
    #[translatable(encoding = "WINDOWS_1250", max_bytes = 20)]
    pub name: String,
    /// Reserved byte at offset `0x14`; zero in every shipped record.
    #[extractor(primitive(type = "u8"))]
    pub reserved_0x14: u8,
    /// Character class identifier. Shipped values are 21–24.
    #[extractor(primitive(type = "u8"))]
    pub class_id: u8,
    /// Initial level used when the party character is created.
    #[extractor(primitive(type = "u8"))]
    pub starting_level: u8,
    /// Mode passed to the game's map/path queries; all shipped records use 7.
    #[extractor(primitive(type = "u8"))]
    pub pathfinding_mode: u8,
    /// Class-specific variant selector. The game uses values 0 and 1 to choose
    /// different title and level-up behavior for otherwise matching classes.
    #[extractor(primitive(type = "u32"))]
    pub character_variant: u32,
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
                npc.reserved_0x14 as i32,
                npc.class_id as i32,
                npc.starting_level as i32,
                npc.pathfinding_mode as i32,
                npc.character_variant as i64,
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
        // bytes 20-27 stay zero (configuration fields)
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

    #[test]
    fn parse_character_configuration_fields() {
        let mut data = Vec::with_capacity(224);
        let mut first = npc_record("Hero");
        first[20] = 0;
        first[21] = 23;
        first[22] = 10;
        first[23] = 7;
        first[24..28].copy_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(&first);
        for _ in 1..8 {
            data.extend_from_slice(&npc_record(""));
        }

        let mut c = Cursor::new(&data[..]);
        let records = PartyIniNpc::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(records[0].reserved_0x14, 0);
        assert_eq!(records[0].class_id, 23);
        assert_eq!(records[0].starting_level, 10);
        assert_eq!(records[0].pathfinding_mode, 7);
        assert_eq!(records[0].character_variant, 1);
    }
}
