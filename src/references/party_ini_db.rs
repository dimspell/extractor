use byteorder::{LittleEndian, ReadBytesExt};
use serde::Serialize;
use std::fs::File;
use std::io::{BufReader, Read, Result};
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct PartyIniNpc {
    pub name: String,
    pub flags: u16,
    pub kind: u16,
    pub value: u32,
}

pub fn read_party_ini_db(source_path: &Path) -> Result<Vec<PartyIniNpc>> {
    let file = File::open(source_path)?;
    let mut reader = BufReader::new(file);
    let mut npcs = Vec::new();

    // The file is 224 bytes, which is 8 NPCs * 28 bytes each.
    // Each 28 byte record consists of:
    // - name: 20 bytes (null-terminated ASCII)
    // - flags: u16
    // - kind: u16
    // - value: u32
    for _ in 0..8 {
        let mut name_bytes = [0u8; 20];
        reader.read_exact(&mut name_bytes)?;

        // Find the first null byte to terminate the string
        let name = name_bytes
            .split(|&b| b == 0)
            .next()
            .map(|b| String::from_utf8_lossy(b).to_string())
            .unwrap_or_default();

        let flags = reader.read_u16::<LittleEndian>()?;
        let kind = reader.read_u16::<LittleEndian>()?;
        let value = reader.read_u32::<LittleEndian>()?;

        npcs.push(PartyIniNpc {
            name,
            flags,
            kind,
            value,
        });
    }

    Ok(npcs)
}
