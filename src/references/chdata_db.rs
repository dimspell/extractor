use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::references::extractor::Extractor;

// ===========================================================================
// CHDATA.DB FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | ChData.db - Character Statistics      |
// +--------------------------------------+
// | Encoding: Binary (Little-Endian)     |
// | Record Size: 84 bytes                |
// | Single-record file                  |
// +--------------------------------------+
// | [Header]                            |
// | - magic: 4 bytes ("Item")            |
// | - padding: 26 bytes                 |
// +--------------------------------------+
// | [Data Section]                      |
// | - values: 16 × u16 (32 bytes)        |
// | - padding: 2 bytes                  |
// | - counts: 4 × u32 (16 bytes)         |
// | - total: u32 (4 bytes)               |
// +--------------------------------------+
//
// ===========================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChData {
    pub magic: String,
    pub values: Vec<u16>,
    pub counts: Vec<u32>,
    pub total: u32,
}

/// Stores global character statistics, counts, or internal game state properties.
///
/// Reads file: `CharacterInGame/ChData.db`
/// # File Format: `CharacterInGame/ChData.db`
///
/// Binary file, little-endian. Fixed-size single-record file:
/// - Bytes 0–3   : magic signature (`Item` ASCII)
/// - Bytes 4–29  : 26 bytes zero-padding (seek to offset 0x1E)
/// - Bytes 30–61 : 16 × u16 values
/// - Bytes 62–63 : 2 bytes padding (align to 0x40)
/// - Bytes 64–79 : 4 × u32 counts
/// - Bytes 80–83 : u32 total
impl Extractor for ChData {
    fn parse<R: Read + Seek>(reader: &mut R, _len: u64) -> std::io::Result<Vec<Self>> {
        // Read magic "Item"
        let mut magic_buf = [0u8; 4];
        reader.read_exact(&mut magic_buf)?;
        let magic = String::from_utf8_lossy(&magic_buf).to_string();

        // Skip padding to 0x1E (30 bytes total from start: 4 magic + 26 padding)
        reader.seek(SeekFrom::Start(30))?;

        // Read 16 u16s
        let mut values = Vec::with_capacity(16);
        for _ in 0..16 {
            values.push(reader.read_u16::<LittleEndian>()?);
        }

        // Skip padding (2 bytes) to 0x40 (64 bytes from start)
        // 30 bytes + 16*2 bytes = 62 bytes. Need 2 bytes more to reach 64.
        reader.seek(SeekFrom::Current(2))?;

        // Read 4 u32s (counts of 5)
        let mut counts = Vec::with_capacity(4);
        for _ in 0..4 {
            counts.push(reader.read_u32::<LittleEndian>()?);
        }

        // Read total (value 10)
        let total = reader.read_u32::<LittleEndian>()?;

        Ok(vec![ChData {
            magic,
            values,
            counts,
            total,
        }])
    }

    fn to_writer<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
        if records.is_empty() {
            return Ok(());
        }
        let record = &records[0];

        let mut magic_buf = [0u8; 4];
        let bytes = record.magic.as_bytes();
        let len = std::cmp::min(bytes.len(), 4);
        magic_buf[..len].copy_from_slice(&bytes[..len]);
        writer.write_all(&magic_buf)?;

        // Padding to 30 bytes
        writer.write_all(&[0u8; 26])?;

        for &val in &record.values {
            writer.write_u16::<LittleEndian>(val)?;
        }

        // Padding of 2 bytes
        writer.write_all(&[0u8; 2])?;

        for &count in &record.counts {
            writer.write_u32::<LittleEndian>(count)?;
        }

        writer.write_u32::<LittleEndian>(record.total)?;

        Ok(())
    }
}

pub fn read_chdata(path: &Path) -> std::io::Result<Vec<ChData>> {
    ChData::read_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn chdata_bytes(values: &[u16; 16], counts: &[u32; 4], total: u32) -> Vec<u8> {
        let mut buf = Vec::with_capacity(84);
        buf.extend_from_slice(b"Item"); // magic (4 bytes)
        buf.extend(vec![0u8; 26]); // padding to offset 30
        for &v in values {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        buf.extend(vec![0u8; 2]); // padding to offset 64
        for &c in counts {
            buf.extend_from_slice(&c.to_le_bytes());
        }
        buf.extend_from_slice(&total.to_le_bytes());
        assert_eq!(buf.len(), 84);
        buf
    }

    #[test]
    fn parse_record() {
        let values = [10u16, 20, 30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let counts = [5u32, 3, 1, 2];
        let data = chdata_bytes(&values, &counts, 100);

        let mut c = Cursor::new(data);
        let records = ChData::parse(&mut c, 84).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].magic, "Item");
        assert_eq!(records[0].values[0], 10);
        assert_eq!(records[0].values[1], 20);
        assert_eq!(records[0].values[2], 30);
        assert_eq!(records[0].counts, vec![5, 3, 1, 2]);
        assert_eq!(records[0].total, 100);
    }

    #[test]
    fn serialize_round_trip() {
        let values = [10u16, 20, 30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let counts = [5u32, 3, 1, 2];
        let data = chdata_bytes(&values, &counts, 100);
        let mut c = Cursor::new(&data[..]);
        let records = ChData::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        ChData::to_writer(&records, &mut out).unwrap();
        assert_eq!(out, data);
    }
}
