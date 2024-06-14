use std::{fs::File, path::Path};
use std::io::{Cursor, prelude::*};
use std::io::BufReader;

use byteorder::{LittleEndian, ReadBytesExt};
use encoding_rs::{EUC_KR, WINDOWS_1250};
use crate::references::references::read_mapper;

#[derive(Debug)]
pub struct MiscItem {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub base_price: i32,
}

pub fn read_misc_item_db(source_path: &Path) -> std::io::Result<Vec<MiscItem>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 4;
    const PROPERTY_ITEM_SIZE: i32 = 64 * 4;
    // const FILLER: u8 = 0x0;

    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
    let mut items: Vec<MiscItem> = Vec::with_capacity(elements as usize);

    for i in 0..elements {
        let mut buffer = [0u8; 30];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let name = dst.0.trim_end_matches("\0").trim();

        let mut buffer = [0u8; 202];
        reader.read_exact(&mut buffer)?;
        let dst = EUC_KR.decode(&buffer);
        let description = dst.0.trim_end_matches("\0").trim();

        let base_price = reader.read_i32::<LittleEndian>()?;

        let mut buffer = [0u8; 20];
        reader.read_exact(&mut buffer)?;
        let cursor = Cursor::new(&buffer);

        let dst = EUC_KR.decode(&buffer);
        let dst = dst.0.trim_end_matches("\0");
        let dst = dst.trim_start_matches("\0");
        // let name = dst.0.trim_end_matches("\0").trim();

        println!("{name} {description} {:?}, {dst:?}", cursor);

        items.push(MiscItem {
            id: i,
            base_price,
            name: name.to_string(),
            description: description.to_string(),
        })
    }

    Ok(items)
}