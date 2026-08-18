use dispel_macros::BinaryRecord;
use serde::{Deserialize, Serialize};

/// Event script record (save file format: 284 bytes each)
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct EventRecord {
    pub event_id: u32,
    pub unknown_1: u32,
    pub unknown_2: u32,
    #[binary_record(string(encoding = "WINDOWS-1250", size = 272))]
    pub script_name: String,
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
