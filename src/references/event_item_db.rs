use std::io::prelude::*;
use std::io::BufReader;
use std::{fs::File, path::Path};

use serde::Serialize;

use crate::references::references::{read_mapper, read_null_terminated_windows_1250};

#[derive(Debug, Serialize)]
pub struct EventItem {
    pub id: i32,
    pub name: String,
    pub description: String,
}

pub fn read_event_item_db(source_path: &Path) -> std::io::Result<Vec<EventItem>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 4;
    const PROPERTY_ITEM_SIZE: i32 = 60 * 4;
    // const FILLER: u8 = 0x0;

    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
    let mut items: Vec<EventItem> = Vec::with_capacity(elements as usize);

    for i in 0..elements {
        let mut buffer = [0u8; 30];
        reader.read_exact(&mut buffer)?;
        let name = read_null_terminated_windows_1250(&buffer).unwrap();

        let mut buffer = [0u8; 202];
        reader.read_exact(&mut buffer)?;
        let description = read_null_terminated_windows_1250(&buffer).unwrap();

        let mut buffer = [0u8; 8];
        reader.read_exact(&mut buffer)?;

        items.push(EventItem {
            id: i,
            name: name.to_string(),
            description: description.to_string(),
        })
    }

    Ok(items)
}
