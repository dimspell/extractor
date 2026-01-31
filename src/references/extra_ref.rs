use std::io::prelude::*;
use std::io::BufReader;
use std::{fs::File, path::Path};

use crate::references::references::{read_mapper, read_null_terminated_windows_1250};
use byteorder::{LittleEndian, ReadBytesExt};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ExtraRef {
    pub id: i32,
    pub number_in_file: u8,
    pub ext_id: u8,
    pub name: String,
    pub object_type: u8,
    pub x_pos: i32,
    pub y_pos: i32,
    pub rotation: u8,
    pub closed: i32,
    pub required_item_id: u8,
    pub required_item_type_id: u8,
    pub required_item_id2: u8,
    pub required_item_type_id2: u8,
    pub gold_amount: i32,
    pub item_id: u8,
    pub item_type_id: u8,
    pub item_count: i32,
    pub event_id: i32,
    pub message_id: i32,
    pub visibility: u8,
}

pub fn read_extra_ref(source_path: &Path) -> std::io::Result<Vec<ExtraRef>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 4;
    const PROPERTY_ITEM_SIZE: i32 = 46 * 4;
    // const FILLER: u8 = 0xcd;

    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
    let mut refs: Vec<ExtraRef> = Vec::with_capacity(elements as usize);

    for i in 0..elements {
        let number_in_file = reader.read_u8()?;

        reader.read_u8()?;
        let ext_id = reader.read_u8()?; // Id from Extra.ini

        let mut buffer = [0u8; 32];
        reader.read_exact(&mut buffer)?;
        let name = read_null_terminated_windows_1250(&buffer).unwrap();

        let object_type = reader.read_u8()?; // 7-magic, 6-interactive object, 5-altar, 4-sign, 2-door, 0-chest

        let x_pos = reader.read_i32::<LittleEndian>()?;
        let y_pos = reader.read_i32::<LittleEndian>()?;
        let rotation = reader.read_u8()?;

        reader.read_u8()?;
        reader.read_u8()?;
        reader.read_u8()?;

        reader.read_i32::<LittleEndian>()?;

        let closed = reader.read_i32::<LittleEndian>()?; // chest 0-open, 1-closed

        let required_item_id = reader.read_u8()?; // lower bound
        let required_item_type_id = reader.read_u8()?;
        reader.read_u8()?;
        reader.read_u8()?;

        let required_item_id2 = reader.read_u8()?; // upper bound
        let required_item_type_id2 = reader.read_u8()?;
        reader.read_u8()?;
        reader.read_u8()?;

        let mut buffer = [0u8; 16];
        reader.read_exact(&mut buffer)?;

        let gold_amount = reader.read_i32::<LittleEndian>()?;

        let item_id = reader.read_u8()?;
        let item_type_id = reader.read_u8()?;
        reader.read_u8()?;
        reader.read_u8()?;

        let item_count = reader.read_i32::<LittleEndian>()?;

        let mut buffer = [0u8; 40];
        reader.read_exact(&mut buffer)?;

        let event_id = reader.read_i32::<LittleEndian>()?; // id from event.ini
        let message_id = reader.read_i32::<LittleEndian>()?; // id from message.scr for signs

        let mut buffer = [0u8; 32];
        reader.read_exact(&mut buffer)?;

        let visibility = reader.read_u8()?;

        let mut buffer = [0u8; 3];
        reader.read_exact(&mut buffer)?;
        println!("{:?}", buffer);

        refs.push(ExtraRef {
            id: i,
            number_in_file,
            ext_id,
            name: name.to_string(),
            object_type,
            x_pos,
            y_pos,
            rotation,
            closed,
            required_item_id,
            required_item_type_id,
            required_item_id2,
            required_item_type_id2,
            gold_amount,
            item_id,
            item_type_id,
            item_count,
            event_id,
            message_id,
            visibility,
        })
    }

    Ok(refs)
}
