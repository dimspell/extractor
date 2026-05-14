use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::references::extractor::Extractor;
use dispel_macros::{Extractor, RecordPatcher};
use serde::{Deserialize, Serialize};

/// Stores information about the initial attributes during character creation.
///
/// Reads file: `CharacterInGame/ChData.db`
#[derive(Debug, Clone, Default, Serialize, Deserialize, Extractor, RecordPatcher)]
#[extractor(property_item_size = 84)]
#[patcher(filename = "ChData.db")]
pub struct ChData {
    /// Asset identifier string (unused).
    #[extractor(string(encoding = "WINDOWS-1250", size = 30))]
    pub unused_name: String,

    /// Strength attribute for Warrior character class
    #[extractor(primitive(type = "i16"))]
    pub warrior_strength: i16,
    /// Constitution attribute for Warrior character class
    #[extractor(primitive(type = "i16"))]
    pub warrior_constitution: i16,
    /// Wisdom attribute for Warrior character class
    #[extractor(primitive(type = "i16"))]
    pub warrior_wisdom: i16,
    /// Agility attribute for Warrior character class
    #[extractor(primitive(type = "i16"))]
    pub warrior_agility: i16,

    /// Strength attribute for Knight character class
    #[extractor(primitive(type = "i16"))]
    pub knight_strength: i16,
    /// Constitution attribute for Knight character class
    #[extractor(primitive(type = "i16"))]
    pub knight_constitution: i16,
    /// Wisdom attribute for Knight character class
    #[extractor(primitive(type = "i16"))]
    pub knight_wisdom: i16,
    /// Agility attribute for Knight character class
    #[extractor(primitive(type = "i16"))]
    pub knight_agility: i16,

    /// Strength attribute for Archer character class
    #[extractor(primitive(type = "i16"))]
    pub archer_strength: i16,
    /// Constitution attribute for Archer character class
    #[extractor(primitive(type = "i16"))]
    pub archer_constitution: i16,
    /// Wisdom attribute for Archer character class
    #[extractor(primitive(type = "i16"))]
    pub archer_wisdom: i16,
    /// Agility attribute for Archer character class
    #[extractor(primitive(type = "i16"))]
    pub archer_agility: i16,

    /// Strength attribute for Mage character class
    #[extractor(primitive(type = "i16"))]
    pub mage_strength: i16,
    /// Constitution attribute for Mage character class
    #[extractor(primitive(type = "i16"))]
    pub mage_constitution: i16,
    /// Wisdom attribute for Mage character class
    #[extractor(primitive(type = "i16"))]
    pub mage_wisdom: i16,
    /// Agility attribute for Mage character class
    #[extractor(primitive(type = "i16"))]
    pub mage_agility: i16,

    /// Unknown value, always zero.
    #[extractor(primitive(type = "i16"))]
    pub reseverd: i16,

    /// Extra attribute points during character creation for the Warrior class (unused)
    #[extractor(primitive(type = "i32"))]
    pub warrior_extra_points: i32,
    /// Extra attribute points during character creation for the Knight class (unused)
    #[extractor(primitive(type = "i32"))]
    pub knight_extra_points: i32,
    /// Extra attribute points during character creation for theArcher class (unused)
    #[extractor(primitive(type = "i32"))]
    pub archer_extra_points: i32,
    /// Extra attribute points during character creation for the Mage class (unused)
    #[extractor(primitive(type = "i32"))]
    pub mage_extra_points: i32,

    /// Extra attribute points received after leveling up (in-game) (unused)
    #[extractor(primitive(type = "i32"))]
    pub extra_points_per_level: i32,
}

pub fn read_chdata(source_path: &Path) -> std::io::Result<Vec<ChData>> {
    ChData::read_file(source_path)
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
        assert_eq!(records[0].unused_name, "Unused");
        assert_eq!(records[0].warrior_extra_points, 10);
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
