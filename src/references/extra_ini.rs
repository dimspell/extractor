use std::io::{BufRead, BufReader, Write};
use std::{fs::File, path::Path};

use crate::references::references::{parse_null, Extractor};
use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

// ===========================================================================
// EXTRA.INI FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | Extra.ini - Interactive Objects      |
// +--------------------------------------+
// | Encoding: EUC-KR                     |
// | Format: CSV with comments             |
// | Record Size: Variable (text)         |
// +--------------------------------------+
// | ; Comment line                       |
// | id,sprite_filename,unknown,description|
// | 1,chest.spr,0,Wooden Chest           |
// | 2,door.spr,1,Iron Door               |
// | ...                                  |
// +--------------------------------------+
//
// FIELD DEFINITIONS:
// - id: Unique interactive object ID
// - sprite_filename: SPR/SPX filename or "null"
// - unknown: Flag (0 or 1)
// - description: Object description or "null"
//
// OBJECT TYPES (by ID range):
// - 1-100: Containers (chests, barrels)
// - 101-200: Doors and gates
// - 201-300: Switches and levers
// - 301-400: Readable objects
// - 401-500: Destructible objects
// - 501-600: Teleporters and portals
//
// UNKNOWN FLAG MEANINGS:
// - 0: Standard interactive object
// - 1: Special/quest-related object
//
// SPECIAL VALUES:
// - "null" literal for missing fields
// - Lines starting with ";" are comments
// - CSV format with comma delimiter
//
// FILE PURPOSE:
// Defines interactive objects with visual assets and descriptions.
// Used for environmental interaction, puzzles, and object-based
// quest systems. Linked to map placements via REF files.
//
// ===========================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct Extra {
    /// Tool or object identifier.
    pub id: i32,
    /// Base SPR/SPX filename for the object.
    pub sprite_filename: Option<String>,
    /// Internal unknown flag.
    pub unknown: i32,
    /// Optional description for the interactive object.
    pub description: Option<String>,
}

/// Stores definitions and types for interactive objects (extras).
///
/// Reads file: `Extra.ini`
/// # File Format: `Extra.ini`
///
/// Text file, EUC-KR encoded. One record per line, CSV format:
/// ```text
/// id,sprite_filename,unknown_flag,description
/// ```
/// - `sprite_filename` and `description` use literal `null` when absent.
/// - `unknown_flag` is always `0` or `1`.
impl Extractor for Extra {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let f = File::open(source_path)?;
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(EUC_KR))
                .build(f),
        );
        let mut extras: Vec<Extra> = Vec::new();
        for line in reader.lines() {
            if let Ok(line) = line {
                if line.starts_with(";") {
                    continue;
                }

                let parts: Vec<&str> = line.split(",").collect();
                if parts.len() < 4 {
                    continue;
                }
                let id: i32 = parts[0].parse::<i32>().unwrap();
                let sprite_filename = parse_null(parts[1]);
                let unknown = parts[2].parse::<i32>().unwrap();
                let description = parse_null(parts[3]);

                extras.push(Extra {
                    id,
                    sprite_filename,
                    unknown,
                    description,
                });
            }
        }
        Ok(extras)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(dest_path)?;
        for record in records {
            let sprite = record.sprite_filename.as_deref().unwrap_or("null");
            let desc = record.description.as_deref().unwrap_or("null");
            let line = format!("{},{},{},{}\r\n", record.id, sprite, record.unknown, desc);
            let (cow, _, _) = EUC_KR.encode(&line);
            file.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_extra_ini(source_path: &Path) -> std::io::Result<Vec<Extra>> {
    Extra::read_file(source_path)
}

pub fn save_extras(conn: &mut Connection, extras: &Vec<Extra>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_extra.sql"))?;
        for extra in extras {
            stmt.execute(params![
                extra.id,
                extra.sprite_filename,
                extra.unknown,
                extra.description,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
