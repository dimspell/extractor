use std::{fs::File, path::Path};
use std::io::BufReader;
use std::io::prelude::*;

use byteorder::{LittleEndian, ReadBytesExt};
use encoding_rs::WINDOWS_1250;
use serde::{Deserialize, Serialize};
use crate::references::references::read_mapper;

#[derive(Debug, Serialize, Deserialize)]
pub struct WeaponItem {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub base_price: i16,
    pub health_points: i16,
    pub magic_points: i16,
    pub strength: i16,
    pub agility: i16,
    pub wisdom: i16,
    pub tf: i16,
    pub unk: i16,
    pub trf: i16,
    pub attack: i16,
    pub defense: i16,
    pub mag: i16,
    pub durability: i16,
    pub req_strength: i16,
    pub req_zw: i16,
    pub req_wisdom: i16,
}

pub fn read_weapons_db(source_path: &Path) -> std::io::Result<Vec<WeaponItem>> {
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
