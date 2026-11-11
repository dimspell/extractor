use std::path::Path;

use crate::references::extractor::Extractor;
use dispel_macros::{TextExtractor, TextRecordPatcher};
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

// Stores definitions and types for interactive objects (extras).
//
// Reads file: `Extra.ini`
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
// | id,sprite_filename,activation_sprite_frame_mode,description |
// | 1,chest.spr,0,Wooden Chest           |
// | 2,door.spr,1,Iron Door               |
// | ...                                  |
// +--------------------------------------+
//
// FIELD DEFINITIONS:
// - id: Unique interactive object ID
// - sprite_filename: SPR filename or "null"
// - activation_sprite_frame_mode: Selects the sprite frame used after activation
// - description: Object description or "null"
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, TextExtractor, TextRecordPatcher)]
#[extractor(encoding = "EUC_KR")]
#[patcher(filename = "Extra.ini")]
pub struct Extra {
    /// Tool or object identifier.
    #[extractor(field = 0)]
    pub id: i32,
    /// Base SPR filename for the object.
    #[extractor(field = 1, parse_null)]
    pub sprite_filename: Option<String>,
    /// Selects the sprite frame used after the object is activated.
    ///
    /// Every placement using this definition carries a copy of this value.
    /// For object interaction handlers 5, 6, and 8, a value greater than `1`
    /// selects activated sprite frame `1`; `0` and `1` select frame `0` on
    /// that path. Observed definitions use `0`, `1`, and `2`, so this is not a
    /// boolean or a quest flag.
    #[extractor(field = 2)]
    pub activation_sprite_frame_mode: i32,
    /// Optional editor-facing description for the interactive object.
    ///
    /// The executable loader stops after the preceding activation-frame mode,
    /// so it does not copy this fourth column into the runtime definition table.
    #[extractor(field = 3, parse_null)]
    pub description: Option<String>,
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
                extra.activation_sprite_frame_mode,
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
        assert_eq!(extras[0].activation_sprite_frame_mode, 0);
        assert_eq!(extras[0].description.as_deref(), Some("Wooden Chest"));
        assert_eq!(extras[1].sprite_filename, None);
        assert_eq!(extras[1].activation_sprite_frame_mode, 1);
        assert_eq!(extras[1].description, None);
    }

    #[test]
    fn parse_skips_comments() {
        let data = b"; comment\n1,spr.spr,0,Desc\n";
        let mut c = Cursor::new(data.as_ref());
        let extras = Extra::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(extras.len(), 1);
    }

    #[test]
    fn serialize_round_trip() {
        let data = b"1,chest.spr,0,Wooden Chest\r\n2,null,1,null\r\n";
        let mut c = Cursor::new(data.as_ref());
        let records = Extra::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        Extra::to_writer(&records, &mut out).unwrap();
        let mut c2 = Cursor::new(out.as_slice());
        let records2 = Extra::parse(&mut c2, out.len() as u64).unwrap();
        assert_eq!(records.len(), records2.len());
        assert_eq!(records[0].id, records2[0].id);
        assert_eq!(records[0].sprite_filename, records2[0].sprite_filename);
        assert_eq!(
            records[0].activation_sprite_frame_mode,
            records2[0].activation_sprite_frame_mode
        );
        assert_eq!(records[1].sprite_filename, records2[1].sprite_filename);
        assert_eq!(
            records[1].activation_sprite_frame_mode,
            records2[1].activation_sprite_frame_mode
        );
    }
}
