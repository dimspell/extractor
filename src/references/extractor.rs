use std::io::{BufReader, BufWriter, Read, Result, Seek, Write};
use std::num::IntErrorKind;
use std::{fs::File, path::Path};

use byteorder::{LittleEndian, ReadBytesExt};
use encoding_rs::WINDOWS_1250;

pub trait Extractor: Sized {
    fn parse<R: Read + Seek>(reader: &mut R, len: u64) -> std::io::Result<Vec<Self>>;

    fn serialize<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()>;

    fn read_file(path: &Path) -> std::io::Result<Vec<Self>> {
        let file = File::open(path)?;
        let len = file.metadata()?.len();
        let mut reader = BufReader::new(file);
        Self::parse(&mut reader, len)
    }

    fn save_file(records: &[Self], path: &Path) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        Self::serialize(records, &mut writer)
    }
}

pub fn read_null_terminated_windows_1250(bytes: &[u8]) -> core::result::Result<String, String> {
    // Find the first null byte (or use a fixed length if no null terminator)
    let (_data, _) = bytes.split_last().ok_or("Empty input")?;
    let data_len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());

    // Decode the Windows-1250 portion before the null terminator
    let (decoded, _, had_errors) = WINDOWS_1250.decode(&bytes[..data_len]);

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

pub fn read_mutli_magic_db(source_path: &Path) -> Result<()> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 4;
    const PROPERTY_ITEM_SIZE: i32 = 90;
    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;

    let pos = reader.stream_position()?;
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

    Ok(())
}

pub fn read_mapper<R: Read>(
    reader: &mut R,
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
