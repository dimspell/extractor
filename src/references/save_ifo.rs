// Save-slot metadata index extraction and serialization for Dispel RPG.

use crate::references::extractor::Extractor;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Total size of a `Save.ifo` file in bytes (6 × 32-byte slots + 32-byte tail).
pub const SAVE_IFO_SIZE: usize = 224;
/// Number of save slots described by the file.
pub const SLOT_COUNT: usize = 6;

/// Metadata index for the six save slots (`0.sav` … `5.sav`).
///
/// Reads file: `Save.ifo` (game root directory)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveIfo {
    /// Per-slot metadata; always exactly [`SLOT_COUNT`] entries.
    pub slots: Vec<SaveSlotInfo>,
    /// Unknown float; written as constant 1.4 before every save.
    pub unknown_float: f32,
    /// Key of this session's payload inside the `game.tmp` append-log.
    pub game_tmp_key: u32,
    /// Map/world id that was active when the game was last saved.
    pub map_id: u32,
    /// Unknown; observed zero. Reset by the game when starting a new game.
    pub unknown: u32,
    /// Element counts snapshotted for traversing `game.tmp` payloads.
    pub payload_counts: [u32; 4],
}

/// Metadata for a single save slot.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveSlotInfo {
    /// Never written by the game; zero in all known samples.
    pub reserved: [u8; 12],
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    /// Byte 0: slot occupied flag; bytes 1..3: padding (preserved verbatim).
    pub flags: [u8; 4],
}

impl SaveSlotInfo {
    pub fn is_occupied(&self) -> bool {
        self.flags[0] != 0
    }
}

impl SaveIfo {
    /// Parse a `Save.ifo` from binary data.
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        if data.len() != SAVE_IFO_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Save.ifo must be exactly {} bytes, got {} bytes",
                    SAVE_IFO_SIZE,
                    data.len()
                ),
            ));
        }

        let mut cursor = std::io::Cursor::new(data);

        let mut slots = Vec::with_capacity(SLOT_COUNT);
        for _ in 0..SLOT_COUNT {
            let mut reserved = [0u8; 12];
            cursor.read_exact(&mut reserved)?;
            let month = cursor.read_u32::<LittleEndian>()?;
            let day = cursor.read_u32::<LittleEndian>()?;
            let hour = cursor.read_u32::<LittleEndian>()?;
            let minute = cursor.read_u32::<LittleEndian>()?;
            let mut flags = [0u8; 4];
            cursor.read_exact(&mut flags)?;
            slots.push(SaveSlotInfo {
                reserved,
                month,
                day,
                hour,
                minute,
                flags,
            });
        }

        let unknown_float = cursor.read_f32::<LittleEndian>()?;
        let game_tmp_key = cursor.read_u32::<LittleEndian>()?;
        let map_id = cursor.read_u32::<LittleEndian>()?;
        let unknown = cursor.read_u32::<LittleEndian>()?;
        let mut payload_counts = [0u32; 4];
        for count in payload_counts.iter_mut() {
            *count = cursor.read_u32::<LittleEndian>()?;
        }

        if slots.len() != SLOT_COUNT {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Save.ifo must contain exactly {} slot records, got {}",
                    SLOT_COUNT,
                    slots.len()
                ),
            ));
        }

        Ok(SaveIfo {
            slots,
            unknown_float,
            game_tmp_key,
            map_id,
            unknown,
            payload_counts,
        })
    }

    /// Serialize to its binary representation.
    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        if self.slots.len() != SLOT_COUNT {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Save.ifo must contain exactly {} slot records, got {}",
                    SLOT_COUNT,
                    self.slots.len()
                ),
            ));
        }
        for slot in &self.slots {
            writer.write_all(&slot.reserved)?;
            writer.write_u32::<LittleEndian>(slot.month)?;
            writer.write_u32::<LittleEndian>(slot.day)?;
            writer.write_u32::<LittleEndian>(slot.hour)?;
            writer.write_u32::<LittleEndian>(slot.minute)?;
            writer.write_all(&slot.flags)?;
        }
        writer.write_f32::<LittleEndian>(self.unknown_float)?;
        writer.write_u32::<LittleEndian>(self.game_tmp_key)?;
        writer.write_u32::<LittleEndian>(self.map_id)?;
        writer.write_u32::<LittleEndian>(self.unknown)?;
        for count in &self.payload_counts {
            writer.write_u32::<LittleEndian>(*count)?;
        }
        Ok(())
    }
}

impl Extractor for SaveIfo {
    fn parse<R: Read + std::io::Seek>(reader: &mut R, _len: u64) -> std::io::Result<Vec<Self>> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Ok(vec![SaveIfo::parse(&data)?])
    }

    fn to_writer<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
        if records.len() != 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "SaveIfo can only serialize one record at a time",
            ));
        }
        records[0].write_to(writer)
    }
}
