use std::io::{Read, Seek, Write};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use encoding_rs::EUC_KR;
use encoding_rs::WINDOWS_1250;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

use crate::references::enums::HealItemFlag;
use crate::references::extractor::{read_mapper, read_null_terminated_windows_1250, Extractor};

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
// | - restore_full_health: u8 (full HP flag)        |
// | - restore_full_mana: u8 (full MP flag)        |
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
// - PM: Mana Points (Polish: Punkty Magii)
// - restore_full_health: Restore to full HP
// - restore_full_mana: Restore to full MP
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HealItem {
    /// Record index mapping internally.
    pub id: i32,
    /// Fixed array byte name for inventory viewing.
    pub name: String,
    /// Descriptive utility tooltip.
    pub description: String,
    /// Standardized merchant valuation.
    pub base_price: i16,
    /// Padding field.
    pub padding1: i16,
    /// Padding field.
    pub padding2: i16,
    /// Padding field.
    pub padding3: i16,
    pub health_points: i16,
    pub mana_points: i16,
    pub restore_full_health: HealItemFlag,
    pub restore_full_mana: HealItemFlag,
    pub poison_heal: HealItemFlag,
    pub petrif_heal: HealItemFlag,
    pub polimorph_heal: HealItemFlag,
    /// Padding field.
    pub padding4: u8,
    /// Padding field.
    pub padding5: i16,
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
/// - `restore_full_health`     : u8 (restore to full HP flag)
/// - `restore_full_mana`       : u8 (restore to full MP flag)
/// - `poison_heal`   : u8
/// - `petrif_heal`   : u8
/// - `polimorph_heal`: u8
/// - 1 byte + i16 padding
impl Extractor for HealItem {
    fn parse<R: Read + Seek>(reader: &mut R, len: u64) -> std::io::Result<Vec<Self>> {
        const COUNTER_SIZE: u8 = 4;
        const PROPERTY_ITEM_SIZE: i32 = 63 * 4;

        let elements = read_mapper(reader, len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
        let mut items: Vec<HealItem> = Vec::with_capacity(elements as usize);

        for i in 0..elements {
            let mut buffer = [0u8; 30];
            reader.read_exact(&mut buffer)?;
            let name = read_null_terminated_windows_1250(&buffer).unwrap();

            let mut buffer = [0u8; 202];
            reader.read_exact(&mut buffer)?;
            let description = read_null_terminated_windows_1250(&buffer).unwrap();
            let description = description.trim();

            let base_price = reader.read_i16::<LittleEndian>()?;

            let padding1 = reader.read_i16::<LittleEndian>()?;
            let padding2 = reader.read_i16::<LittleEndian>()?;
            let padding3 = reader.read_i16::<LittleEndian>()?;

            let health_points = reader.read_i16::<LittleEndian>()?;
            let mana_points = reader.read_i16::<LittleEndian>()?;
            let restore_full_health_raw = reader.read_u8()?;
            let restore_full_mana_raw = reader.read_u8()?;
            let poison_heal_raw = reader.read_u8()?;
            let petrif_heal_raw = reader.read_u8()?;
            let polimorph_heal_raw = reader.read_u8()?;

            let padding4 = reader.read_u8()?;
            let padding5 = reader.read_i16::<LittleEndian>()?;

            let restore_full_health =
                HealItemFlag::from_u8(restore_full_health_raw).unwrap_or(HealItemFlag::None);
            let restore_full_mana =
                HealItemFlag::from_u8(restore_full_mana_raw).unwrap_or(HealItemFlag::None);
            let poison_heal = HealItemFlag::from_u8(poison_heal_raw).unwrap_or(HealItemFlag::None);
            let petrif_heal = HealItemFlag::from_u8(petrif_heal_raw).unwrap_or(HealItemFlag::None);
            let polimorph_heal =
                HealItemFlag::from_u8(polimorph_heal_raw).unwrap_or(HealItemFlag::None);

            items.push(HealItem {
                id: i,
                name: name.to_string(),
                description: description.to_string(),
                base_price,
                padding1,
                padding2,
                padding3,
                health_points,
                mana_points,
                restore_full_health,
                restore_full_mana,
                poison_heal,
                petrif_heal,
                polimorph_heal,
                padding4,
                padding5,
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
            let (cow, _, _) = EUC_KR.encode(&record.description);
            let len = std::cmp::min(cow.len(), 202);
            desc_buf[..len].copy_from_slice(&cow[..len]);
            writer.write_all(&desc_buf)?;

            writer.write_i16::<LittleEndian>(record.base_price)?;

            writer.write_i16::<LittleEndian>(record.padding1)?;
            writer.write_i16::<LittleEndian>(record.padding2)?;
            writer.write_i16::<LittleEndian>(record.padding3)?;

            writer.write_i16::<LittleEndian>(record.health_points)?;
            writer.write_i16::<LittleEndian>(record.mana_points)?;

            writer.write_u8(u8::from(record.restore_full_health))?;
            writer.write_u8(u8::from(record.restore_full_mana))?;
            writer.write_u8(u8::from(record.poison_heal))?;
            writer.write_u8(u8::from(record.petrif_heal))?;
            writer.write_u8(u8::from(record.polimorph_heal))?;

            writer.write_u8(record.padding4)?;
            writer.write_i16::<LittleEndian>(record.padding5)?;
        }
        Ok(())
    }
}

pub fn read_heal_item_db(source_path: &Path) -> std::io::Result<Vec<HealItem>> {
    HealItem::read_file(source_path)
}

pub fn save_heal_items(conn: &mut Connection, heal_items: &[HealItem]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_heal_item.sql"))?;
        for item in heal_items {
            stmt.execute(params![
                item.id,
                item.name,
                item.description,
                item.base_price,
                item.padding1,
                item.padding2,
                item.padding3,
                item.health_points,
                item.mana_points,
                u8::from(item.restore_full_health),
                u8::from(item.restore_full_mana),
                u8::from(item.poison_heal),
                u8::from(item.petrif_heal),
                u8::from(item.polimorph_heal),
                item.padding4,
                item.padding5,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

impl std::fmt::Display for HealItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HealItem({} - {})", self.id, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn item_bytes(name: &str, base_price: i16, health_points: i16) -> Vec<u8> {
        let mut rec = Vec::with_capacity(252);
        let mut name_buf = [0u8; 30];
        name_buf[..name.len().min(29)].copy_from_slice(&name.as_bytes()[..name.len().min(29)]);
        rec.extend_from_slice(&name_buf);
        rec.extend(vec![0u8; 202]); // description
        rec.extend_from_slice(&base_price.to_le_bytes());
        rec.extend(vec![0u8; 6]); // 3 padding i16s
        rec.extend_from_slice(&health_points.to_le_bytes());
        rec.extend(vec![0u8; 10]); // mana_points + 5 u8 flags + 1 pad u8 + pad i16
        rec
    }

    #[test]
    fn parse_single_item() {
        let mut data = 1i32.to_le_bytes().to_vec();
        data.extend(item_bytes("Potion", 50, 100));
        assert_eq!(data.len(), 256);

        let mut c = Cursor::new(&data[..]);
        let items = HealItem::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Potion");
        assert_eq!(items[0].base_price, 50);
        assert_eq!(items[0].health_points, 100);
    }

    #[test]
    fn serialize_round_trip() {
        let mut data = 1i32.to_le_bytes().to_vec();
        data.extend(item_bytes("Potion", 50, 100));
        let mut c = Cursor::new(&data[..]);
        let records = HealItem::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        HealItem::to_writer(&records, &mut out).unwrap();
        assert_eq!(out, data);
    }
}
