use std::path::Path;

use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

use crate::references::extractor::Extractor;
use dispel_macros::{Extractor, Localizable, RecordPatcher};

/// MiscItem.db - Miscellaneous Items
///
/// Stores definitions, stats, and prices for generic miscellaneous items.
///
/// Reads file: `CharacterInGame/MiscItem.db`
///
/// # Binary Format
///
/// - **Encoding**: Little-endian for all numeric values
/// - **Text Encoding**: WINDOWS-1250 for `name` (30 bytes) and `description` (202 bytes)
/// - **Record Size**: 256 bytes (4 + 30 + 202 + 20)
/// - **Header**: 4-byte i32 record count, followed by records
///
/// ```text
/// +--------------------------------------+
/// | MiscItem.db - Misc Items          |
/// +--------------------------------------+
/// | Encoding: Binary (Little-Endian)     |
/// | Text Encoding: WINDOWS-1250           |
/// | Record Size: 256 bytes               |
/// | Header: 4-byte record count          |
/// +--------------------------------------+
/// | [Header]                             |
/// | - record_count: i32                  |
/// +--------------------------------------+
/// | [Record 1] - 256 bytes               |
/// | - id: i32 (auto-generated)           |
/// | - name: 30 bytes (WINDOWS-1250)    |
/// | - description: 202 bytes (WINDOWS...) |
/// | - base_price: i32                    |
/// | - padding: 20 bytes (binary compat)  |
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
/// - **Economy**: `base_price` (i32, economic valuation)
/// - **Padding**: `padding` (20 bytes for binary compatibility)
///
/// # Special Values
///
/// - `name`: 30 bytes max, null-padded (WINDOWS-1250)
/// - `description`: 202 bytes max, null-padded (WINDOWS-1250)
/// - `padding`: Always observed as 20 zero bytes
///
/// # File Purpose
///
/// Defines miscellaneous items with names, descriptions,
/// and prices. Used for consumables, quest items,
/// and generic inventory objects.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Extractor, Localizable, RecordPatcher)]
#[extractor(property_item_size = 256)]
#[patcher(filename = "MiscItem.db")]
pub struct MiscItem {
    /// Numeric tracking index.
    #[extractor(id)]
    pub id: i32,
    /// Translated string asset ID for naming the object.
    #[extractor(string(encoding = "WINDOWS-1250", size = 30))]
    #[translatable(encoding = "WINDOWS-1250", max_bytes = 30)]
    pub name: String,
    /// Tooltip string explaining use.
    #[extractor(string(encoding = "EUC-KR", size = 202))]
    #[translatable(encoding = "EUC-KR", max_bytes = 202)]
    pub description: String,
    /// Value retrieved when standard bartering.
    #[extractor(primitive(type = "i32"))]
    pub base_price: i32,
    /// Padding field to preserve binary compatibility.
    #[extractor(array(size = 20, type = "u8"))]
    pub padding: [u8; 20],
}

pub fn read_misc_item_db(source_path: &Path) -> std::io::Result<Vec<MiscItem>> {
    MiscItem::read_file(source_path)
}

pub fn save_misc_items(conn: &mut Connection, misc_items: &[MiscItem]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_misc_item.sql"))?;
        for item in misc_items {
            stmt.execute(params![
                item.id,
                item.name,
                item.description,
                item.base_price,
                item.padding
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn item_bytes(name: &str, base_price: i32) -> Vec<u8> {
        let mut rec = Vec::with_capacity(256);
        let mut name_buf = [0u8; 30];
        name_buf[..name.len().min(29)].copy_from_slice(&name.as_bytes()[..name.len().min(29)]);
        rec.extend_from_slice(&name_buf);
        rec.extend(vec![0u8; 202]); // description (zeroed = empty)
        rec.extend_from_slice(&base_price.to_le_bytes());
        rec.extend(vec![0u8; 20]); // padding
        rec
    }

    #[test]
    fn parse_single_item() {
        let mut data = 1i32.to_le_bytes().to_vec();
        data.extend(item_bytes("Torch", 15));
        assert_eq!(data.len(), 260);

        let mut c = Cursor::new(&data[..]);
        let items = MiscItem::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Torch");
        assert_eq!(items[0].base_price, 15);
    }

    #[test]
    fn serialize_round_trip() {
        let mut data = 1i32.to_le_bytes().to_vec();
        data.extend(item_bytes("Torch", 15));
        let mut c = Cursor::new(&data[..]);
        let records = MiscItem::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        MiscItem::to_writer(&records, &mut out).unwrap();
        assert_eq!(out, data);
    }
}
