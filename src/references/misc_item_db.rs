use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::{fs::File, path::Path};

use crate::references::references::{read_mapper, read_null_terminated_windows_1250, Extractor};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use encoding_rs::EUC_KR;
use encoding_rs::WINDOWS_1250;
use serde::Serialize;

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

#[derive(Debug, Serialize)]
pub struct MiscItem {
    /// Numeric tracking index.
    pub id: i32,
    /// Translated string asset ID for naming the object.
    pub name: String,
    /// Tooltip string explaining use.
    pub description: String,
    /// Value retrieved when standard bartering.
    pub base_price: i32,
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
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let file = File::open(source_path)?;

        let metadata = file.metadata()?;
        let file_len = metadata.len();

        let mut reader = BufReader::new(file);

        const COUNTER_SIZE: u8 = 4;
        const PROPERTY_ITEM_SIZE: i32 = 64 * 4;

        let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
        let mut items: Vec<MiscItem> = Vec::with_capacity(elements as usize);

        for i in 0..elements {
            let mut buffer = [0u8; 30];
            reader.read_exact(&mut buffer)?;
            let name = read_null_terminated_windows_1250(&buffer)
                .unwrap_or_else(|_| "Unknown".to_string());

            let mut buffer = [0u8; 202];
            reader.read_exact(&mut buffer)?;
            let dst = EUC_KR.decode(&buffer);
            let description = dst.0.trim_end_matches("\0").trim();

            let base_price = reader.read_i32::<LittleEndian>()?;

            let mut _buffer = [0u8; 20];
            reader.read_exact(&mut _buffer)?;

            items.push(MiscItem {
                id: i,
                base_price,
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
            let (cow, _, _) = EUC_KR.encode(&record.description);
            let len = std::cmp::min(cow.len(), 202);
            desc_buf[..len].copy_from_slice(&cow[..len]);
            writer.write_all(&desc_buf)?;

            writer.write_i32::<LittleEndian>(record.base_price)?;
            writer.write_all(&[0u8; 20])?;
        }
        Ok(())
    }
}

pub fn read_misc_item_db(source_path: &Path) -> std::io::Result<Vec<MiscItem>> {
    MiscItem::read_file(source_path)
}
