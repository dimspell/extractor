use std::io::{BufReader, BufWriter, Read, Result, Seek, Write};
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
    s.parse::<i32>().ok()
}

pub fn read_mapper<R: Read>(
    reader: &mut R,
    file_len: u64,
    counter_size: u8,
    property_item_size: i32,
) -> Result<i32> {
    if property_item_size <= 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid property item size: {property_item_size}"),
        ));
    }

    let usable_len = file_len.saturating_sub(counter_size as u64);
    let space_for_elements = (usable_len as f64 / property_item_size as f64).floor();
    if !space_for_elements.is_finite() || space_for_elements > i32::MAX as f64 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "record count exceeds the supported range",
        ));
    }
    let space_for_elements = space_for_elements as i32;

    // Consume the on-disk element counter when present. The authoritative
    // count is derived from the file size below, so a mismatch is ignored.
    if counter_size > 0 {
        let _ = reader.read_i32::<LittleEndian>()?;
    }

    Ok(space_for_elements)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_mapper_handles_files_shorter_than_the_counter() {
        // A truncated file must produce an error, never a wrapped-around count
        // that a generated parser would turn into a huge allocation.
        let mut short = std::io::Cursor::new(vec![0u8; 3]);
        assert!(
            read_mapper(&mut short, 3, 4, 948).is_err(),
            "a file shorter than its record counter must be rejected"
        );

        let mut empty = std::io::Cursor::new(Vec::new());
        assert!(read_mapper(&mut empty, 0, 4, 948).is_err());

        // With no counter, an empty file legitimately contains zero records.
        let mut empty = std::io::Cursor::new(Vec::new());
        assert_eq!(read_mapper(&mut empty, 0, 0, 948).unwrap(), 0);
    }

    #[test]
    fn test_read_mapper_rejects_non_positive_item_size() {
        let mut data = std::io::Cursor::new(vec![0u8; 8]);
        assert!(read_mapper(&mut data, 8, 0, 0).is_err());
        assert!(read_mapper(&mut data, 8, 0, -1).is_err());
    }
}
