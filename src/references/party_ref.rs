use std::path::Path;

use crate::references::enums::GhostFaceId;
use crate::references::extractor::Extractor;
use dispel_macros::{TextExtractor, TextRecordPatcher};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

/// PartyRef.ref - Party Characters
///
/// Stores character definitions and references for the party.
///
/// Reads file: `Ref/PartyRef.ref`
///
/// # ASCII Structure
///
/// ```text
/// +--------------------------------------+
/// | PartyRef.ref - Party Characters     |
/// +--------------------------------------+
/// | Encoding: WINDOWS-1250              |
/// | Format: CSV with comments            |
/// | Record Size: Variable (text)        |
/// +--------------------------------------+
/// | ; Comment line                      |
/// | id,name,job,map_id,npc_id,dlg_out,dlg_in,ghost|
/// | 1,Hero,null,1,1,100,101,1           |
/// | 2,Warrior,Fighter,1,2,102,103,2     |
/// | ...                                 |
/// +--------------------------------------+
/// ```
///
/// # Field Definitions
///
/// - `id`: Unique character identifier
/// - `full_name`: Character display name or "null" (shown as `name` in file)
/// - `job_name`: Character class/job or "null" (shown as `job` in file)
/// - `root_map_id`: Origin map ID where character is found (shown as `map_id` in file)
/// - `npc_id`: Linked NPC record ID
/// - `dlg_when_not_in_party`: Dialog ID when not in party (shown as `dlg_out` in file)
/// - `dlg_when_in_party`: Dialog ID when in party (shown as `dlg_in` in file)
/// - `ghost_face_id`: Ghost face/sprite ID for UI (shown as `ghost` in file)
///
/// # Special Values
///
/// - `"null"` literal for missing `full_name`/`job_name` fields
/// - Lines starting with `;` are comments
/// - CSV format with comma delimiter
///
/// # File Purpose
///
/// Defines all party characters with their names, classes, origin locations,
/// dialog references, and visual representations. Used for party management,
/// recruitment, and character interaction systems.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TextExtractor, TextRecordPatcher)]
#[extractor(encoding = "WINDOWS_1250")]
#[patcher(filename = "PartyRef.ref")]
pub struct PartyRef {
    /// Party member identifier.
    #[extractor(field = 0)]
    pub id: i32,
    /// Display name of the party character.
    #[extractor(field = 1, parse_null)]
    pub full_name: Option<String>,
    /// Character class or job title.
    #[extractor(field = 2, parse_null)]
    pub job_name: Option<String>,
    /// Origin map identifier where the character is found.
    #[extractor(field = 3)]
    pub root_map_id: i32,
    /// NPC record ID this character is linked to.
    #[extractor(field = 4)]
    pub npc_id: i32,
    /// Dialog topic when the character is roaming/not recruited.
    #[extractor(field = 5)]
    pub dlg_when_not_in_party: i32,
    /// Dialog topic when the character is actively grouped.
    #[extractor(field = 6)]
    pub dlg_when_in_party: i32,
    /// Sprite ID for their UI portrait or ghost form.
    #[extractor(field = 7, enum_from_i32(type = "GhostFaceId"))]
    pub ghost_face_id: GhostFaceId,
}

pub fn read_part_refs(source_path: &Path) -> std::io::Result<Vec<PartyRef>> {
    PartyRef::read_file(source_path)
}

pub fn save_party_refs(conn: &mut Connection, party_refs: &[PartyRef]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_party_ref.sql"))?;
        for party_ref in party_refs {
            stmt.execute(params![
                party_ref.id,
                party_ref.full_name,
                party_ref.job_name,
                party_ref.root_map_id,
                party_ref.npc_id,
                party_ref.dlg_when_not_in_party,
                party_ref.dlg_when_in_party,
                i32::from(party_ref.ghost_face_id),
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::references::enums::GhostFaceId;
    use std::io::Cursor;

    #[test]
    fn parse_entry() {
        let data = b"1,Hero,Fighter,2,5,100,101,0\n";
        let mut c = Cursor::new(data.as_ref());
        let refs = PartyRef::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].id, 1);
        assert_eq!(refs[0].full_name.as_deref(), Some("Hero"));
        assert_eq!(refs[0].job_name.as_deref(), Some("Fighter"));
        assert_eq!(refs[0].root_map_id, 2);
        assert_eq!(refs[0].npc_id, 5);
        assert_eq!(refs[0].dlg_when_not_in_party, 100);
        assert_eq!(refs[0].dlg_when_in_party, 101);
        assert_eq!(refs[0].ghost_face_id, GhostFaceId::None);
    }

    #[test]
    fn parse_null_names() {
        let data = b"2,null,null,1,1,0,0,0\n";
        let mut c = Cursor::new(data.as_ref());
        let refs = PartyRef::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(refs[0].full_name, None);
        assert_eq!(refs[0].job_name, None);
    }

    #[test]
    fn serialize_round_trip() {
        let data = b"1,Hero,Fighter,2,5,100,101,0\r\n";
        let mut c = Cursor::new(data.as_ref());
        let records = PartyRef::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        PartyRef::to_writer(&records, &mut out).unwrap();
        let mut c2 = Cursor::new(out.as_slice());
        let records2 = PartyRef::parse(&mut c2, out.len() as u64).unwrap();
        assert_eq!(records.len(), records2.len());
        assert_eq!(records[0].id, records2[0].id);
        assert_eq!(records[0].full_name, records2[0].full_name);
        assert_eq!(records[0].root_map_id, records2[0].root_map_id);
    }
}
