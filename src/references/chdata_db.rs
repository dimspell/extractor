use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ChData {
    pub magic: String,
    pub values: Vec<u16>,
    pub counts: Vec<u32>,
    pub total: u32,
}

pub fn read_chdata(path: &Path) -> std::io::Result<ChData> {
    let mut file = File::open(path)?;
    let mut reader = BufReader::new(file);

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

    Ok(ChData {
        magic,
        values,
        counts,
        total,
    })
}
