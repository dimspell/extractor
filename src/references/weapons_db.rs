use std::path::Path;

use crate::references::extractor::Extractor;
use dispel_macros::{Extractor, Localizable, RecordPatcher};
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

/// WeaponItem.db - Weapons & Armor
///
/// Stores stats, prices, and requirements for weapons and armor.
///
/// Reads file: `CharacterInGame/weaponItem.db`
///
/// # Binary Format
///
/// - **Encoding**: Little-endian for all numeric values
/// - **Text Encoding**: WINDOWS-1250 for `name` (30 bytes) and `description` (202 bytes)
/// - **Header**: 4-byte little-endian record count
/// - **Record Size**: 284 bytes per record
///
/// ```text
/// +--------------------------------------+
/// | WeaponItem.db - Weapons & Armor   |
/// +--------------------------------------+
/// | Encoding: Binary (Little-Endian)     |
/// | Text Encoding: WINDOWS-1250           |
/// | Record Size: 284 bytes               |
/// | Header: i32 record count              |
/// +--------------------------------------+
/// | [Record 1] - 284 bytes               |
/// | - id: i32 (auto-generated)           |
/// | - name: 30 bytes (WINDOWS-1250)     |
/// | - description: 202 bytes (WINDOWS...) |
/// | - base_price: i32                    |
/// | - weapon_item_id: i32 (runtime ID)   |
/// | - health_points: i16 (HP mod)        |
/// | - mana_points: i16 (MP mod)          |
/// | - strength: i16                      |
/// | - agility: i16                       |
/// | - wisdom: i16                        |
/// | - constitution: i16                   |
/// | - to_dodge: i16 (evasion)           |
/// | - to_hit: i16 (accuracy)            |
/// | - attack: i16 (offense)              |
/// | - defense: i16 (armor class)         |
/// | - magical_strength: i16 (magical)    |
/// | - durability: i16 (item HP)          |
/// | - reserved_0x108: i16                |
/// | - reserved_0x10a: i16                |
/// | - req_strength: i16 (requirement)    |
/// | - reserved_0x10e: i16                |
/// | - req_agility: i16 (requirement)     |
/// | - reserved_0x112: i16                |
/// | - req_wisdom: i16 (requirement)      |
/// | - reserved_0x116..0x11a: i16 × 3    |
/// +--------------------------------------+
/// | [Record 2]                           |
/// | ... (same structure) ...             |
/// +--------------------------------------+
/// ```
///
/// # Field Categories
///
/// - **Identification**: `id` (auto-generated), `name` (30 bytes), `description` (202 bytes)
/// - **Economy**: `base_price` (i32, merchant valuation)
/// - **Stat Modifiers**: `health_points`, `mana_points`, `strength`, `agility`, `wisdom`, `constitution`
/// - **Combat Stats**: `to_dodge` (evasion), `to_hit` (accuracy), `attack` (offense), `defense` (armor)
/// - **Magical**: `magical_strength` (magical power)
/// - **Durability**: `durability` (item HP/wear limit)
/// - **Requirements**: `req_strength`, `req_agility`, `req_wisdom` (minimum stats to equip)
/// - **Runtime identity**: `weapon_item_id` is overwritten with the zero-based
///   record index by the game loader.
/// - **Reserved**: the seven `reserved_0x*` words are serialized in the file and
///   copied into inventory items, but are zero in every shipped record and have
///   no observed runtime use.
///
/// # Special Values
///
/// - `base_price`: i32 economic value
/// - `durability`: Item health points before destruction
/// - `weapon_item_id`: Runtime-only ID; the on-disk value is ignored and replaced
///   with the record index when the database loads.
/// - `reserved_0x*`: Preserved 16-bit values. All are zero in the shipped database.
/// - Requirements: 0 = no requirement
///
/// # File Purpose
///
/// Defines weapons and armor with stat modifications,
/// requirements, and durability. Used for equipment
/// system, character progression, and combat mechanics.
#[derive(
    Debug, Clone, Default, PartialEq, Serialize, Deserialize, Localizable, Extractor, RecordPatcher,
)]
#[extractor(property_item_size = 284)]
#[patcher(filename = "WeaponItem.db")]
pub struct WeaponItem {
    /// Internal record index (0-based) for the weapon/armor.
    #[extractor(id)]
    pub id: i32,
    /// Fixed-size string (30 bytes) for item name.
    #[translatable(encoding = "WINDOWS-1250", max_bytes = 30)]
    #[extractor(string(encoding = "WINDOWS-1250", size = 30))]
    pub name: String,
    /// Fixed-size string (202 bytes) for item description.
    #[translatable(encoding = "WINDOWS-1250", max_bytes = 202)]
    #[extractor(string(encoding = "WINDOWS-1250", size = 202))]
    pub description: String,
    /// Shop value in gold.
    #[extractor(primitive(type = "i32"))]
    pub base_price: i32,
    /// Runtime weapon-item identifier. The game loader overwrites this on-disk
    /// word with the zero-based record index, so it should not be edited as a
    /// persistent identifier.
    #[extractor(primitive(type = "i32"))]
    pub weapon_item_id: i32,
    /// HP modifier the equipment grants.
    #[extractor(primitive(type = "i16"))]
    pub health_points: i16,
    /// MP modifier the equipment grants.
    #[extractor(primitive(type = "i16"))]
    pub mana_points: i16,
    /// Strength buff granted.
    #[extractor(primitive(type = "i16"))]
    pub strength: i16,
    /// Agility modifier.
    #[extractor(primitive(type = "i16"))]
    pub agility: i16,
    /// Wisdom/Magic multiplier.
    #[extractor(primitive(type = "i16"))]
    pub wisdom: i16,
    /// Constitution / TF modifier.
    #[extractor(primitive(type = "i16"))]
    pub constitution: i16,
    /// Evasion bonus (Polish UI: `UNIK`).
    #[extractor(primitive(type = "i16"))]
    pub to_dodge: i16,
    /// Accuracy bonus (Polish UI: `TRF`).
    #[extractor(primitive(type = "i16"))]
    pub to_hit: i16,
    /// Base offensive stat.
    #[extractor(primitive(type = "i16"))]
    pub attack: i16,
    /// Base defensive armor class.
    #[extractor(primitive(type = "i16"))]
    pub defense: i16,
    /// Enhanced magical defense/offense.
    #[extractor(primitive(type = "i16"))]
    pub magical_strength: i16,
    /// Item health points or wear limit.
    #[extractor(primitive(type = "i16"))]
    pub durability: i16,
    /// Reserved serialized word at byte offset `0x108`. It is zero in all
    /// shipped records and has no observed runtime use.
    #[extractor(padding(count = 1, type = "i16"))]
    pub reserved_0x108: i16,
    /// Reserved serialized word at byte offset `0x10a`.
    #[extractor(padding(count = 1, type = "i16"))]
    pub reserved_0x10a: i16,
    /// Player base strength needed to equip.
    #[extractor(primitive(type = "i16"))]
    pub req_strength: i16,
    /// Reserved serialized word at byte offset `0x10e`.
    #[extractor(padding(count = 1, type = "i16"))]
    pub reserved_0x10e: i16,
    /// Player base agility needed to equip.
    #[extractor(primitive(type = "i16"))]
    pub req_agility: i16,
    /// Reserved serialized word at byte offset `0x112`.
    #[extractor(padding(count = 1, type = "i16"))]
    pub reserved_0x112: i16,
    /// Player base wisdom/magic needed to equip.
    #[extractor(primitive(type = "i16"))]
    pub req_wisdom: i16,
    /// Reserved serialized word at byte offset `0x116`.
    #[extractor(padding(count = 1, type = "i16"))]
    pub reserved_0x116: i16,
    /// Reserved serialized word at byte offset `0x118`.
    #[extractor(padding(count = 1, type = "i16"))]
    pub reserved_0x118: i16,
    /// Reserved serialized word at byte offset `0x11a`.
    #[extractor(padding(count = 1, type = "i16"))]
    pub reserved_0x11a: i16,
}

pub fn read_weapons_db(source_path: &Path) -> std::io::Result<Vec<WeaponItem>> {
    WeaponItem::read_file(source_path)
}

pub fn save_weapons(conn: &mut Connection, weapons: &[WeaponItem]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_weapon.sql"))?;
        for weapon in weapons {
            stmt.execute(params![
                weapon.id,
                weapon.name,
                weapon.description,
                weapon.base_price,
                weapon.weapon_item_id,
                weapon.health_points,
                weapon.mana_points,
                weapon.strength,
                weapon.agility,
                weapon.wisdom,
                weapon.constitution,
                weapon.to_dodge,
                weapon.to_hit,
                weapon.attack,
                weapon.defense,
                weapon.magical_strength,
                weapon.durability,
                weapon.reserved_0x108,
                weapon.reserved_0x10a,
                weapon.req_strength,
                weapon.reserved_0x10e,
                weapon.req_agility,
                weapon.reserved_0x112,
                weapon.req_wisdom,
                weapon.reserved_0x116,
                weapon.reserved_0x118,
                weapon.reserved_0x11a,
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

    fn weapon_bytes(name: &str, base_price: i32, weapon_item_id: i32, attack: i16) -> Vec<u8> {
        let mut rec = Vec::with_capacity(284);
        let mut name_buf = [0u8; 30];
        name_buf[..name.len().min(29)].copy_from_slice(&name.as_bytes()[..name.len().min(29)]);
        rec.extend_from_slice(&name_buf);
        rec.extend(vec![0u8; 202]); // description
        rec.extend_from_slice(&base_price.to_le_bytes());
        rec.extend_from_slice(&weapon_item_id.to_le_bytes());
        rec.extend(vec![0u8; 8]); // hp, mp, str, agi
        rec.extend(vec![0u8; 8]); // wis, con, dodge, to_hit
        rec.extend_from_slice(&attack.to_le_bytes());
        rec.extend(vec![0u8; 26]); // defense, magical_str, durability, pads, reqs
        rec
    }

    #[test]
    fn parse_single_weapon() {
        let mut data = 1i32.to_le_bytes().to_vec();
        data.extend(weapon_bytes("Sword", 300, 17, 25));
        assert_eq!(data.len(), 288);

        let mut c = Cursor::new(&data[..]);
        let weapons = WeaponItem::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(weapons.len(), 1);
        assert_eq!(weapons[0].name, "Sword");
        assert_eq!(weapons[0].base_price, 300);
        assert_eq!(weapons[0].weapon_item_id, 17);
        assert_eq!(weapons[0].attack, 25);
    }

    #[test]
    fn serialize_round_trip() {
        let mut data = 1i32.to_le_bytes().to_vec();
        data.extend(weapon_bytes("Sword", 300, 17, 25));
        let mut c = Cursor::new(&data[..]);
        let records = WeaponItem::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        WeaponItem::to_writer(&records, &mut out).unwrap();
        assert_eq!(out, data);
    }
}
