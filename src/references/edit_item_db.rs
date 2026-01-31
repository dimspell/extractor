use std::io::prelude::*;
use std::io::BufReader;
use std::{fs::File, path::Path};

use byteorder::{LittleEndian, ReadBytesExt};
use encoding_rs::WINDOWS_1250;
use serde::Serialize;

use crate::references::references::{read_mapper, read_null_terminated_windows_1250};

#[derive(Debug, Serialize)]
pub struct EditItem {
    pub index: i32,
    pub name: String,
    pub description: String,
    pub base_price: i16,
    pub health_points: i16,
    pub magic_points: i16,
    pub strength: i16,
    pub agility: i16,
    pub wisdom: i16,
    pub constitution: i16,
    pub to_dodge: i16,
    pub to_hit: i16,
    pub offense: i16,
    pub defense: i16,
    pub item_destroying_power: i16,
    pub modifies_item: u8,
    pub additional_effect: i16,
}

pub fn read_edit_item_db(source_path: &Path) -> std::io::Result<Vec<EditItem>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 4;
    const PROPERTY_ITEM_SIZE: i32 = 67 * 4;
    // const FILLER: u8 = 0x0;

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

        let modifies_item = reader.read_u8()?;
        let additional_effect = reader.read_i16::<LittleEndian>()?; // poison or burn or none

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
