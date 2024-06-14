use std::{fs::File, path::Path};
use std::io::BufReader;
use std::io::prelude::*;

use byteorder::{LittleEndian, ReadBytesExt};
use encoding_rs::WINDOWS_1250;

use crate::references::references::read_mapper;

#[derive(Debug)]
pub struct EditItem {
    pub index: i32,
    pub name: String,
    pub description: String,
    pub base_price: i16,
    pub pz: i16,
    pub pm: i16,
    pub sil: i16,
    pub zw: i16,
    pub mm: i16,
    pub tf: i16,
    pub unk: i16,
    pub trf: i16,
    pub atk: i16,
    pub obr: i16,
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
        let dst = WINDOWS_1250.decode(&buffer);
        let name = dst.0.trim_end_matches("\0").trim();

        let mut buffer = [0u8; 202];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let description = dst.0.trim_end_matches("\0").trim();

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
            pz,
            pm,
            sil,
            zw,
            mm,
            tf,
            unk,
            trf,
            atk,
            obr,
            item_destroying_power,
            modifies_item,
            additional_effect,
        })
    }
    Ok(items)
}