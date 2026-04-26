use std::io::{Read, Seek, Write};
use std::path::Path;

use byteorder::{LittleEndian, WriteBytesExt};
use encoding_rs::WINDOWS_1250;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

use crate::references::extractor::{read_mapper, read_null_terminated_windows_1250, Extractor};

// ===========================================================================
// EVENTITEM.DB FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | EventItem.db - Quest Items           |
// +--------------------------------------+
// | Encoding: Binary (Little-Endian)     |
// | Text Encoding: WINDOWS-1250          |
// | Header: 4-byte record count          |
// | Record Size: 240 bytes (60 × i32)    |
// +--------------------------------------+
// | [Header]                            |
// | - record_count: i32                  |
// +--------------------------------------+
// | [Record 1]                           |
// | - name: 30 bytes (WINDOWS-1250)     |
// | - description: 202 bytes (WINDOWS-1250)|
// | - padding: 8 bytes                   |
// +--------------------------------------+
// | [Record 2]                           |
// | ... (same structure) ...             |
// +--------------------------------------+
//
// SPECIAL VALUES:
// - Fixed-size string fields
// - Null-padded text fields
// - 8-byte padding per record
//
// FILE PURPOSE:
// Defines special quest and event items with names
// and descriptions. Used for quest progression,
// event triggering, and unique item management.
//
// ===========================================================================

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EventItem {
    /// Internal record ID representing the quest item.
    pub id: i32,
    /// Canonical lore name, translated locally.
    pub name: String,
    /// Item tooltip giving clues on application.
    pub description: String,
    /// Padding field to preserve binary compatibility.
    pub padding: [u8; 8],
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
    fn parse<R: Read + Seek>(reader: &mut R, len: u64) -> std::io::Result<Vec<Self>> {
        const COUNTER_SIZE: u8 = 4;
        const PROPERTY_ITEM_SIZE: i32 = 60 * 4;

        let elements = read_mapper(reader, len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
        let mut items: Vec<EventItem> = Vec::with_capacity(elements as usize);

        for i in 0..elements {
            let mut buffer = [0u8; 30];
            reader.read_exact(&mut buffer)?;
            let name = read_null_terminated_windows_1250(&buffer).unwrap();

            let mut buffer = [0u8; 202];
            reader.read_exact(&mut buffer)?;
            let description = read_null_terminated_windows_1250(&buffer).unwrap();

            let padding = {
                let mut buffer = [0u8; 8];
                reader.read_exact(&mut buffer)?;
                buffer
            };

            items.push(EventItem {
                id: i,
                name: name.to_string(),
                description: description.to_string(),
                padding,
            })
        }

        Ok(items)
    }

    fn to_writer<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
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

            writer.write_all(&record.padding)?;
        }

        Ok(())
    }
}

pub fn read_event_item_db(source_path: &Path) -> std::io::Result<Vec<EventItem>> {
    EventItem::read_file(source_path)
}

pub fn save_event_items(conn: &mut Connection, event_items: &[EventItem]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_event_item.sql"))?;
        for item in event_items {
            stmt.execute(params![item.id, item.name, item.description, item.padding])?;
        }
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn item_bytes(name: &str, desc: &str) -> Vec<u8> {
        let mut rec = Vec::with_capacity(240);
        let mut name_buf = [0u8; 30];
        name_buf[..name.len().min(29)].copy_from_slice(&name.as_bytes()[..name.len().min(29)]);
        rec.extend_from_slice(&name_buf);
        let mut desc_buf = [0u8; 202];
        desc_buf[..desc.len().min(201)].copy_from_slice(&desc.as_bytes()[..desc.len().min(201)]);
        rec.extend_from_slice(&desc_buf);
        rec.extend(vec![0u8; 8]); // padding
        rec
    }

    #[test]
    fn parse_single_item() {
        let mut data = 1i32.to_le_bytes().to_vec();
        data.extend(item_bytes("Scroll", "A magic scroll"));
        assert_eq!(data.len(), 244);

        let mut c = Cursor::new(&data[..]);
        let items = EventItem::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, 0);
        assert_eq!(items[0].name, "Scroll");
        assert_eq!(items[0].description, "A magic scroll");
    }

    #[test]
    fn serialize_round_trip() {
        let mut data = 1i32.to_le_bytes().to_vec();
        data.extend(item_bytes("Scroll", "A magic scroll"));
        let mut c = Cursor::new(&data[..]);
        let records = EventItem::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        EventItem::to_writer(&records, &mut out).unwrap();
        assert_eq!(out, data);
    }
}
