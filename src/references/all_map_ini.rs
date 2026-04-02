use std::io::{BufRead, BufReader, Write};
use std::{fs::File, path::Path};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let f = File::open(source_path)?;
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1250))
                .build(f),
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

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(dest_path)?;
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
            file.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_all_map_ini(source_path: &Path) -> std::io::Result<Vec<Map>> {
    Map::read_file(source_path)
}

pub fn save_maps(conn: &mut Connection, maps: &Vec<Map>) -> Result<()> {
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
