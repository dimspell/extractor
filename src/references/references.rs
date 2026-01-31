use std::io::{prelude::*, Cursor};
use std::io::{BufRead, BufReader, Result, Seek, SeekFrom};
use std::num::IntErrorKind;
use std::{fs::File, path::Path};

use byteorder::{LittleEndian, ReadBytesExt};
use encoding_rs::{EUC_KR, WINDOWS_1250};
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::Serialize;

pub fn read_null_terminated_windows_1250(bytes: &[u8]) -> core::result::Result<String, String> {
    // Find the first null byte (or use a fixed length if no null terminator)
    let (data, _) = bytes.split_last().ok_or("Empty input")?;
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

// test {
//     // Simulate a null-terminated Windows-1250 string (e.g., "Ahoj" + \0)
//     let raw_bytes = b"Ahoj\0Test"; // "Ahoj" in Windows-1250 is [0xC3, 0x9Ah, 0xF3, 0x9Ah, 0xF3, 0x9Ah] (but this is UTF-8; adjust for actual Windows-1250)
//                                    // For Windows-1250, "Ahoj" is [0xE1, 0xF3, 0xE8, 0xF8, 0xE9] (example; check actual encoding)
//                                    // let actual_windows_1250_bytes = vec![0xE1, 0xF3, 0xE8, 0xF8, 0xE9, 0x00]; // Replace with real bytes

//     match read_null_terminated_windows_1250(raw_bytes) {
//         Ok(s) => println!("Decoded: {}", s),
//         Err(e) => eprintln!("Error: {}", e),
//     }
// }

struct OnMapSpriteInfo {
    x: i32,
    y: i32,
    db_id: i32,
    sprite_id: i32,
    sprite_seq: i32,
    flip: bool,
}

pub fn read_ini() -> Result<()> {
    let f = File::open(&Path::new("sample-data/Extra.ini"))?;
    let mut reader = BufReader::new(f);

    loop {
        let mut line = String::new();
        let num = reader.read_line(&mut line)?;
        if num == 0 {
            break;
        }

        // println!("{line}");
        line.clear();
    }

    Ok(())
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

pub fn read_party_ini_db() {
    // ? something about party members
    todo!(); // PrtIni.db
}

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

#[derive(Debug, Serialize)]
pub struct EventNpcRef {
    id: i32,
    event_id: i32,
    name: String,
}

pub fn read_event_npc_ref(source_path: &Path) -> Result<Vec<EventNpcRef>> {
    let f = File::open(source_path)?;
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(WINDOWS_1250))
            .build(f),
    );
    let mut npc_refs: Vec<EventNpcRef> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(";") {
                    continue;
                }
                println!("{line}");
            }
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(npc_refs)
}

fn read_multi_monster_db() {
    todo!();
}

pub fn read_party_level_db(source_path: &Path) -> Result<()> {
    // 5760
    // Divisors of number 5760: 1, 2, 3, 4, 5, 6, 8, 9, 10, 12, 15, 16, 18, 20, 24, 30, 32, 36, 40, 45, 48, 60, 64, 72, 80, 90, 96, 120, 128, 144, 160, 180, 192, 240, 288, 320, 360, 384, 480, 576, 640, 720, 960, 1152, 1440, 1920, 2880, 5760

    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    // let mut buffer: Vec<u8> = Vec::new();
    // reader.read_to_end(&mut buffer)?;
    // let dst = WINDOWS_1250.decode(&buffer);
    // println!("{:?}", buffer.len());
    // println!("{:?}", dst.0);
    // let pos = reader.seek(SeekFrom::Current(0))?;
    // println!("{file_len} {pos}");

    const COUNTER_SIZE: u8 = 0;
    const PROPERTY_ITEM_SIZE: i32 = 180 * 4;
    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;

    // let pos = reader.seek(SeekFrom::Current(0))?;

    println!("{elements}");

    for i in 0..16 {
        let mut buffer = [0u8; 360];
        reader.read_exact(&mut buffer)?;

        let cursor = Cursor::new(&buffer);
        println!("{i} {cursor:?}");

        let dst = EUC_KR.decode(&buffer);
        println!("{:?}", dst.0);
    }

    Ok(())
}

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

    for i in 0..elements {
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
    let mut expected_elements = 0;
    let space_for_elements =
        (((file_len - counter_size as u64) as f64) / property_item_size as f64).floor();
    let space_for_elements: i32 = space_for_elements as i32;

    if counter_size > 0 {
        expected_elements = reader.read_i32::<LittleEndian>()?;
    } else {
        expected_elements = space_for_elements;
    }
    if expected_elements != space_for_elements {
        println!(
            "expected_elements: {expected_elements} / space_for_elements: {space_for_elements}"
        );
    }

    Ok(space_for_elements)
}
