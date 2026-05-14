use std::path::Path;

use crate::references::extractor::Extractor;
use dispel_macros::{Localizable, TextExtractor, TextRecordPatcher};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

/// EventNpc.ref - Event NPC Definitions
///
/// Stores specific placements for NPCs that appear only during scripted events.
///
/// Reads file: `NpcInGame/Eventnpc.ref`
///
/// # ASCII Structure
///
/// ```text
/// ; Comment line
/// id,event_id,name
/// 1,1001,Quest Giver
/// 2,1002,Special Merchant
/// ...
/// ```
///
/// # Field Definitions
///
/// - `id`: Unique event NPC identifier
/// - `event_id`: Linked event script ID
/// - `name`: NPC display name
///
/// # Event Linking
///
/// - `event_id` links to Event script entries
/// - NPCs appear only during specific events
/// - Removed after event completion
/// - Can trigger quests and dialogues
///
/// # Special Values
///
/// - Lines starting with `;` are comments
/// - CSV format with comma delimiter
/// - Empty lines ignored
///
/// # File Purpose
///
/// Defines NPCs that appear only during specific scripted
/// events. Used for quest-related characters, temporary
/// merchants, and story-critical encounters.
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, TextExtractor, Localizable, TextRecordPatcher,
)]
#[extractor(encoding = "WINDOWS_1250")]
#[patcher(filename = "Eventnpc.ref")]
pub struct EventNpcRef {
    /// Linear structural tracker.
    #[extractor(field = 0)]
    pub id: i32,
    /// Parent overarching event linking context.
    #[extractor(field = 1)]
    pub event_id: i32,
    /// Descriptive string of placeholder entity.
    #[extractor(field = 2)]
    #[translatable(encoding = "WINDOWS-1250", max_bytes = 1024)]
    pub name: String,
}

pub fn read_event_npc_ref(source_path: &Path) -> std::io::Result<Vec<EventNpcRef>> {
    EventNpcRef::read_file(source_path)
}

pub fn save_event_npc_refs(conn: &mut Connection, npc_refs: &[EventNpcRef]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_event_npc_ref.sql"))?;
        for npc in npc_refs {
            stmt.execute(params![npc.id, npc.event_id, npc.name])?;
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
        let data = b"1,100,Guard Bob\n2,200,Merchant\n";
        let mut c = Cursor::new(data.as_ref());
        let refs = EventNpcRef::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].id, 1);
        assert_eq!(refs[0].event_id, 100);
        assert_eq!(refs[0].name, "Guard Bob");
        assert_eq!(refs[1].name, "Merchant");
    }

    #[test]
    fn parse_skips_comments_and_empty() {
        let data = b"; comment\n\n1,0,NPC\n";
        let mut c = Cursor::new(data.as_ref());
        let refs = EventNpcRef::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(refs.len(), 1);
    }

    #[test]
    fn serialize_round_trip() {
        let data = b"1,100,Guard Bob\r\n2,200,Merchant\r\n";
        let mut c = Cursor::new(data.as_ref());
        let records = EventNpcRef::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        EventNpcRef::to_writer(&records, &mut out).unwrap();
        let mut c2 = Cursor::new(out.as_slice());
        let records2 = EventNpcRef::parse(&mut c2, out.len() as u64).unwrap();
        assert_eq!(records.len(), records2.len());
        assert_eq!(records[0].id, records2[0].id);
        assert_eq!(records[0].name, records2[0].name);
        assert_eq!(records[1].event_id, records2[1].event_id);
    }
}
