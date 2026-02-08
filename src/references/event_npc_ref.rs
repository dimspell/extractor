use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::Serialize;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct EventNpcRef {
    pub id: i32,
    pub event_id: i32,
    pub name: String,
}

pub fn read_event_npc_ref(source_path: &Path) -> std::io::Result<Vec<EventNpcRef>> {
    let f = File::open(source_path)?;
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(WINDOWS_1250))
            .build(f),
    );
    let mut npc_refs: Vec<EventNpcRef> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
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
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(npc_refs)
}
