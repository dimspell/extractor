use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::{fs::File, path::Path};

use crate::references::references::{read_mapper, read_null_terminated_windows_1250, Extractor};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use encoding_rs::WINDOWS_1250;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

// ===========================================================================
// WEAPONITEM.DB FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | weaponItem.db - Weapons & Armor      |
// +--------------------------------------+
// | Encoding: Binary (Little-Endian)     |
// | Text Encoding: WINDOWS-1250          |
// | Header: 4-byte record count          |
// | Record Size: 284 bytes (71 × i16)     |
// +--------------------------------------+
// | [Header]                            |
// | - record_count: i32                  |
// +--------------------------------------+
// | [Record 1]                           |
// | - name: 30 bytes (WINDOWS-1250)      |
// | - description: 202 bytes (WINDOWS-1250)|
// | - base_price: i16                    |
// | - padding: i16 × 3                   |
// | - health_points: i16                 |
// | - mana_points: i16                  |
// | - strength: i16                      |
// | - agility: i16                       |
// | - wisdom: i16                        |
// | - constitution: i16                  |
// | - unk: i16                           |
// | - trf: i16                           |
// | - attack: i16                        |
// | - defense: i16                       |
// | - magical_power: i16                 |
// | - durability: i16                    |
// | - padding: i16 × 2                   |
// | - req_strength: i16                  |
// | - padding: i16                       |
// | - req_zw: i16                        |
// | - padding: i16                       |
// | - req_wisdom: i16                    |
// | - padding: i16 × 3                   |
// +--------------------------------------+
// | [Record 2]                           |
// | ... (same structure) ...             |
// +--------------------------------------+
//
// FIELD ABBREVIATIONS:
// - PZ: Health Points (Polish: Punkty Zdrowia)
// - PM: Mana Points (Polish: Punkty Magii)
// - SIŁ: Strength (Polish: Siła)
// - ZW: Agility (Polish: Zwinność)
// - MM: Wisdom/Magic (Polish: Magia/Mądrość)
// - TF: Constitution (Polish: Tężyzna Fizyczna)
// - UNK: Dodge (Polish: Unik)
// - TRF: Hit Rate (Polish: Trafienie)
// - ATK: Attack power
// - OBR: Defense (Polish: Obrona)
// - MAG: Magical power
// - WYT: Durability (Polish: Wytrzymałość)
// - REQ: Required stat for equipment
//
// FILE PURPOSE:
// Complete database of all weapons, armor, and equipment with statistics,
// requirements, and game properties. Used for character equipment,
// inventory management, and shop systems.
//
// ===========================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeaponItem {
    /// Internal record index (0-based) for the weapon/armor.
    pub id: i32,
    /// Fixed-size string (30 bytes) for item name.
    pub name: String,
    /// Fixed-size string (202 bytes) for item description.
    pub description: String,
    /// Shop value in gold.
    pub base_price: i16,
    /// HP modifier the equipment grants.
    pub health_points: i16,
    /// MP modifier the equipment grants.
    pub mana_points: i16,
    /// Strength buff granted.
    pub strength: i16,
    /// Agility modifier.
    pub agility: i16,
    /// Wisdom/Magic multiplier.
    pub wisdom: i16,
    /// Constitution / TF modifier.
    pub constitution: i16,
    /// Unknown modifier parameter.
    pub to_dodge: i16,
    /// Hit rate or TRF stat bonus.
    pub to_hit: i16,
    /// Base offensive stat.
    pub attack: i16,
    /// Base defensive armor class.
    pub defense: i16,
    /// Enhanced magical defense/offense.
    pub magical_strength: i16,
    /// Item health points or wear limit.
    pub durability: i16,
    /// Player base strength needed to equip.
    pub req_strength: i16,
    /// Player base agility needed to equip.
    pub req_agility: i16,
    /// Player base wisdom/magic needed to equip.
    pub req_wisdom: i16,
}

/// Stores stats, prices, and requirements for weapons and armor.
///
/// Reads file: `CharacterInGame/weaponItem.db`
/// # File Format: `CharacterInGame/weaponItem.db`
///
/// Binary file, little-endian. Starts with a 4-byte i32 record count.
/// Each record is a fixed-size block containing:
/// - `name`        : 30 bytes, null-padded, WINDOWS-1250
/// - `description` : 202 bytes, null-padded, WINDOWS-1250
/// - Stats         : sequence of i16 fields (base_price, padding×3, PZ, PM, SIŁ, ZW,
///                   MM, TF, UNK, TRF, ATK, OBR, MAG, WYT, pad×2,
///                   REQ_SIŁ, pad, REQ_ZW, pad, REQ_MM, pad×3)
impl Extractor for WeaponItem {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let file = File::open(source_path)?;

        let metadata = file.metadata()?;
        let file_len = metadata.len();

        let mut reader = BufReader::new(file);

        const COUNTER_SIZE: u8 = 4;
        const PROPERTY_ITEM_SIZE: i32 = 71 * 4;
        let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;

        let mut weapons: Vec<WeaponItem> = vec![];
        for i in 0..elements as usize {
            // name
            let mut buffer = [0u8; 30];
            reader.read_exact(&mut buffer)?;
            let name = read_null_terminated_windows_1250(&buffer).unwrap();

            // description
            let mut buffer = [0u8; 202];
            reader.read_exact(&mut buffer)?;
            let description = read_null_terminated_windows_1250(&buffer).unwrap();

            // "Base price"
            let base_price = reader.read_i16::<LittleEndian>()?;
            reader.read_i16::<LittleEndian>()?;
            reader.read_i16::<LittleEndian>()?;
            reader.read_i16::<LittleEndian>()?;
            let health_points = reader.read_i16::<LittleEndian>()?;
            let mana_points = reader.read_i16::<LittleEndian>()?;
            let strength = reader.read_i16::<LittleEndian>()?;
            let agility = reader.read_i16::<LittleEndian>()?;
            let wisdom = reader.read_i16::<LittleEndian>()?;
            let constitution = reader.read_i16::<LittleEndian>()?;
            let to_dodge = reader.read_i16::<LittleEndian>()?;
            let to_hit = reader.read_i16::<LittleEndian>()?;
            let attack = reader.read_i16::<LittleEndian>()?;
            let defense = reader.read_i16::<LittleEndian>()?;
            let magical_strength = reader.read_i16::<LittleEndian>()?;
            let durability = reader.read_i16::<LittleEndian>()?;
            reader.read_i16::<LittleEndian>()?;
            reader.read_i16::<LittleEndian>()?;
            let req_strength = reader.read_i16::<LittleEndian>()?;
            reader.read_i16::<LittleEndian>()?;
            let req_agility = reader.read_i16::<LittleEndian>()?;
            reader.read_i16::<LittleEndian>()?;
            let req_wisdom = reader.read_i16::<LittleEndian>()?;
            reader.read_i16::<LittleEndian>()?;
            reader.read_i16::<LittleEndian>()?;
            reader.read_i16::<LittleEndian>()?;

            let item = WeaponItem {
                id: i as i32,
                attack,
                base_price,
                description: description.to_string(),
                magical_strength,
                wisdom,
                name: name.to_string(),
                defense,
                mana_points,
                health_points,
                req_wisdom,
                req_strength,
                req_agility,
                strength,
                constitution,
                to_hit,
                to_dodge,
                durability,
                agility,
            };
            weapons.push(item);
        }

        Ok(weapons)
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
            let (cow, _, _) = WINDOWS_1250.encode(&record.description);
            let len = std::cmp::min(cow.len(), 202);
            desc_buf[..len].copy_from_slice(&cow[..len]);
            writer.write_all(&desc_buf)?;

            writer.write_i16::<LittleEndian>(record.base_price)?;
            writer.write_i16::<LittleEndian>(0)?;
            writer.write_i16::<LittleEndian>(0)?;
            writer.write_i16::<LittleEndian>(0)?;
            writer.write_i16::<LittleEndian>(record.health_points)?;
            writer.write_i16::<LittleEndian>(record.mana_points)?;
            writer.write_i16::<LittleEndian>(record.strength)?;
            writer.write_i16::<LittleEndian>(record.agility)?;
            writer.write_i16::<LittleEndian>(record.wisdom)?;
            writer.write_i16::<LittleEndian>(record.constitution)?;
            writer.write_i16::<LittleEndian>(record.to_dodge)?;
            writer.write_i16::<LittleEndian>(record.to_hit)?;
            writer.write_i16::<LittleEndian>(record.attack)?;
            writer.write_i16::<LittleEndian>(record.defense)?;
            writer.write_i16::<LittleEndian>(record.magical_strength)?;
            writer.write_i16::<LittleEndian>(record.durability)?;
            writer.write_i16::<LittleEndian>(0)?;
            writer.write_i16::<LittleEndian>(0)?;
            writer.write_i16::<LittleEndian>(record.req_strength)?;
            writer.write_i16::<LittleEndian>(0)?;
            writer.write_i16::<LittleEndian>(record.req_agility)?;
            writer.write_i16::<LittleEndian>(0)?;
            writer.write_i16::<LittleEndian>(record.req_wisdom)?;
            writer.write_i16::<LittleEndian>(0)?;
            writer.write_i16::<LittleEndian>(0)?;
            writer.write_i16::<LittleEndian>(0)?;
        }

        Ok(())
    }
}

pub fn read_weapons_db(source_path: &Path) -> std::io::Result<Vec<WeaponItem>> {
    WeaponItem::read_file(source_path)
}

pub fn save_weapons(conn: &mut Connection, weapons: &Vec<WeaponItem>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_weapon.sql"))?;
        for weapon in weapons {
            stmt.execute(params![
                weapon.id,
                weapon.name,
                weapon.description,
                weapon.base_price,
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
                weapon.req_strength,
                weapon.req_agility,
                weapon.req_wisdom,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
