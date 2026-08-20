use std::path::Path;

use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

use crate::references::enums::HealItemFlag;
use crate::references::extractor::Extractor;
use dispel_macros::{Extractor, Localizable, RecordPatcher};

/// HealItem.db - Consumable Healing Items
///
/// Stores definitions, stats, and prices for consumable healing items.
///
/// Reads file: `CharacterInGame/HealItem.db`
///
/// # Binary Format
///
/// - **Encoding**: Little-endian for all numeric values
/// - **Text Encoding**: WINDOWS-1250 for `name` and EUC-KR for `description`
/// - **Record Size**: 252 bytes
/// - **Header**: 4-byte record count
///
/// ```text
/// +--------------------------------------+
/// | HealItem.db - Healing Items          |
/// +--------------------------------------+
/// | Encoding: Binary (Little-Endian)     |
/// | Text: WINDOWS-1250 / EUC-KR          |
/// | Record Size: 252 bytes               |
/// | Header: i32 record count             |
/// +--------------------------------------+
/// | [Record 1] - 252 bytes               |
/// | - id: i32 (auto-generated)           |
/// | - name: 30 bytes (WINDOWS-1250)    |
/// | - description: 202 bytes (EUC-KR)   |
/// | - base_price: i32                    |
/// | - runtime_item_index_slot: i32       |
/// | - health_points: i16 (healing amount) |
/// | - mana_points: i16 (mana restore)     |
/// | - restores_full_health: u8           |
/// | - restores_full_mana: u8             |
/// | - cures_poison: u8                   |
/// | - cures_petrification: u8            |
/// | - cures_polymorph: u8                |
/// | - reserved_trailer: 3 bytes          |
/// +--------------------------------------+
/// | [Record 2]                           |
/// | ... (same structure) ...             |
/// +--------------------------------------+
/// ```
///
/// # Field Categories
///
/// - **Identification**: `id` (auto-generated), `name` (30 bytes, WINDOWS-1250), `description` (202 bytes, EUC-KR)
/// - **Economy**: `base_price` (i32, merchant valuation)
/// - **Healing**: `health_points` (HP restore), `mana_points` (MP restore)
/// - **Full Restore Flags**: `restores_full_health`, `restores_full_mana` (HealItemFlag)
/// - **Cure Effects**: `cures_poison`, `cures_petrification`, `cures_polymorph` (HealItemFlag)
/// - **Runtime bookkeeping**: `runtime_item_index_slot` is overwritten with the
///   record index while loading.
/// - **Reserved data**: `reserved_trailer` is retained verbatim.
///
/// # Special Values
///
/// - `restores_full_health`: HealItemFlag::Active = restore full HP
/// - `restores_full_mana`: HealItemFlag::Active = restore full MP
/// - `cures_poison`: HealItemFlag::Active = cure poison
/// - `cures_petrification`: HealItemFlag::Active = cure petrification
/// - `cures_polymorph`: HealItemFlag::Active = cure polymorph
/// - `runtime_item_index_slot`: Overwritten with the sequential record index.
/// - `reserved_trailer`: Three bytes, zero in the bundled fixture.
///
/// # File Purpose
///
/// Defines consumable healing items with restoration
/// amounts and cure capabilities. Used for potions,
/// scrolls, and other consumable healing items.
#[derive(
    Debug, Clone, Default, PartialEq, Serialize, Deserialize, Extractor, Localizable, RecordPatcher,
)]
#[extractor(property_item_size = 252)]
#[patcher(filename = "HealItem.db")]
pub struct HealItem {
    /// Record index mapping internally.
    #[extractor(id)]
    pub id: i32,
    /// Fixed array byte name for inventory viewing.
    #[extractor(string(encoding = "WINDOWS-1250", size = 30))]
    #[translatable(encoding = "WINDOWS-1250", max_bytes = 30)]
    pub name: String,
    /// Descriptive utility tooltip.
    #[extractor(string(encoding = "EUC-KR", size = 202))]
    #[translatable(encoding = "WINDOWS-1250", max_bytes = 202)]
    pub description: String,
    /// Standardized merchant valuation.
    #[extractor(primitive(type = "i32"))]
    pub base_price: i32,
    /// On-disk slot replaced with the sequential item index during loading.
    ///
    /// The game writes the record index to offset `0xEC` after reading each
    /// 252-byte record. Preserve the disk value, but do not treat it as an
    /// item property.
    #[extractor(primitive(type = "i32"))]
    pub runtime_item_index_slot: i32,
    #[extractor(primitive(type = "i16"))]
    pub health_points: i16,
    #[extractor(primitive(type = "i16"))]
    pub mana_points: i16,
    #[extractor(enum_from_u8(type = "HealItemFlag"))]
    pub restores_full_health: HealItemFlag,
    #[extractor(enum_from_u8(type = "HealItemFlag"))]
    pub restores_full_mana: HealItemFlag,
    #[extractor(enum_from_u8(type = "HealItemFlag"))]
    pub cures_poison: HealItemFlag,
    #[extractor(enum_from_u8(type = "HealItemFlag"))]
    pub cures_petrification: HealItemFlag,
    #[extractor(enum_from_u8(type = "HealItemFlag"))]
    pub cures_polymorph: HealItemFlag,
    /// Reserved bytes at offsets `0xF9..0xFB`.
    ///
    /// No direct use was found in the executable. Preserve these bytes
    /// verbatim; the bundled fixture stores zero in all three positions.
    #[extractor(vec_u8(size = 3))]
    pub reserved_trailer: Vec<u8>,
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
                item.runtime_item_index_slot,
                item.health_points,
                item.mana_points,
                u8::from(item.restores_full_health),
                u8::from(item.restores_full_mana),
                u8::from(item.cures_poison),
                u8::from(item.cures_petrification),
                u8::from(item.cures_polymorph),
                item.reserved_trailer,
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

    fn item_bytes(name: &str, base_price: i32, health_points: i16) -> Vec<u8> {
        let mut rec = Vec::with_capacity(252);
        let mut name_buf = [0u8; 30];
        name_buf[..name.len().min(29)].copy_from_slice(&name.as_bytes()[..name.len().min(29)]);
        rec.extend_from_slice(&name_buf);
        rec.extend(vec![0u8; 202]); // description
        rec.extend_from_slice(&base_price.to_le_bytes());
        rec.extend_from_slice(&0x1234_5678i32.to_le_bytes()); // runtime index slot
        rec.extend_from_slice(&health_points.to_le_bytes());
        rec.extend(vec![0u8; 7]); // mana_points + 5 effect flags
        rec.extend_from_slice(&[0xA5; 3]); // reserved trailer
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
        assert_eq!(items[0].runtime_item_index_slot, 0x1234_5678);
        assert_eq!(items[0].reserved_trailer, vec![0xA5; 3]);
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
