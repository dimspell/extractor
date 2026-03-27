use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::{fs::File, path::Path};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use encoding_rs::WINDOWS_1250;
use rusqlite::{params, Connection, Result};
use serde::Serialize;

use crate::references::enums::{EditItemEffect, EditItemModification};
use crate::references::references::{read_mapper, read_null_terminated_windows_1250, Extractor};

// ===========================================================================
// EDITITEM.DB FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | EditItem.db - Modifiable Items       |
// +--------------------------------------+
// | Encoding: Binary (Little-Endian)     |
// | Text Encodings: Mixed                |
// | Header: 4-byte record count          |
// | Record Size: 268 bytes (67 × i32)    |
// +--------------------------------------+
// | [Header]                            |
// | - record_count: i32                  |
// +--------------------------------------+
// | [Record 1]                           |
// | - name: 30 bytes (WINDOWS-1250)     |
// | - description: 202 bytes (EUC-KR)   |
// | - base_price: i16                   |
// | - padding: 6 bytes                   |
// | - health_points: i16 (PZ)            |
// | - magic_points: i16 (PM)             |
// | - strength: i16 (SIŁ)               |
// | - agility: i16 (ZW)                 |
// | - wisdom: i16 (MM)                  |
// | - constitution: i16 (TF)            |
// | - to_dodge: i16 (UNK)              |
// | - to_hit: i16 (TRF)                 |
// | - offense: i16 (ATK)               |
// | - defense: i16 (OBR)                |
// | - padding: i16                      |
// | - item_destroying_power: i16        |
// | - padding: u8                      |
// | - modifies_item: u8                 |
// | - additional_effect: i16            |
// +--------------------------------------+
// | [Record 2]                           |
// | ... (same structure) ...             |
// +--------------------------------------+
//
// FIELD ABBREVIATIONS:
// - PZ: Health Points (Polish: Punkty Zdrowia)
// - PM: Magic Points (Polish: Punkty Magii)
// - SIŁ: Strength (Polish: Siła)
// - ZW: Agility (Polish: Zwinność)
// - MM: Wisdom (Polish: Mądrość)
// - TF: Constitution (Polish: Tężyzna Fizyczna)
// - UNK: Unknown modifier
// - TRF: Hit Rate (Polish: Trafienie)
// - ATK: Attack power
// - OBR: Defense (Polish: Obrona)
//
// MODIFICATION TYPES:
// - 0: Does not modify base item
// - 1: Temporary modification
// - 2: Permanent enhancement
//
// SPECIAL VALUES:
// - base_price = 0: Non-tradable items
// - item_destroying_power: Durability impact
// - Fixed-size string fields
// - Mixed text encodings
//
// FILE PURPOSE:
// Defines modifiable items with upgradeable statistics
// and special effects. Used for item enhancement,
// crafting, and equipment customization systems.
//
// ===========================================================================

#[derive(Debug, Serialize)]
pub struct EditItem {
    /// Iteration tracking for editor modifications.
    pub index: i32,
    /// Asset identifier string.
    pub name: String,
    /// Standard inventory tool-tip.
    pub description: String,
    /// Economic valuation offset.
    pub base_price: i16,
    /// Base additive metric for derived vitality.
    pub health_points: i16,
    /// Spell scaling base factor.
    pub magic_points: i16,
    /// Stat adjustment logic constant.
    pub strength: i16,
    /// Physical tracking parameter limit.
    pub agility: i16,
    /// Mind attribute modifier block.
    pub wisdom: i16,
    /// Core status alignment tracking.
    pub constitution: i16,
    /// Raw deflection parameter.
    pub to_dodge: i16,
    /// Base hit resolution constant.
    pub to_hit: i16,
    /// Flat output augmentation rating.
    pub offense: i16,
    /// Armor calculation pool scaling rating.
    pub defense: i16,
    /// Durability erosion factor.
    pub item_destroying_power: i16,
    /// Flag specifying if behavior mutates.
    pub modifies_item: EditItemModification,
    /// Procedural elemental modifier appended (poison, fire).
    pub additional_effect: EditItemEffect,
}

/// Stores definitions for modifiable base items.
///
/// Reads file: `CharacterInGame/EditItem.db`
/// # File Format: `CharacterInGame/EditItem.db`
///
/// Binary file, little-endian.  Starts with a 4-byte i32 record count.
/// Each record is exactly `67 × 4 = 268` bytes:
/// - `name`                 : 30 bytes, null-padded, WINDOWS-1250
/// - `description`          : 202 bytes, null-padded, WINDOWS-1250
/// - `base_price`           : i16
/// - 6 bytes padding
/// - Stats (all i16)        : PZ, PM, SIŁ, ZW, MM, TF, UNK, TRF, ATK, OBR
/// - 2 bytes padding
/// - `item_destroying_power`: i16
/// - 1 byte padding
/// - `modifies_item`        : u8
/// - `additional_effect`    : i16
impl Extractor for EditItem {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let file = File::open(source_path)?;

        let metadata = file.metadata()?;
        let file_len = metadata.len();

        let mut reader = BufReader::new(file);

        const COUNTER_SIZE: u8 = 4;
        const PROPERTY_ITEM_SIZE: i32 = 67 * 4;

        let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
        let mut items: Vec<EditItem> = Vec::with_capacity(elements as usize);

        for i in 0..elements {
            let mut buffer = [0u8; 30];
            reader.read_exact(&mut buffer)?;
            let name = read_null_terminated_windows_1250(&buffer).unwrap();

            let mut buffer = [0u8; 202];
            reader.read_exact(&mut buffer)?;
            let description = read_null_terminated_windows_1250(&buffer).unwrap();

            let base_price = reader.read_i16::<LittleEndian>()?;

            let mut buffer = [0u8; 3 * 2];
            reader.read_exact(&mut buffer)?;

            let pz = reader.read_i16::<LittleEndian>()?; // PZ
            let pm = reader.read_i16::<LittleEndian>()?; // PM
            let sil = reader.read_i16::<LittleEndian>()?; // SIŁ
            let zw = reader.read_i16::<LittleEndian>()?; // ZW
            let mm = reader.read_i16::<LittleEndian>()?; // MM
            let tf = reader.read_i16::<LittleEndian>()?; // TF
            let unk = reader.read_i16::<LittleEndian>()?; // UNK
            let trf = reader.read_i16::<LittleEndian>()?; // TRF
            let atk = reader.read_i16::<LittleEndian>()?; // ATK
            let obr = reader.read_i16::<LittleEndian>()?; // OBR

            reader.read_i16::<LittleEndian>()?;

            let item_destroying_power = reader.read_i16::<LittleEndian>()?; // durability probably
            reader.read_u8()?;

            let modifies_item_raw = reader.read_u8()?;
            let additional_effect_raw = reader.read_i16::<LittleEndian>()?; // poison or burn or none

            let modifies_item = EditItemModification::from_u8(modifies_item_raw)
                .unwrap_or(EditItemModification::DoesNotModify);
            let additional_effect =
                EditItemEffect::from_i16(additional_effect_raw).unwrap_or(EditItemEffect::None);

            items.push(EditItem {
                index: i,
                name: name.to_string(),
                description: description.to_string(),
                base_price,
                health_points: pz,
                magic_points: pm,
                strength: sil,
                agility: zw,
                wisdom: mm,
                constitution: tf,
                to_dodge: unk,
                to_hit: trf,
                offense: atk,
                defense: obr,
                item_destroying_power,
                modifies_item,
                additional_effect,
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
            let (cow, _, _) = WINDOWS_1250.encode(&record.description);
            let len = std::cmp::min(cow.len(), 202);
            desc_buf[..len].copy_from_slice(&cow[..len]);
            writer.write_all(&desc_buf)?;

            writer.write_i16::<LittleEndian>(record.base_price)?;
            writer.write_all(&[0u8; 6])?;

            writer.write_i16::<LittleEndian>(record.health_points)?;
            writer.write_i16::<LittleEndian>(record.magic_points)?;
            writer.write_i16::<LittleEndian>(record.strength)?;
            writer.write_i16::<LittleEndian>(record.agility)?;
            writer.write_i16::<LittleEndian>(record.wisdom)?;
            writer.write_i16::<LittleEndian>(record.constitution)?;
            writer.write_i16::<LittleEndian>(record.to_dodge)?;
            writer.write_i16::<LittleEndian>(record.to_hit)?;
            writer.write_i16::<LittleEndian>(record.offense)?;
            writer.write_i16::<LittleEndian>(record.defense)?;

            writer.write_i16::<LittleEndian>(0)?;
            writer.write_i16::<LittleEndian>(record.item_destroying_power)?;
            writer.write_u8(0)?;

            writer.write_u8(u8::from(record.modifies_item))?;
            writer.write_i16::<LittleEndian>(i16::from(record.additional_effect))?;
        }
        Ok(())
    }
}

pub fn read_edit_item_db(source_path: &Path) -> std::io::Result<Vec<EditItem>> {
    EditItem::read_file(source_path)
}

pub fn save_edit_items(conn: &mut Connection, edit_items: &Vec<EditItem>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_edit_item.sql"))?;
        for item in edit_items {
            stmt.execute(params![
                item.index,
                item.name,
                item.description,
                item.base_price,
                item.health_points,
                item.magic_points,
                item.strength,
                item.agility,
                item.wisdom,
                item.constitution,
                item.to_dodge,
                item.to_hit,
                item.to_hit,
                item.offense,
                item.defense,
                item.item_destroying_power,
                u8::from(item.modifies_item),
                i16::from(item.additional_effect),
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
