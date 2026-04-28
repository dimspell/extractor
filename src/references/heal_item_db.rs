use std::path::Path;

use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

use crate::references::enums::HealItemFlag;
use crate::references::extractor::Extractor;
use dispel_macros::{Extractor, Localizable};

/// HealItem.db - Consumable Healing Items
///
/// Stores definitions, stats, and prices for consumable healing items.
///
/// Reads file: `CharacterInGame/HealItem.db`
///
/// # Binary Format
///
/// - **Encoding**: Little-endian for all numeric values
/// - **Text Encoding**: WINDOWS-1250 for `name` (30 bytes) and `description` (202 bytes)
/// - **Record Size**: 252 bytes (4 + 30 + 202 + 16 × i16)
/// - **Header**: None; parse until EOF
///
/// ```text
/// +--------------------------------------+
/// | HealItem.db - Healing Items        |
/// +--------------------------------------+
/// | Encoding: Binary (Little-Endian)     |
/// | Text Encoding: WINDOWS-1250           |
/// | Record Size: 252 bytes               |
/// | Header: None (parse until EOF)       |
/// +--------------------------------------+
/// | [Record 1] - 252 bytes               |
/// | - id: i32 (auto-generated)           |
/// | - name: 30 bytes (WINDOWS-1250)    |
/// | - description: 202 bytes (EUC-KR)   |
/// | - base_price: i16                    |
/// | - padding1-3: i16 (unknown)         |
/// | - health_points: i16 (healing amount) |
/// | - mana_points: i16 (mana restore)     |
/// | - restore_full_health: u8 (HealItemFlag)|
/// | - restore_full_mana: u8 (HealItemFlag)|
/// | - poison_heal: u8 (HealItemFlag)    |
/// | - petrif_heal: u8 (HealItemFlag)    |
/// | - polimorph_heal: u8 (HealItemFlag) |
/// | - padding4: u8 (unknown)            |
/// | - padding5: i16 (unknown)           |
/// +--------------------------------------+
/// | [Record 2]                           |
/// | ... (same structure) ...             |
/// +--------------------------------------+
/// ```
///
/// # Field Categories
///
/// - **Identification**: `id` (auto-generated), `name` (30 bytes, WINDOWS-1250), `description` (202 bytes, EUC-KR)
/// - **Economy**: `base_price` (i16, merchant valuation)
/// - **Healing**: `health_points` (HP restore), `mana_points` (MP restore)
/// - **Full Restore Flags**: `restore_full_health`, `restore_full_mana` (HealItemFlag)
/// - **Cure Effects**: `poison_heal`, `petrif_heal`, `polimorph_heal` (HealItemFlag)
/// - **Unknown**: `padding1-5` (need investigation)
///
/// # Special Values
///
/// - `restore_full_health`: HealItemFlag::Enabled = restore full HP
/// - `restore_full_mana`: HealItemFlag::Enabled = restore full MP
/// - `poison_heal`: HealItemFlag::Enabled = cure poison
/// - `petrif_heal`: HealItemFlag::Enabled = cure petrification
/// - `polimorph_heal`: HealItemFlag::Enabled = cure polymorph
/// - `padding1-3`: Unknown i16 fields, observed as 0
/// - `padding4`: Unknown u8, observed as 0 or 255
/// - `padding5`: Unknown i16, observed as 0
///
/// # File Purpose
///
/// Defines consumable healing items with restoration
/// amounts and cure capabilities. Used for potions,
/// scrolls, and other consumable healing items.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Extractor, Localizable)]
#[extractor(property_item_size = 252)]
pub struct HealItem {
    /// Record index mapping internally.
    #[extractor(id)]
    pub id: i32,
    /// Fixed array byte name for inventory viewing.
    #[extractor(string(encoding = "WINDOWS-1250", size = 30))]
    pub name: String,
    /// Descriptive utility tooltip.
    #[extractor(string(encoding = "EUC-KR", size = 202))]
    #[translatable(encoding = "WINDOWS-1250", max_bytes = 202)]
    pub description: String,
    /// Standardized merchant valuation.
    #[extractor(primitive(type = "i16"))]
    pub base_price: i16,
    /// Padding field.
    #[extractor(primitive(type = "i16"))]
    pub padding1: i16,
    /// Padding field.
    #[extractor(primitive(type = "i16"))]
    pub padding2: i16,
    /// Padding field.
    #[extractor(primitive(type = "i16"))]
    pub padding3: i16,
    #[extractor(primitive(type = "i16"))]
    pub health_points: i16,
    #[extractor(primitive(type = "i16"))]
    pub mana_points: i16,
    #[extractor(enum_from_u8(type = "HealItemFlag"))]
    pub restore_full_health: HealItemFlag,
    #[extractor(enum_from_u8(type = "HealItemFlag"))]
    pub restore_full_mana: HealItemFlag,
    #[extractor(enum_from_u8(type = "HealItemFlag"))]
    pub poison_heal: HealItemFlag,
    #[extractor(enum_from_u8(type = "HealItemFlag"))]
    pub petrif_heal: HealItemFlag,
    #[extractor(enum_from_u8(type = "HealItemFlag"))]
    pub polimorph_heal: HealItemFlag,
    /// Padding field.
    #[extractor(padding(count = 1, type = "u8", default_value = "0"))]
    pub padding4: u8,
    /// Padding field.
    #[extractor(padding(count = 1, type = "i16", default_value = "0"))]
    pub padding5: i16,
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
