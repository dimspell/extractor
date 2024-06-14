use std::{fs::File, path::Path};
use std::io::{BufRead, BufReader};

use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};
use crate::references::references::parse_null;

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
    pub event_id: i32,
    pub previous_event_id: i32,
    pub event_type_id: i32,
    pub event_filename: Option<String>,
    pub counter: i32, // N counter
}

pub fn read_event_ini(source_path: &Path) -> std::io::Result<Vec<Event>> {
    let f = File::open(source_path)?;
    // let mut reader = BufReader::new(f);
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(EUC_KR))
            .build(f),
    );

    let mut events: Vec<Event> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(";") {
                    continue;
                }

                let parts: Vec<&str> = line.split(",").collect();
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
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(events)
}