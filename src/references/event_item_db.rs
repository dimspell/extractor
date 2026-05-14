use std::path::Path;

use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

use crate::references::extractor::Extractor;
use dispel_macros::{Extractor, Localizable, RecordPatcher};

/// EventItem.db - Quest/Event Specific Items
///
/// Stores definitions and parameters for quest/event specific items.
///
/// Reads file: `CharacterInGame/EventItem.db`
///
/// # Binary Format
///
/// - **Encoding**: Little-endian for all numeric values
/// - **Text Encoding**: WINDOWS-1250 for `name` and `description` fields
/// - **Record Size**: 240 bytes (4 + 30 + 202 + 8)
/// - **Header**: 4-byte i32 record count, followed by records
///
/// ```text
/// +--------------------------------------+
/// | EventItem.db - Event Items         |
/// +--------------------------------------+
/// | Encoding: Binary (Little-Endian)     |
/// | Text Encoding: WINDOWS-1250           |
/// | Record Size: 240 bytes               |
/// | Header: 4-byte record count          |
/// +--------------------------------------+
/// | [Header]                             |
/// | - record_count: i32                  |
/// +--------------------------------------+
/// | [Record 1] - 240 bytes               |
/// | - id: i32 (auto-generated)           |
/// | - name: 30 bytes (WINDOWS-1250)     |
/// | - description: 202 bytes (WINDOWS...) |
/// | - padding: 8 bytes                   |
/// +--------------------------------------+
/// | [Record 2]                           |
/// | ... (same structure) ...             |
/// +--------------------------------------+
/// ```
///
/// # Field Categories
///
/// - **Identification**: `id` (auto-generated from position)
/// - **Localization**: `name` (30 bytes, WINDOWS-1250, null-padded)
/// - **Description**: `description` (202 bytes, WINDOWS-1250, null-padded)
/// - **Padding**: `padding` (8 bytes for binary compatibility)
///
/// # Special Values
///
/// - `name`: 30 bytes max, null-padded (WINDOWS-1250)
/// - `description`: 202 bytes max, null-padded (WINDOWS-1250)
/// - `padding`: Always observed as 8 zero bytes
///
/// # File Purpose
///
/// Defines quest and event-specific items with names and
/// descriptions. Used for unique items that trigger
/// events or are required for quest progression.
#[derive(
    Debug, Clone, Default, PartialEq, Serialize, Deserialize, Extractor, Localizable, RecordPatcher,
)]
#[extractor(property_item_size = 240)]
#[patcher(filename = "EventItem.db")]
pub struct EventItem {
    /// Internal record ID representing the quest item.
    #[extractor(id)]
    pub id: i32,
    /// Canonical lore name, translated locally.
    #[extractor(string(encoding = "WINDOWS-1250", size = 30))]
    #[translatable(encoding = "WINDOWS-1250", max_bytes = 30)]
    pub name: String,
    /// Item tooltip giving clues on application.
    #[extractor(string(encoding = "WINDOWS-1250", size = 202))]
    pub description: String,
    /// Padding field to preserve binary compatibility.
    #[extractor(array(size = 8, type = "u8"))]
    pub padding: [u8; 8],
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
