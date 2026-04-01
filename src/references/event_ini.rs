use std::io::{BufRead, BufReader, Write};
use std::{fs::File, path::Path};

use crate::references::enums::EventType;
use crate::references::extractor::{parse_null, Extractor};
use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

// ===========================================================================
// EVENT.INI FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | Event.ini - Event Script Mappings    |
// +--------------------------------------+
// | Encoding: EUC-KR                     |
// | Format: CSV with comments             |
// | Record Size: Variable (text)         |
// +--------------------------------------+
// | ; Comment line                       |
// | event_id,prev_id,type,filename,counter|
// | 1,0,1,script1.scr,0                  |
// | 2,1,2,script2.scr,5                  |
// | ...                                  |
// +--------------------------------------+
//
// FIELD DEFINITIONS:
// - event_id: Unique event identifier
// - prev_id: Prerequisite event ID
// - type: Execution condition type
// - filename: Script file or "null"
// - counter: Execution limit (N times)
//
// EVENT TYPES:
// - 1: Execute once unconditionally
// - 2: Execute N times (uses counter)
// - 3: Execute if previous succeeded
// - 4: Execute on map load
// - 5: Execute on dialog trigger
//
// SPECIAL VALUES:
// - "null" literal for missing filenames
// - Lines starting with ";" are comments
// - CSV format with comma delimiter
// - counter = 0: No limit (infinite)
//
// FILE PURPOSE:
// Defines event scripts with execution conditions, prerequisites,
// and repetition limits. Used for quest progression, interactive
// objects, and game state management.
//
// ===========================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    /// Unique event identifier.
    pub event_id: i32,
    /// Prerequisite event ID that must have occurred.
    pub previous_event_id: i32,
    /// Determines execution condition (e.g. unconditionally, N times, if previous succeeded).
    pub event_type: EventType,
    /// Filename of the event script.
    pub event_filename: Option<String>,
    /// Execution counter (N limit) for types that execute multiple times.
    pub counter: i32,
}

/// Stores script and event mappings.
///
/// Reads file: `Event.ini`
/// # File Format: `Event.ini`
///
/// Text file, EUC-KR encoded. One record per line, CSV format:
/// ```text
/// event_id,previous_event_id,event_type_id,event_filename,counter
/// ```
/// - `event_filename` uses literal `null` when absent.
/// - `event_type_id` controls execution condition (see `EventType` variants).
/// - `counter` is the N-execution limit for repeating event types.
impl Extractor for Event {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let f = File::open(source_path)?;
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(EUC_KR))
                .build(f),
        );

        let mut events: Vec<Event> = Vec::new();
        for line in reader.lines() {
            if let Ok(line) = line {
                let trimmed = line.trim();
                if trimmed.starts_with(";") || trimmed.is_empty() {
                    continue;
                }

                let parts: Vec<&str> = trimmed.split(",").collect();
                if parts.len() < 5 {
                    continue;
                }
                let event_id = parts[0].parse::<i32>().unwrap();
                let previous_event_id: i32 = parts[1].parse::<i32>().unwrap();
                let event_type_id = parts[2].parse::<i32>().unwrap();
                let event_filename = parse_null(parts[3]);
                let counter = parts[4].parse::<i32>().unwrap();

                // Convert the raw event_type_id to our type-safe enum
                let event_type = EventType::from_i32(event_type_id).unwrap_or(EventType::Unknown);

                events.push(Event {
                    event_id,
                    previous_event_id,
                    event_type,
                    event_filename,
                    counter,
                });
            }
        }
        Ok(events)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(dest_path)?;
        for record in records {
            let filename = record.event_filename.as_deref().unwrap_or("null");
            let event_type_id: i32 = record.event_type.into();
            let line = format!(
                "{},{},{},{},{}\r\n",
                record.event_id, record.previous_event_id, event_type_id, filename, record.counter
            );
            let (cow, _, _) = EUC_KR.encode(&line);
            file.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_event_ini(source_path: &Path) -> std::io::Result<Vec<Event>> {
    Event::read_file(source_path)
}

pub fn save_events(conn: &mut Connection, events: &Vec<Event>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_event.sql"))?;
        for event in events {
            stmt.execute(params![
                event.event_id,
                event.previous_event_id,
                i32::from(event.event_type),
                event.event_filename,
                event.counter,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
