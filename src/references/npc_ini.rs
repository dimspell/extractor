use std::path::Path;

use crate::references::extractor::Extractor;
use dispel_macros::TextExtractor;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

/// Npc.ini - NPC Visual References
///
/// Stores visual references and configuration for NPCs.
///
/// Reads file: `Npc.ini`
///
/// # ASCII Structure
///
/// ```text
/// +--------------------------------------+
/// | Npc.ini - NPC Visual References      |
/// +--------------------------------------+
/// | Encoding: EUC-KR                    |
/// | Format: CSV with comments            |
/// | Record Size: Variable (text)        |
/// +--------------------------------------+
/// | ; Comment line                       |
/// | id,sprite_filename,description       |
/// | 1,guard.spr,City Guard              |
/// | 2,merchant.spr,Shopkeeper           |
/// | ...                                  |
/// +--------------------------------------+
/// ```
///
/// # Field Definitions
///
/// - `id`: Unique NPC visual type identifier
/// - `sprite_filename`: SPR filename or "null"
/// - `description`: NPC role/appearance description
///
/// # Special Values
///
/// - `"null"` literal for missing sprite filenames
/// - Lines starting with `;` are comments
/// - CSV format with comma delimiter
///
/// # File Purpose
///
/// Defines visual appearances for NPC characters with sprite
/// filenames and descriptions. Used for rendering NPCs in
/// the game world and linking to NPC behavior scripts.
#[derive(Debug, Clone, Serialize, Deserialize, Default, TextExtractor)]
#[extractor(encoding = "EUC_KR")]
pub struct NpcIni {
    /// NPC visual type identifier.
    #[extractor(field = 0)]
    pub id: i32,
    /// Sprite filename representing the NPC.
    #[extractor(field = 1, parse_null)]
    pub sprite_filename: Option<String>,
    /// Internal description or role of the NPC.
    #[extractor(field = 2)]
    pub description: String,
}

pub fn read_npc_ini(source_path: &Path) -> std::io::Result<Vec<NpcIni>> {
    NpcIni::read_file(source_path)
}

pub fn save_npc_inis(conn: &mut Connection, npc_inis: &[NpcIni]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_npc_ini.sql"))?;
        for npc_ini in npc_inis {
            stmt.execute(params![
                npc_ini.id,
                npc_ini.sprite_filename,
                npc_ini.description,
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
    fn parse_entries() {
        let data = b"1,guard.spr,City Guard\n2,null,Merchant\n";
        let mut c = Cursor::new(data.as_ref());
        let npcs = NpcIni::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(npcs.len(), 2);
        assert_eq!(npcs[0].id, 1);
        assert_eq!(npcs[0].sprite_filename.as_deref(), Some("guard.spr"));
        assert_eq!(npcs[0].description, "City Guard");
        assert_eq!(npcs[1].sprite_filename, None);
        assert_eq!(npcs[1].description, "Merchant");
    }

    #[test]
    fn parse_skips_comments_and_empty() {
        let data = b"; comment\n\n1,spr.spr,Guard\n";
        let mut c = Cursor::new(data.as_ref());
        let npcs = NpcIni::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(npcs.len(), 1);
    }

    #[test]
    fn serialize_round_trip() {
        let data = b"1,guard.spr,City Guard\r\n2,null,Merchant\r\n";
        let mut c = Cursor::new(data.as_ref());
        let records = NpcIni::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        NpcIni::to_writer(&records, &mut out).unwrap();
        let mut c2 = Cursor::new(out.as_slice());
        let records2 = NpcIni::parse(&mut c2, out.len() as u64).unwrap();
        assert_eq!(records.len(), records2.len());
        assert_eq!(records[0].id, records2[0].id);
        assert_eq!(records[0].sprite_filename, records2[0].sprite_filename);
        assert_eq!(records[1].description, records2[1].description);
    }
}
