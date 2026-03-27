use std::{fs::File, path::Path};
use std::io::{BufRead, BufReader, Write};

use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};
use crate::references::references::{parse_null, Extractor};

enum EventType {
    // type (Translation from Korean)
    // 0 unconditionally executes once (ignores previous event)
    // 1 unconditionally executes N times (ignores previous event)
    // 2 unconditionally executed (ignores previous event)
    // 3 executed once when previous event failed
    // 4 before event. Execute N times when condition is true
    // 5 execute event when previous event is successful
    // 6 execute once when previous event is successful
    // 7 execute N times, when previous event is successful
    // 8 continues event, when previous event is successful
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    /// Unique event identifier.
    pub event_id: i32,
    /// Prerequisite event ID that must have occurred.
    pub previous_event_id: i32,
    /// Determines execution condition (e.g. unconditionally, N times, if previous succeeded).
    pub event_type_id: i32,
    /// Filename of the event script.
    pub event_filename: Option<String>,
    /// Execution counter (N limit) for types that execute multiple times.
    pub counter: i32, // N counter

}

/// Stores script and event mappings.
///
/// Reads file: `Event.ini`
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
            match line {
                Ok(line) => {
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

                    events.push(Event {
                        event_id,
                        previous_event_id,
                        event_type_id,
                        event_filename,
                        counter,
                    });
                }
                _ => {}
            }
        }
        Ok(events)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(dest_path)?;
        for record in records {
            let filename = record.event_filename.as_deref().unwrap_or("null");
            let line = format!(
                "{},{},{},{},{}\r\n",
                record.event_id, record.previous_event_id, record.event_type_id, filename, record.counter
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