use std::io::prelude::*;
use std::io::{BufReader, Result, Seek, SeekFrom};
use std::num::IntErrorKind;
use std::{fs::File, path::Path};

use byteorder::{LittleEndian, ReadBytesExt};
use encoding_rs::WINDOWS_1250;

pub fn read_null_terminated_windows_1250(bytes: &[u8]) -> core::result::Result<String, String> {
    // Find the first null byte (or use a fixed length if no null terminator)
    let (_data, _) = bytes.split_last().ok_or("Empty input")?;
    let data_len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());

    // Decode the Windows-1250 portion before the null terminator
    let out = WINDOWS_1250.decode(&bytes[..data_len]);
    // .(|e| format!("Decoding error: {}", e))?;

    let decoded = out.0;
    let had_errors = out.2;

    if had_errors {
        return Err("Invalid Windows-1250 sequence".to_string());
    }

    Ok(decoded.to_string())
}

pub fn parse_null(s: &str) -> Option<String> {
    if s == "null" {
        None
    } else {
        Some(s.to_string())
    }
}

pub fn parse_int(s: &str) -> Option<i32> {
    match s.parse::<i32>() {
        Ok(value) => Some(value),
        Err(err) => match err.kind() {
            IntErrorKind::Empty => None,
            _ => {
                println!("{err:?} {s}");
                None
            }
        },
    }
}

// Message.scr
// first line of text
// second line of text or null
// third line of text or null

// Quest.scr
// id
// dairy type 0=main quest 1=side quest 2=traders journal
// title
// description

//     todo!(); // Eventnpc.ref

//     // NPCs used only in events
//     //
//     // id
//     // sprite id
//     //     ?
//     //     ?
//     //     ?
//     //     ?
//     // x coordinate,
//     // y coordinate,
//     // 30 times ?

//     todo!(); // Eventnpc.ref

pub fn read_mutli_magic_db(source_path: &Path) -> Result<()> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 4;
    const PROPERTY_ITEM_SIZE: i32 = 90;
    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;

    let pos = reader.seek(SeekFrom::Current(0))?;
    println!(
        "{:?} {:?} {:?} {:?}",
        file_len,
        elements,
        pos,
        PROPERTY_ITEM_SIZE * elements
    );

    for _i in 0..elements {
        let mut buffer = [0u8; 90];
        reader.read_exact(&mut buffer)?;
        println!("{:?}", buffer);
    }

    // println!("{:?} {:?} {:?}", file_len, elements, pos);
    Ok(())
}

pub fn read_mapper(
    reader: &mut BufReader<File>,
    file_len: u64,
    counter_size: u8,
    property_item_size: i32,
) -> Result<i32> {
    let space_for_elements =
        (((file_len - counter_size as u64) as f64) / property_item_size as f64).floor();
    let space_for_elements: i32 = space_for_elements as i32;

    let expected_elements = if counter_size > 0 {
        reader.read_i32::<LittleEndian>()?
    } else {
        space_for_elements
    };
    if expected_elements != space_for_elements {
        println!(
            "expected_elements: {expected_elements} / space_for_elements: {space_for_elements}"
        );
    }

    Ok(space_for_elements)
}
