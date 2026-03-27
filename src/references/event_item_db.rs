use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::{fs::File, path::Path};

use byteorder::{LittleEndian, WriteBytesExt};
use serde::Serialize;
use encoding_rs::WINDOWS_1250;

use crate::references::references::{read_mapper, read_null_terminated_windows_1250, Extractor};

#[derive(Debug, Serialize)]
pub struct EventItem {
    /// Internal record ID representing the quest item.
    pub id: i32,
    /// Canonical lore name, translated locally.
    pub name: String,
    /// Item tooltip giving clues on application.
    pub description: String,

}

/// Stores definitions and parameters for quest/event specific items.
///
/// Reads file: `CharacterInGame/EventItem.db`
/// # File Format: `CharacterInGame/EventItem.db`
///
/// Binary file, little-endian.  Starts with a 4-byte i32 record count.
/// Each record is exactly `60 × 4 = 240` bytes:
/// - `name`        : 30 bytes, null-padded, WINDOWS-1250
/// - `description` : 202 bytes, null-padded, WINDOWS-1250
/// - 8 bytes padding
impl Extractor for EventItem {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let file = File::open(source_path)?;

        let metadata = file.metadata()?;
        let file_len = metadata.len();

        let mut reader = BufReader::new(file);

        const COUNTER_SIZE: u8 = 4;
        const PROPERTY_ITEM_SIZE: i32 = 60 * 4;

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

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let file = File::create(dest_path)?;
        let mut writer = BufWriter::new(file);

        let elements = records.len() as i32;
        writer.write_i32::<LittleEndian>(elements)?;

        for record in records {
            let mut name_buf = [0u8; 30];
            let (cow, _, _) = WINDOWS_1250.encode(&record.name);
            let len = std::cmp::min(cow.len(), 30);
            name_buf[..len].copy_from_slice(&cow[..len]);
            writer.write_all(&name_buf)?;

            let mut desc_buf = [0u8; 202];
            let (cow, _, _) = WINDOWS_1250.encode(&record.description);
            let len = std::cmp::min(cow.len(), 202);
            desc_buf[..len].copy_from_slice(&cow[..len]);
            writer.write_all(&desc_buf)?;

            writer.write_all(&[0u8; 8])?;
        }

        Ok(())
    }
}

pub fn read_event_item_db(source_path: &Path) -> std::io::Result<Vec<EventItem>> {
    EventItem::read_file(source_path)
}
