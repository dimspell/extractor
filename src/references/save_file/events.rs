use byteorder::{LittleEndian, ReadBytesExt};
use dispel_macros::BinaryRecord;
use serde::{Deserialize, Serialize};
use std::io::Read;

pub(super) const EVENT_COUNT: usize = 2_251;
pub(super) const EVENT_RECORD_SIZE: usize = 284;
const POST_EVENTS_BLOCK_A_SIZE: usize = 12;
const POST_EVENTS_RECORD_SIZE: usize = 24;
const POST_EVENTS_BLOCK_B_SIZE: usize = 56;

/// Event script record (save file format: 284 bytes each)
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct EventRecord {
    pub event_id: u32,
    pub unknown_1: u32,
    pub unknown_2: u32,
    #[binary_record(string(encoding = "WINDOWS-1250", size = 272))]
    pub script_name: String,
}

pub(super) fn read_events<R: Read>(reader: &mut R) -> std::io::Result<Vec<EventRecord>> {
    let mut events = Vec::with_capacity(EVENT_COUNT);
    for _ in 0..EVENT_COUNT {
        let mut data = [0u8; EVENT_RECORD_SIZE];
        reader.read_exact(&mut data)?;
        events.push(EventRecord::parse(&data)?);
    }
    Ok(events)
}

/// Unknown data block between events and journal sections.
///
/// Structure: fixed 12 bytes + counter-prefixed 24-byte records + fixed 56 bytes.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostEventsData {
    /// Unknown fixed block (12 bytes).
    pub block_a: Vec<u8>,
    /// Unknown records (counter × 24 bytes each).
    pub records: Vec<u8>,
    /// Unknown fixed block (56 bytes).
    pub block_b: Vec<u8>,
}

impl PostEventsData {
    pub(super) fn read_from<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut block_a = vec![0u8; POST_EVENTS_BLOCK_A_SIZE];
        reader.read_exact(&mut block_a)?;

        let count = reader.read_u32::<LittleEndian>()? as usize;
        let mut records = vec![0u8; count * POST_EVENTS_RECORD_SIZE];
        reader.read_exact(&mut records)?;

        let mut block_b = vec![0u8; POST_EVENTS_BLOCK_B_SIZE];
        reader.read_exact(&mut block_b)?;

        Ok(Self {
            block_a,
            records,
            block_b,
        })
    }
}
