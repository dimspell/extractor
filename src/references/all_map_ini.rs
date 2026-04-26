use std::io::{BufRead, BufReader, Read, Seek, Write};
use std::path::Path;

use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

use crate::references::enums::MapLighting;
use crate::references::extractor::{parse_null, Extractor};

// ===========================================================================
// ALLMAP.INI FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +------------------------------+
// | AllMap.ini - Master Map List |
// +------------------------------+
// | Encoding: WINDOWS-1250      |
// | Format: CSV with comments    |
// +------------------------------+
// | ; Comment line              |
// | id,map_file,name,pgp,dlg,lit|
// | 1,cat1,Category 1,null,null,0 |
// | 2,cat2,Category 2,null,null,0 |
// | ...                         |
// +------------------------------+
//
// FIELD DEFINITIONS:
// - id: Unique map identifier (1, 2, 3, ...)
// - map_file: .map filename (e.g., "cat1")
// - name: Display name shown in game
// - pgp: Conversation script filename or "null"
// - dlg: Dialog text filename or "null"
// - lit: 0=dark/dungeon, 1=lit/outdoor
//
// SPECIAL VALUES:
// - "null" literal for missing PGP/DLG files
// - Lines starting with ";" are comments
// - CSV format with comma delimiter
//
// FILE PURPOSE:
// Master index of all game maps, linking map IDs to filenames and metadata.
// Used by the game engine to load the correct map files and associated assets.
//
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Map {
    /// Map identifier.
    pub id: i32,
    /// Filename of the .map file.
    pub map_filename: String,
    /// Display name of the map.
    pub map_name: String,
    /// Filename of the associated converstation script file.
    pub pgp_filename: Option<String>,
    /// Filename of the associated dialog file.
    pub dlg_filename: Option<String>,
    // light - 0=light, 1=darkness
    /// Light indicator (0=light, 1=darkness).
    pub lighting: MapLighting,
}

/// Stores the general list of all maps in the game.
///
/// Reads file: `AllMap.ini`
/// # File Format: `AllMap.ini`
///
/// Text file, WINDOWS-1250 encoded. One map entry per line.
/// Lines starting with `;` are comments and are skipped.
///
/// Each line follows the CSV format:
/// ```text
/// id,map_filename,map_name,pgp_filename,npc_dlg_filename,is_dark
/// ```
/// - `pgp_filename` / `npc_dlg_filename` use literal `null` when absent.
/// - `is_dark`: `0` = lit map, `1` = dark/dungeon map.
impl Extractor for Map {
    fn parse<R: Read + Seek>(reader: &mut R, _len: u64) -> std::io::Result<Vec<Self>> {
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1250))
                .build(reader.by_ref()),
        );

        let mut maps: Vec<Map> = Vec::new();
        for line in reader.lines().map_while(Result::ok) {
            if line.starts_with(";") {
                continue;
            }

            let parts: Vec<&str> = line.split(",").collect();
            if parts.len() < 6 {
                continue;
            }
            let id: i32 = parts[0].parse::<i32>().unwrap();
            let map_filename = parts[1].to_string();
            let map_name = parts[2].to_string();
            let pgp_filename = parse_null(parts[3]);
            let dlg_filename = parse_null(parts[4]);
            let lighting = if parts[5] == "1" {
                MapLighting::Light
            } else {
                MapLighting::Dark
            };

            maps.push(Map {
                id,
                map_filename,
                map_name,
                pgp_filename,
                dlg_filename,
                lighting,
            });
        }
        Ok(maps)
    }

    fn to_writer<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
        for record in records {
            let pgp = record
                .pgp_filename
                .clone()
                .unwrap_or_else(|| "null".to_string());
            let dlg = record
                .dlg_filename
                .clone()
                .unwrap_or_else(|| "null".to_string());
            let light_str = if record.lighting == MapLighting::Light {
                "1"
            } else {
                "0"
            };
            let line = format!(
                "{},{},{},{},{},{}\r\n",
                record.id, record.map_filename, record.map_name, pgp, dlg, light_str
            );
            let (cow, _, _) = WINDOWS_1250.encode(&line);
            writer.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_all_map_ini(source_path: &Path) -> std::io::Result<Vec<Map>> {
    Map::read_file(source_path)
}

pub fn save_maps(conn: &mut Connection, maps: &[Map]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_map.sql"))?;
        for map in maps {
            stmt.execute(params![
                map.id,
                map.map_filename,
                map.map_name,
                map.pgp_filename,
                map.dlg_filename,
                i32::from(map.lighting),
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::references::enums::MapLighting;
    use std::io::Cursor;

    #[test]
    fn parse_two_entries() {
        let data = b"; comment\n1,cat1,Forest,null,null,0\r\n2,cat2,Dungeon,pgp2,dlg2,1\r\n";
        let mut cursor = Cursor::new(data.as_ref());
        let maps = Map::parse(&mut cursor, data.len() as u64).unwrap();

        assert_eq!(maps.len(), 2);

        assert_eq!(maps[0].id, 1);
        assert_eq!(maps[0].map_filename, "cat1");
        assert_eq!(maps[0].map_name, "Forest");
        assert!(maps[0].pgp_filename.is_none());
        assert!(maps[0].dlg_filename.is_none());
        assert_eq!(maps[0].lighting, MapLighting::Dark);

        assert_eq!(maps[1].id, 2);
        assert_eq!(maps[1].pgp_filename.as_deref(), Some("pgp2"));
        assert_eq!(maps[1].dlg_filename.as_deref(), Some("dlg2"));
        assert_eq!(maps[1].lighting, MapLighting::Light);
    }

    #[test]
    fn parse_skips_comments_and_empty_lines() {
        let data = b"; header\n; another comment\n\n1,m1,Map One,null,null,0\n";
        let mut cursor = Cursor::new(data.as_ref());
        let maps = Map::parse(&mut cursor, data.len() as u64).unwrap();
        assert_eq!(maps.len(), 1);
        assert_eq!(maps[0].map_name, "Map One");
    }

    #[test]
    fn parse_empty() {
        let mut cursor = Cursor::new(b"" as &[u8]);
        let maps = Map::parse(&mut cursor, 0).unwrap();
        assert!(maps.is_empty());
    }

    #[test]
    fn serialize_round_trip() {
        let data = b"1,cat1,Forest,null,null,0\r\n2,cat2,Dungeon,pgp2,dlg2,1\r\n";
        let mut c = Cursor::new(data.as_ref());
        let records = Map::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        Map::to_writer(&records, &mut out).unwrap();
        let mut c2 = Cursor::new(out.as_slice());
        let records2 = Map::parse(&mut c2, out.len() as u64).unwrap();
        assert_eq!(records.len(), records2.len());
        assert_eq!(records[0].id, records2[0].id);
        assert_eq!(records[0].map_name, records2[0].map_name);
        assert_eq!(records[1].pgp_filename, records2[1].pgp_filename);
    }
}
