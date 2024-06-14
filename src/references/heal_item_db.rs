use std::{fs::File, path::Path};
use std::io::BufReader;
use std::io::prelude::*;

use byteorder::{LittleEndian, ReadBytesExt};
use encoding_rs::{EUC_KR, WINDOWS_1250};

use crate::references::references::read_mapper;

#[derive(Debug)]
pub struct HealItem {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub base_price: i16,
    pub pz: i16,
    pub pm: i16,
    pub full_pz: u8,
    pub full_pm: u8,
    pub poison_heal: u8,
    pub petrif_heal: u8,
    pub polimorph_heal: u8,
}

pub fn read_heal_item_db(source_path: &Path) -> std::io::Result<Vec<HealItem>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 4;
    const PROPERTY_ITEM_SIZE: i32 = 63 * 4;
    // const FILLER: u8 = 0x0;

    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
    let mut items: Vec<HealItem> = Vec::with_capacity(elements as usize);

    for i in 0..elements {
        let mut buffer = [0u8; 30];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let name = dst.0.trim_end_matches("\0").trim();

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
        let full_pz = reader.read_u8()?;
        let full_pm = reader.read_u8()?;
        let poison_heal = reader.read_u8()?;
        let petrif_heal = reader.read_u8()?;
        let polimorph_heal = reader.read_u8()?;

        reader.read_u8()?;
        reader.read_i16::<LittleEndian>()?;

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