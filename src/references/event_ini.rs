use std::path::Path;

use crate::references::enums::EventType;
use crate::references::extractor::Extractor;
use dispel_macros::{TextExtractor, TextRecordPatcher};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

/// Stores script and event mappings.
///
/// Reads file: `Event.ini`
///
/// Text file, EUC-KR encoded. One record per line, CSV format:
/// ```text
/// event_id,required_event_id,event_type_id,event_filename,counter
/// 1,0,1,script1.scr,0
/// 2,1,2,script2.scr,5
/// ```
///
/// # Field Definitions
///
/// - `event_id`: Unique event identifier
/// - `prev_id`: Prerequisite event ID
/// - `type`: Execution condition type
/// - `filename`: Script file or "null"
/// - `counter`: Execution limit (N times)
///
/// # Event Types
///
/// - `1`: Execute once unconditionally
/// - `2`: Execute N times (uses counter)
/// - `3`: Execute if previous succeeded
/// - `4`: Execute on map load
/// - `5`: Execute on dialog trigger
///
/// # Special Values
///
/// - `"null"` literal for missing filenames
/// - Lines starting with `;` are comments
/// - CSV format with comma delimiter
/// - `counter = 0`: No limit (infinite)
///
/// # File Purpose
///
/// Defines event scripts with execution conditions, prerequisites,
/// and repetition limits. Used for quest progression, interactive
/// objects, and game state management.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TextExtractor, TextRecordPatcher)]
#[extractor(encoding = "EUC_KR")]
#[patcher(filename = "Event.ini")]
pub struct Event {
    /// Unique event identifier.
    #[extractor(field = 0)]
    pub event_id: i32,
    /// Prerequisite event ID that must have occurred.
    #[extractor(field = 1)]
    pub required_event_id: i32,
    /// Determines execution condition (e.g. unconditionally, N times, if previous succeeded).
    #[extractor(field = 2, enum_from_i32(type = "EventType"))]
    pub event_type: EventType,
    /// Filename of the event script.
    #[extractor(field = 3, parse_null)]
    pub event_filename: Option<String>,
    /// Execution counter (N limit) for types that execute multiple times.
    #[extractor(field = 4)]
    pub counter: i32,
}

pub fn read_event_ini(source_path: &Path) -> std::io::Result<Vec<Event>> {
    Event::read_file(source_path)
}

pub fn save_events(conn: &mut Connection, events: &[Event]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_event.sql"))?;
        for event in events {
            stmt.execute(params![
                event.event_id,
                event.required_event_id,
                i32::from(event.event_type),
                event.event_filename,
                event.counter,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::references::enums::EventType;
    use std::io::Cursor;

    #[test]
    fn parse_events() {
        let data = b"100,0,2,script.scr,5\n200,100,0,null,0\n";
        let mut c = Cursor::new(data.as_ref());
        let events = Event::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_id, 100);
        assert_eq!(events[0].required_event_id, 0);
        assert_eq!(events[0].event_type, EventType::Conditional);
        assert_eq!(events[0].event_filename.as_deref(), Some("script.scr"));
        assert_eq!(events[0].counter, 5);
        assert_eq!(events[1].event_id, 200);
        assert_eq!(events[1].event_filename, None);
        assert_eq!(events[1].event_type, EventType::Unknown);
    }

    #[test]
    fn parse_skips_comments_and_short_lines() {
        let data = b"; header\n1,0,0,null,1\n";
        let mut c = Cursor::new(data.as_ref());
        let events = Event::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn serialize_round_trip() {
        let data = b"100,0,2,script.scr,5\r\n200,100,0,null,0\r\n";
        let mut c = Cursor::new(data.as_ref());
        let records = Event::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        Event::to_writer(&records, &mut out).unwrap();
        let mut c2 = Cursor::new(out.as_slice());
        let records2 = Event::parse(&mut c2, out.len() as u64).unwrap();
        assert_eq!(records.len(), records2.len());
        assert_eq!(records[0].event_id, records2[0].event_id);
        assert_eq!(records[0].event_filename, records2[0].event_filename);
        assert_eq!(records[1].required_event_id, records2[1].required_event_id);
    }
}
