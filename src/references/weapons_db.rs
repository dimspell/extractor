use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::{fs::File, path::Path};

use crate::references::references::{read_mapper, Extractor};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use encoding_rs::WINDOWS_1250;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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
    pub magic_points: i16,
    /// Strength buff granted.
    pub strength: i16,
    /// Agility modifier.
    pub agility: i16,
    /// Wisdom/Magic multiplier.
    pub wisdom: i16,
    /// Constitution / TF modifier.
    pub tf: i16,
    /// Unknown modifier parameter.
    pub unk: i16,
    /// Hit rate or TRF stat bonus.
    pub trf: i16,
    /// Base offensive stat.
    pub attack: i16,
    /// Base defensive armor class.
    pub defense: i16,
    /// Enhanced magical defense/offense.
    pub mag: i16,
    /// Item health points or wear limit.
    pub durability: i16,
    /// Player base strength needed to equip.
    pub req_strength: i16,
    /// Player base agility (ZW) needed to equip.
    pub req_zw: i16,
    /// Player base wisdom/magic needed to equip.
    pub req_wisdom: i16,

}

/// Stores stats, prices, and requirements for weapons and armor.
///
/// Reads file: `CharacterInGame/weaponItem.db`
impl Extractor for WeaponItem {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let file = File::open(source_path)?;

        let metadata = file.metadata()?;
        let file_len = metadata.len();

        let mut reader = BufReader::new(file);

        const COUNTER_SIZE: u8 = 4;
        const PROPERTY_ITEM_SIZE: i32 = 71 * 4;
        let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;

        const NAME_STRING_MAX_LENGTH: usize = 30;
        const DESCRIPTION_STRING_MAX_LENGTH: usize = 202;

        let mut weapons: Vec<WeaponItem> = vec![];
        for i in 0..elements as usize {
            // println!("{i}");

            // name
            let mut buffer = [0u8; NAME_STRING_MAX_LENGTH];
            reader.read_exact(&mut buffer)?;
            let dst = WINDOWS_1250.decode(&buffer);
            let name = dst.0.trim_end_matches("\0").trim();
            // println!("{:?}", name);

            // description
            let mut buffer = [0u8; DESCRIPTION_STRING_MAX_LENGTH];
            reader.read_exact(&mut buffer)?;
            let dst = WINDOWS_1250.decode(&buffer);
            let description = dst.0.trim_end_matches("\0").trim_end_matches("\00").trim();
            // println!("{:?}", description);

            // "Base price"
            let base_price = reader.read_i16::<LittleEndian>()?;
            //
            reader.read_i16::<LittleEndian>()?;
            //
            reader.read_i16::<LittleEndian>()?;
            //
            reader.read_i16::<LittleEndian>()?;
            // "PZ"
            let health_points = reader.read_i16::<LittleEndian>()?;
            // "PM"
            let magic_points = reader.read_i16::<LittleEndian>()?;
            // "SIŁ"
            let strength = reader.read_i16::<LittleEndian>()?;
            // "ZW"
            let zw = reader.read_i16::<LittleEndian>()?;
            // "MM"
            let wisdom = reader.read_i16::<LittleEndian>()?;
            // "TF"
            let tf = reader.read_i16::<LittleEndian>()?;
            // "UNK"
            let unk = reader.read_i16::<LittleEndian>()?;
            // "TRF"
            let trf = reader.read_i16::<LittleEndian>()?;
            // "ATK"
            let attack = reader.read_i16::<LittleEndian>()?;
            // "OBR"
            let defense = reader.read_i16::<LittleEndian>()?;
            // "MAG"
            let mag = reader.read_i16::<LittleEndian>()?;
            // "WYT"
            let durability = reader.read_i16::<LittleEndian>()?;
            //
            reader.read_i16::<LittleEndian>()?;
            //
            reader.read_i16::<LittleEndian>()?;
            // "REQ SIŁ"
            let req_strength = reader.read_i16::<LittleEndian>()?;
            //
            reader.read_i16::<LittleEndian>()?;
            // "REQ ZW"
            let req_zw = reader.read_i16::<LittleEndian>()?;
            //
            reader.read_i16::<LittleEndian>()?;
            // "REQ MM"
            let req_wisdom = reader.read_i16::<LittleEndian>()?;
            //
            reader.read_i16::<LittleEndian>()?;
            //
            reader.read_i16::<LittleEndian>()?;
            //
            reader.read_i16::<LittleEndian>()?;

            let item = WeaponItem {
                id: i as i32,
                attack,
                base_price,
                description: description.to_string(),
                mag,
                wisdom,
                name: name.to_string(),
                defense,
                magic_points,
                health_points,
                req_wisdom,
                req_strength,
                req_zw,
                strength,
                tf,
                trf,
                unk,
                durability,
                agility: zw,
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
            writer.write_i16::<LittleEndian>(record.magic_points)?;
            writer.write_i16::<LittleEndian>(record.strength)?;
            writer.write_i16::<LittleEndian>(record.agility)?;
            writer.write_i16::<LittleEndian>(record.wisdom)?;
            writer.write_i16::<LittleEndian>(record.tf)?;
            writer.write_i16::<LittleEndian>(record.unk)?;
            writer.write_i16::<LittleEndian>(record.trf)?;
            writer.write_i16::<LittleEndian>(record.attack)?;
            writer.write_i16::<LittleEndian>(record.defense)?;
            writer.write_i16::<LittleEndian>(record.mag)?;
            writer.write_i16::<LittleEndian>(record.durability)?;
            writer.write_i16::<LittleEndian>(0)?;
            writer.write_i16::<LittleEndian>(0)?;
            writer.write_i16::<LittleEndian>(record.req_strength)?;
            writer.write_i16::<LittleEndian>(0)?;
            writer.write_i16::<LittleEndian>(record.req_zw)?;
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
