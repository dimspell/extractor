use std::path::Path;

use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

use crate::references::enums::{EditItemEffect, EditItemModification};
use crate::references::extractor::Extractor;
use dispel_macros::Extractor;
use dispel_macros::Localizable;
use dispel_macros::RecordPatcher;

/// EditItem.db - Modifiable Base Items
///
/// Stores definitions for modifiable base items with stat modifications.
///
/// Reads file: `CharacterInGame/EditItem.db`
///
/// # Binary Format
///
/// - **Encoding**: Little-endian for all numeric values
/// - **Text Encoding**: WINDOWS-1250 for `name` (30 bytes) and `description` (202 bytes)
/// - **Record Size**: 268 bytes (4 + 30 + 202 + 16 × i16 + 2 × u8 + 2 padding)
/// - **Header**: 4-byte i32 record count, followed by records
///
/// ```text
/// +--------------------------------------+
/// | EditItem.db - Modifiable Items       |
/// +--------------------------------------+
/// | Encoding: Binary (Little-Endian)     |
/// | Text Encoding: WINDOWS-1250           |
/// | Record Size: 268 bytes               |
/// | Header: 4-byte record count          |
/// +--------------------------------------+
/// | [Record 1] - 268 bytes               |
/// | - index: i32 (auto-generated)        |
/// | - name: 30 bytes (WINDOWS-1250)     |
/// | - description: 202 bytes (WINDOWS...) |
/// | - base_price: i16                     |
/// | - padding1-3: i16 (unknown)         |
/// | - health_points: i16 (vitality)      |
/// | - mana_points: i16 (spell scaling)    |
/// | - strength: i16                       |
/// | - agility: i16                        |
/// | - wisdom: i16                        |
/// | - constitution: i16                   |
/// | - to_dodge: i16                      |
/// | - to_hit: i16                        |
/// | - offense: i16                       |
/// | - defense: i16                       |
/// | - magical_power: i16                  |
/// | - item_destroying_power: i16          |
/// | - padding4: u8 (unknown)            |
/// | - modifies_item: u8 (EditItemModification)|
/// | - additional_effect: i16 (EditItemEffect)|
/// +--------------------------------------+
/// | [Record 2]                           |
/// | ... (same structure) ...             |
/// +--------------------------------------+
/// ```
///
/// # Field Categories
///
/// - **Identification**: `index` (auto-generated), `name` (30 bytes), `description` (202 bytes)
/// - **Economy**: `base_price` (i16, economic valuation)
/// - **Stats**: `health_points`, `mana_points`, `strength`, `agility`, `wisdom`, `constitution`
/// - **Combat**: `to_dodge`, `to_hit`, `offense`, `defense`, `magical_power`
/// - **Durability**: `item_destroying_power` (erosion factor)
/// - **Behavior**: `modifies_item` (EditItemModification flag), `additional_effect` (EditItemEffect)
/// - **Unknown**: `padding1-4` (need investigation)
///
/// # Special Values
///
/// - `modifies_item`: Enum controlling if item mutates behavior
/// - `additional_effect`: Enum for procedural elemental modifiers (mana drain, fire, etc.)
/// - `padding1-3`: Unknown fields, observed as 0
/// - `padding4`: Unknown byte, observed as 0 or 255
///
/// # File Purpose
///
/// Defines modifiable base items with stat modifications for
/// character equipment. Used for item crafting and stat
/// enhancement systems.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Extractor, Localizable, RecordPatcher)]
#[extractor(property_item_size = 268)]
#[patcher(filename = "EditItem.db")]
pub struct EditItem {
    /// Iteration tracking for editor modifications.
    #[extractor(index)]
    pub index: i32,
    /// Asset identifier string.
    #[extractor(string(encoding = "WINDOWS-1250", size = 30))]
    #[translatable(encoding = "WINDOWS-1250", max_bytes = 30)]
    pub name: String,
    /// Standard inventory tool-tip.
    #[extractor(string(encoding = "WINDOWS-1250", size = 202))]
    #[translatable(encoding = "WINDOWS-1250", max_bytes = 202)]
    pub description: String,
    /// Economic valuation offset.
    #[extractor(primitive(type = "i16"))]
    pub base_price: i16,
    /// Unknown field.
    #[extractor(primitive(type = "i16"))]
    pub padding1: i16,
    /// Unknown field.
    #[extractor(primitive(type = "i16"))]
    pub padding2: i16,
    /// Unknown field.
    #[extractor(primitive(type = "i16"))]
    pub padding3: i16,
    /// Base additive metric for derived vitality.
    #[extractor(primitive(type = "i16"))]
    pub health_points: i16,
    /// Spell scaling base factor.
    #[extractor(primitive(type = "i16"))]
    pub mana_points: i16,
    /// Stat adjustment logic constant.
    #[extractor(primitive(type = "i16"))]
    pub strength: i16,
    /// Physical tracking parameter limit.
    #[extractor(primitive(type = "i16"))]
    pub agility: i16,
    /// Mind attribute modifier block.
    #[extractor(primitive(type = "i16"))]
    pub wisdom: i16,
    /// Core status alignment tracking.
    #[extractor(primitive(type = "i16"))]
    pub constitution: i16,
    /// Raw deflection parameter.
    #[extractor(primitive(type = "i16"))]
    pub to_dodge: i16,
    /// Base hit resolution constant.
    #[extractor(primitive(type = "i16"))]
    pub to_hit: i16,
    /// Flat output augmentation rating.
    #[extractor(primitive(type = "i16"))]
    pub offense: i16,
    /// Armor calculation pool scaling rating.
    #[extractor(primitive(type = "i16"))]
    pub defense: i16,
    /// Magical power bonus.
    #[extractor(primitive(type = "i16"))]
    pub magical_power: i16,
    /// Durability erosion factor.
    #[extractor(primitive(type = "i16"))]
    pub item_destroying_power: i16,
    /// Unknown field.
    #[extractor(primitive(type = "u8"))]
    pub padding4: u8,
    /// Flag specifying if behavior mutates.
    #[extractor(enum_from_u8(type = "EditItemModification"))]
    pub modifies_item: EditItemModification,
    /// Procedural elemental modifier appended (mana drain, fire).
    #[extractor(enum_from_i16(type = "EditItemEffect"))]
    pub additional_effect: EditItemEffect,
}

pub fn read_edit_item_db(source_path: &Path) -> std::io::Result<Vec<EditItem>> {
    EditItem::read_file(source_path)
}

pub fn save_edit_items(conn: &mut Connection, edit_items: &[EditItem]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_edit_item.sql"))?;
        for item in edit_items {
            stmt.execute(params![
                item.index,
                item.name,
                item.description,
                item.base_price,
                item.padding1,
                item.padding2,
                item.padding3,
                item.health_points,
                item.mana_points,
                item.strength,
                item.agility,
                item.wisdom,
                item.constitution,
                item.to_dodge,
                item.to_hit,
                item.offense,
                item.defense,
                item.magical_power,
                item.item_destroying_power,
                item.padding4,
                u8::from(item.modifies_item),
                i16::from(item.additional_effect),
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

    fn item_bytes(name: &str, base_price: i16, defense: i16) -> Vec<u8> {
        let mut rec = Vec::with_capacity(268);
        let mut name_buf = [0u8; 30];
        name_buf[..name.len().min(29)].copy_from_slice(&name.as_bytes()[..name.len().min(29)]);
        rec.extend_from_slice(&name_buf);
        rec.extend(vec![0u8; 202]); // description
        rec.extend_from_slice(&base_price.to_le_bytes());
        rec.extend(vec![0u8; 6]); // 3 padding i16s
        rec.extend(vec![0u8; 14]); // hp, mp, str, agi, wis, con, dodge i16s
        rec.extend_from_slice(&(0i16).to_le_bytes()); // to_hit
        rec.extend_from_slice(&(0i16).to_le_bytes()); // offense
        rec.extend_from_slice(&defense.to_le_bytes());
        rec.extend(vec![0u8; 8]); // magical_power, item_destroy, pad4+modifies, additional_effect
        rec
    }

    #[test]
    fn parse_single_item() {
        let mut data = 1i32.to_le_bytes().to_vec();
        data.extend(item_bytes("Shield", 200, 15));
        assert_eq!(data.len(), 272);

        let mut c = Cursor::new(&data[..]);
        let items = EditItem::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Shield");
        assert_eq!(items[0].base_price, 200);
        assert_eq!(items[0].defense, 15);
    }

    #[test]
    fn serialize_round_trip() {
        let mut data = 1i32.to_le_bytes().to_vec();
        data.extend(item_bytes("Shield", 200, 15));
        let mut c = Cursor::new(&data[..]);
        let records = EditItem::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        EditItem::to_writer(&records, &mut out).unwrap();
        assert_eq!(out, data);
    }
}
