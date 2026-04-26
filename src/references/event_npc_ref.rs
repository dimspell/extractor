use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read, Seek, Write};
use std::path::Path;

use crate::references::extractor::Extractor;

// ===========================================================================
// EVENTNPC.REF FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | EventNpc.ref - Event NPC Definitions |
// +--------------------------------------+
// | Encoding: WINDOWS-1250              |
// | Format: CSV with comments            |
// | Record Size: Variable (text)        |
// +--------------------------------------+
// | ; Comment line                       |
// | id,event_id,name                      |
// | 1,1001,Quest Giver                    |
// | 2,1002,Special Merchant              |
// | ...                                   |
// +--------------------------------------+
//
// FIELD DEFINITIONS:
// - id: Unique event NPC identifier
// - event_id: Linked event script ID
// - name: NPC display name
//
// EVENT LINKING:
// - event_id links to Event.ini entries
// - NPCs appear only during specific events
// - Removed after event completion
// - Can trigger quests and dialogues
//
// SPECIAL VALUES:
// - Lines starting with ";" are comments
// - CSV format with comma delimiter
// - Empty lines ignored
//
// FILE PURPOSE:
// Defines NPCs that appear only during specific scripted
// events. Used for quest-related characters, temporary
// merchants, and story-critical encounters.
//
// ===========================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventNpcRef {
    /// Linear structural tracker.
    pub id: i32,
    /// Parent overarching event linking context.
    pub event_id: i32,
    /// Descriptive string of placeholder entity.
    pub name: String,
}

/// Stores specific placements for NPCs that appear only during scripted events.
///
/// Reads file: `NpcInGame/Eventnpc.ref`
/// # File Format: `NpcInGame/Eventnpc.ref`
///
/// Binary file, little-endian.  Starts with a 4-byte i32 record count.
/// Each record:
/// - `id`       : i32
/// - `event_id` : i32
/// - `name`     : fixed-size null-padded string
impl Extractor for EventNpcRef {
    fn parse<R: Read + Seek>(reader: &mut R, _len: u64) -> std::io::Result<Vec<Self>> {
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1250))
                .build(reader.by_ref()),
        );
        let mut npc_refs: Vec<EventNpcRef> = Vec::new();
        for line in reader.lines().map_while(Result::ok) {
            if line.starts_with(";") || line.trim().is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split(",").collect();
            if parts.len() < 3 {
                continue;
            }

            let id = parts[0].trim().parse::<i32>().unwrap_or(0);
            let event_id = parts[1].trim().parse::<i32>().unwrap_or(0);
            let name = parts[2].trim().to_string();

            npc_refs.push(EventNpcRef { id, event_id, name });
        }
        Ok(npc_refs)
    }

    fn to_writer<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
        for record in records {
            let line = format!("{},{},{}\r\n", record.id, record.event_id, record.name);
            let (cow, _, _) = WINDOWS_1250.encode(&line);
            writer.write_all(&cow)?;
        }
        Ok(())
    }
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
