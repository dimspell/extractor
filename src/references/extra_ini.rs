use std::io::{BufRead, BufReader, Read, Seek, Write};
use std::path::Path;

use crate::references::extractor::{parse_null, Extractor};
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
// - sprite_filename: SPR filename or "null"
// - unknown: Flag (0 or 1)
// - description: Object description or "null"
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Extra {
    /// Tool or object identifier.
    pub id: i32,
    /// Base SPR filename for the object.
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
    fn parse<R: Read + Seek>(reader: &mut R, _len: u64) -> std::io::Result<Vec<Self>> {
        let decoded = DecodeReaderBytesBuilder::new()
            .encoding(Some(EUC_KR))
            .build(reader.by_ref());
        let buf_reader = BufReader::new(decoded);
        let mut extras: Vec<Extra> = Vec::new();
        for line in buf_reader.lines().map_while(std::io::Result::ok) {
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
        Ok(extras)
    }

    fn serialize<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
        for record in records {
            let sprite = record.sprite_filename.as_deref().unwrap_or("null");
            let desc = record.description.as_deref().unwrap_or("null");
            let line = format!("{},{},{},{}\r\n", record.id, sprite, record.unknown, desc);
            let (cow, _, _) = EUC_KR.encode(&line);
            writer.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_extra_ini(source_path: &Path) -> std::io::Result<Vec<Extra>> {
    Extra::read_file(source_path)
}

pub fn save_extras(conn: &mut Connection, extras: &[Extra]) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parse_two_entries() {
        let data = b"1,chest.spr,0,Wooden Chest\n2,null,1,null\n";
        let mut c = Cursor::new(data.as_ref());
        let extras = Extra::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(extras.len(), 2);
        assert_eq!(extras[0].id, 1);
        assert_eq!(extras[0].sprite_filename.as_deref(), Some("chest.spr"));
        assert_eq!(extras[0].unknown, 0);
        assert_eq!(extras[0].description.as_deref(), Some("Wooden Chest"));
        assert_eq!(extras[1].sprite_filename, None);
        assert_eq!(extras[1].unknown, 1);
        assert_eq!(extras[1].description, None);
    }

    #[test]
    fn parse_skips_comments() {
        let data = b"; comment\n1,spr.spr,0,Desc\n";
        let mut c = Cursor::new(data.as_ref());
        let extras = Extra::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(extras.len(), 1);
    }
}
