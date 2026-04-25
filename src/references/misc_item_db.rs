use std::io::prelude::*;
use std::io::{BufWriter, Read, Seek};
use std::{fs::File, path::Path};

use crate::references::extractor::{read_mapper, read_null_terminated_windows_1250, Extractor};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use encoding_rs::EUC_KR;
use encoding_rs::WINDOWS_1250;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

// ===========================================================================
// MISCITEM.DB FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | MiscItem.db - Generic Items           |
// +--------------------------------------+
// | Encoding: Binary (Little-Endian)     |
// | Text Encodings: Mixed                |
// | Header: 4-byte record count           |
// | Record Size: 256 bytes (64 × i32)     |
// +--------------------------------------+
// | [Header]                            |
// | - record_count: i32                  |
// +--------------------------------------+
// | [Record 1]                           |
// | - name: 30 bytes (WINDOWS-1250)      |
// | - description: 202 bytes (EUC-KR)    |
// | - base_price: i32                    |
// | - padding: 20 bytes                 |
// +--------------------------------------+
// | [Record 2]                           |
// | ... (same structure) ...             |
// +--------------------------------------+
//
// SPECIAL VALUES:
// - base_price = 0: Non-tradable items
// - base_price = -1: Quest items
// - Fixed-size string fields
// - Mixed text encodings
//
// FILE PURPOSE:
// Defines generic miscellaneous items with names,
// descriptions, and economic values. Used for crafting
// materials, consumables, and various utility items.
//
// ===========================================================================

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MiscItem {
    /// Numeric tracking index.
    pub id: i32,
    /// Translated string asset ID for naming the object.
    pub name: String,
    /// Tooltip string explaining use.
    pub description: String,
    /// Value retrieved when standard bartering.
    pub base_price: i32,
    /// Padding field to preserve binary compatibility.
    pub padding: [u8; 20],
}

/// Stores definitions, stats, and prices for generic miscellaneous items.
///
/// Reads file: `CharacterInGame/MiscItem.db`
/// # File Format: `CharacterInGame/MiscItem.db`
///
/// Binary file, little-endian.  Starts with a 4-byte i32 record count.
/// Each record is exactly `64 × 4 = 256` bytes:
/// - `name`        : 30 bytes, null-padded, WINDOWS-1250
/// - `description` : 202 bytes, null-padded, EUC-KR
/// - `base_price`  : i32
/// - 20 bytes padding
impl Extractor for MiscItem {
    fn parse<R: Read + Seek>(reader: &mut R, len: u64) -> std::io::Result<Vec<Self>> {
        const COUNTER_SIZE: u8 = 4;
        const PROPERTY_ITEM_SIZE: i32 = 64 * 4;

        let elements = read_mapper(reader, len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
        let mut items: Vec<MiscItem> = Vec::with_capacity(elements as usize);

        for i in 0..elements {
            let mut buffer = [0u8; 30];
            reader.read_exact(&mut buffer)?;
            let name = read_null_terminated_windows_1250(&buffer)
                .unwrap_or_else(|_| "Unknown".to_string());

            let mut buffer = [0u8; 202];
            reader.read_exact(&mut buffer)?;
            let dst = EUC_KR.decode(&buffer);
            let stripped: String = dst.0.chars().filter(|&c| c != '\0').collect();
            let description = stripped.trim();

            let base_price = reader.read_i32::<LittleEndian>()?;

            let padding = {
                let mut buffer = [0u8; 20];
                reader.read_exact(&mut buffer)?;
                buffer
            };

            // let string: String = padding.;
            // println!("{:?}, {}", padding);

            items.push(MiscItem {
                id: i,
                base_price,
                name: name.to_string(),
                description: description.to_string(),
                padding,
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
            let (cow, _, _) = EUC_KR.encode(&record.description);
            let len = std::cmp::min(cow.len(), 202);
            desc_buf[..len].copy_from_slice(&cow[..len]);
            writer.write_all(&desc_buf)?;

            writer.write_i32::<LittleEndian>(record.base_price)?;
            writer.write_all(&record.padding)?;
        }
        Ok(())
    }
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
}
