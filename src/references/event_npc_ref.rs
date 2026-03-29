use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rusqlite::{params, Connection, Result};
use serde::Serialize;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::references::references::Extractor;

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
// NPC CATEGORIES:
// - 1-50: Quest-related NPCs
// - 51-100: Story-critical NPCs
// - 101-150: Temporary event NPCs
// - 151-200: Special merchants/traders
// - 201-250: Hidden/secret NPCs
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

#[derive(Debug, Serialize)]
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
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let f = File::open(source_path)?;
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1250))
                .build(f),
        );
        let mut npc_refs: Vec<EventNpcRef> = Vec::new();
        for line in reader.lines() {
            if let Ok(line) = line {
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
        }
        Ok(npc_refs)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(dest_path)?;
        for record in records {
            let line = format!("{},{},{}\r\n", record.id, record.event_id, record.name);
            let (cow, _, _) = WINDOWS_1250.encode(&line);
            file.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_event_npc_ref(source_path: &Path) -> std::io::Result<Vec<EventNpcRef>> {
    EventNpcRef::read_file(source_path)
}

pub fn save_event_npc_refs(conn: &mut Connection, npc_refs: &Vec<EventNpcRef>) -> Result<()> {
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
