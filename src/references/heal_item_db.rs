use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::{fs::File, path::Path};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use encoding_rs::EUC_KR;
use serde::Serialize;
use encoding_rs::WINDOWS_1250;

use crate::references::references::{read_mapper, read_null_terminated_windows_1250, Extractor};
use crate::references::enums::HealItemFlag;

// ===========================================================================
// HEALITEM.DB FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | HealItem.db - Consumable Items       |
// +--------------------------------------+
// | Encoding: Binary (Little-Endian)     |
// | Text Encodings: Mixed                |
// | Header: 4-byte record count          |
// | Record Size: 252 bytes (63 × i32)    |
// +--------------------------------------+
// | [Header]                            |
// | - record_count: i32                  |
// +--------------------------------------+
// | [Record 1]                           |
// | - name: 30 bytes (WINDOWS-1250)       |
// | - description: 202 bytes (EUC-KR)    |
// | - base_price: i16                    |
// | - padding: i16 × 3                   |
// | - pz: i16 (HP restore)               |
// | - pm: i16 (MP restore)               |
// | - full_pz: u8 (full HP flag)        |
// | - full_pm: u8 (full MP flag)        |
// | - poison_heal: u8                    |
// | - petrif_heal: u8                    |
// | - polimorph_heal: u8                 |
// | - padding: u8 + i16                  |
// +--------------------------------------+
// | [Record 2]                           |
// | ... (same structure) ...             |
// +--------------------------------------+
//
// FIELD ABBREVIATIONS:
// - PZ: Health Points (Polish: Punkty Zdrowia)
// - PM: Magic Points (Polish: Punkty Magii)
// - full_pz: Restore to full HP
// - full_pm: Restore to full MP
//
// HEALING FLAGS:
// - poison_heal: Cures poison status
// - petrif_heal: Cures petrification
// - polimorph_heal: Cures polymorph
//
// SPECIAL VALUES:
// - pz/pm: Negative values for damage items
// - base_price: 0 for non-tradable items
// - Flags: 0=none, 1=active
//
// FILE PURPOSE:
// Defines all consumable healing items with restoration effects,
// status cures, and economic values. Used for inventory management,
// combat healing, and status effect systems.
//
// ===========================================================================

#[derive(Debug, Serialize)]
pub struct HealItem {
    /// Record index mapping internally.
    pub id: i32,
    /// Fixed array byte name for inventory viewing.
    pub name: String,
    /// Descriptive utility tooltip.
    pub description: String,
    /// Standardized merchant valuation.
    pub base_price: i16,
    pub pz: i16,
    pub pm: i16,
    pub full_pz: HealItemFlag,
    pub full_pm: HealItemFlag,
    pub poison_heal: HealItemFlag,
    pub petrif_heal: HealItemFlag,
    pub polimorph_heal: HealItemFlag,

}

/// Stores definitions, stats, and prices for consumable healing items.
///
/// Reads file: `CharacterInGame/HealItem.db`
/// # File Format: `CharacterInGame/HealItem.db`
///
/// Binary file, little-endian.  Starts with a 4-byte i32 record count.
/// Each record is exactly `63 × 4 = 252` bytes:
/// - `name`          : 30 bytes, null-padded, WINDOWS-1250
/// - `description`   : 202 bytes, null-padded, EUC-KR
/// - `base_price`    : i16
/// - 3 × i16 padding
/// - `pz` / `pm`     : i16 (HP/MP restore amount)
/// - `full_pz`       : u8 (restore to full HP flag)
/// - `full_pm`       : u8 (restore to full MP flag)
/// - `poison_heal`   : u8
/// - `petrif_heal`   : u8
/// - `polimorph_heal`: u8
/// - 1 byte + i16 padding
impl Extractor for HealItem {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let file = File::open(source_path)?;

        let metadata = file.metadata()?;
        let file_len = metadata.len();

        let mut reader = BufReader::new(file);

        const COUNTER_SIZE: u8 = 4;
        const PROPERTY_ITEM_SIZE: i32 = 63 * 4;

        let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
        let mut items: Vec<HealItem> = Vec::with_capacity(elements as usize);

        for i in 0..elements {
            let mut buffer = [0u8; 30];
            reader.read_exact(&mut buffer)?;
            let name = read_null_terminated_windows_1250(&buffer).unwrap();

            let mut buffer = [0u8; 202];
            reader.read_exact(&mut buffer)?;
            let dst = EUC_KR.decode(&buffer);
            let description = dst.0.trim_end_matches("\0").trim();

            let base_price = reader.read_i16::<LittleEndian>()?;

            reader.read_i16::<LittleEndian>()?;
            reader.read_i16::<LittleEndian>()?;
            reader.read_i16::<LittleEndian>()?;

            let pz = reader.read_i16::<LittleEndian>()?;
            let pm = reader.read_i16::<LittleEndian>()?;
            let full_pz_raw = reader.read_u8()?;
            let full_pm_raw = reader.read_u8()?;
            let poison_heal_raw = reader.read_u8()?;
            let petrif_heal_raw = reader.read_u8()?;
            let polimorph_heal_raw = reader.read_u8()?;

            reader.read_u8()?;
            reader.read_i16::<LittleEndian>()?;

            let full_pz = HealItemFlag::from_u8(full_pz_raw).unwrap_or(HealItemFlag::None);
            let full_pm = HealItemFlag::from_u8(full_pm_raw).unwrap_or(HealItemFlag::None);
            let poison_heal = HealItemFlag::from_u8(poison_heal_raw).unwrap_or(HealItemFlag::None);
            let petrif_heal = HealItemFlag::from_u8(petrif_heal_raw).unwrap_or(HealItemFlag::None);
            let polimorph_heal = HealItemFlag::from_u8(polimorph_heal_raw).unwrap_or(HealItemFlag::None);

            items.push(HealItem {
                id: i,
                name: name.to_string(),
                description: description.to_string(),
                base_price,
                pz,
                pm,
                full_pz,
                full_pm,
                poison_heal,
                petrif_heal,
                polimorph_heal,
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

            writer.write_i16::<LittleEndian>(record.base_price)?;

            writer.write_i16::<LittleEndian>(0)?;
            writer.write_i16::<LittleEndian>(0)?;
            writer.write_i16::<LittleEndian>(0)?;

            writer.write_i16::<LittleEndian>(record.pz)?;
            writer.write_i16::<LittleEndian>(record.pm)?;

            writer.write_u8(u8::from(record.full_pz))?;
            writer.write_u8(u8::from(record.full_pm))?;
            writer.write_u8(u8::from(record.poison_heal))?;
            writer.write_u8(u8::from(record.petrif_heal))?;
            writer.write_u8(u8::from(record.polimorph_heal))?;

            writer.write_u8(0)?;
            writer.write_i16::<LittleEndian>(0)?;
        }
        Ok(())
    }
}

pub fn read_heal_item_db(source_path: &Path) -> std::io::Result<Vec<HealItem>> {
    HealItem::read_file(source_path)
}
