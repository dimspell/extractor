use std::io::{BufReader, BufWriter, Read, Result, Seek, Write};
use std::num::IntErrorKind;
use std::{fs::File, path::Path};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::localization::TextEncoding;

pub trait Extractor: Sized {
    fn parse<R: Read + Seek>(reader: &mut R, len: u64) -> std::io::Result<Vec<Self>>;

    fn to_writer<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()>;

    fn read_file(path: &Path) -> std::io::Result<Vec<Self>> {
        let file = File::open(path)?;
        let len = file.metadata()?.len();
        let mut reader = BufReader::new(file);
        Self::parse(&mut reader, len)
    }

    fn save_file(records: &[Self], path: &Path) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        Self::to_writer(records, &mut writer)
    }
}

/// Decode a fixed-size, null-terminated byte field using `encoding`.
///
/// Bytes after the first NUL are ignored; a field with no NUL is decoded in
/// full. Invalid byte sequences for the declared encoding are reported as
/// errors rather than silently replaced.
pub fn read_null_terminated(
    bytes: &[u8],
    encoding: &TextEncoding,
) -> core::result::Result<String, String> {
    // Validate the input is non-empty (the original helper rejected empties).
    bytes.split_last().ok_or("Empty input")?;
    let data_len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());

    let (decoded, _, had_errors) = encoding.encoding_rs().decode(&bytes[..data_len]);
    if had_errors {
        return Err(format!("Invalid {} sequence", encoding.label()));
    }
    Ok(decoded.into_owned())
}

/// Backwards-compatible Windows-1250 specialisation of [`read_null_terminated`].
pub fn read_null_terminated_windows_1250(bytes: &[u8]) -> core::result::Result<String, String> {
    read_null_terminated(bytes, &TextEncoding::Windows1250)
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
