use std::path::Path;

use crate::references::extractor::Extractor;
use dispel_macros::{Extractor, RecordLayout, RecordPatcher};
use serde::{Deserialize, Serialize};

/// Stores information about the initial attributes during character creation.
///
/// Reads file: `CharacterInGame/ChData.db`
#[derive(Debug, Clone, Default, Serialize, Deserialize, Extractor, RecordPatcher, RecordLayout)]
#[extractor(counter_size = 0, property_item_size = 84)]
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

    /// Reserved stat between class attributes and extra points (purpose unknown).
    #[extractor(primitive(type = "i16"))]
    pub reserved_stat: i16,

    /// Derived-stat bonus for the Warrior class: added to offense (STR-derived).
    #[extractor(primitive(type = "i32"))]
    pub warrior_offense_bonus: i32,
    /// Derived-stat bonus for the Knight class: added to defense (AGI-derived).
    #[extractor(primitive(type = "i32"))]
    pub knight_defense_bonus: i32,
    /// Derived-stat bonus for the Archer class: added to dodge_rate (CON-derived).
    #[extractor(primitive(type = "i32"))]
    pub archer_dodge_bonus: i32,
    /// Derived-stat bonus for the Archer class: added to hit_rate (CON-derived).
    #[extractor(primitive(type = "i32"))]
    pub archer_hit_bonus: i32,
    /// Derived-stat bonus for the Mage class: added to magic_power (WIS-derived).
    #[extractor(primitive(type = "i32"))]
    pub mage_magic_power_bonus: i32,
}

pub fn read_chdata(source_path: &Path) -> std::io::Result<Vec<ChData>> {
    ChData::read_file(source_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Returns an 84-byte zero-filled buffer representing one ChData record.
    fn empty_record() -> Vec<u8> {
        vec![0u8; 84]
    }

    #[test]
    fn parse_default_record() {
        let data = empty_record();
        let mut c = Cursor::new(data);
        let records = ChData::parse(&mut c, 84).unwrap();
        assert_eq!(records.len(), 1);
        let r = &records[0];
        assert_eq!(r.unused_name, "");
        assert_eq!(r.warrior_strength, 0);
        assert_eq!(r.reserved_stat, 0);
        assert_eq!(r.warrior_offense_bonus, 0);
        assert_eq!(r.mage_magic_power_bonus, 0);
    }

    #[test]
    fn parse_record_with_values() {
        let mut buf = empty_record();

        // unused_name at offset 0 (30 bytes, WINDOWS-1250)
        buf[..5].copy_from_slice(b"Test\0");
        // warrior_strength at offset 30 (i16)
        buf[30..32].copy_from_slice(&10i16.to_le_bytes());
        // knight_constitution at offset 40 (i16)
        buf[40..42].copy_from_slice(&20i16.to_le_bytes());
        // archer_wisdom at offset 50 (i16)
        buf[50..52].copy_from_slice(&30i16.to_le_bytes());
        // mage_agility at offset 60 (i16)
        buf[60..62].copy_from_slice(&40i16.to_le_bytes());
        // reserved_stat at offset 62 (i16, after 16 class attrs)
        buf[62..64].copy_from_slice(&99i16.to_le_bytes());
        // warrior_offense_bonus at offset 64 (i32)
        buf[64..68].copy_from_slice(&5i32.to_le_bytes());
        // mage_magic_power_bonus at offset 80 (i32)
        buf[80..84].copy_from_slice(&42i32.to_le_bytes());

        let mut c = Cursor::new(buf);
        let records = ChData::parse(&mut c, 84).unwrap();
        assert_eq!(records.len(), 1);
        let r = &records[0];
        assert_eq!(r.unused_name, "Test");
        assert_eq!(r.warrior_strength, 10);
        assert_eq!(r.knight_constitution, 20);
        assert_eq!(r.archer_wisdom, 30);
        assert_eq!(r.mage_agility, 40);
        assert_eq!(r.reserved_stat, 99);
        assert_eq!(r.warrior_offense_bonus, 5);
        assert_eq!(r.mage_magic_power_bonus, 42);
        // default-zero fields
        assert_eq!(r.warrior_constitution, 0);
        assert_eq!(r.knight_defense_bonus, 0);
    }

    #[test]
    fn serialize_round_trip() {
        let records = vec![ChData {
            unused_name: "Item".to_string(),
            warrior_strength: 10,
            knight_constitution: 20,
            archer_wisdom: 30,
            mage_agility: 40,
            reserved_stat: 99,
            warrior_offense_bonus: 5,
            mage_magic_power_bonus: 42,
            ..Default::default()
        }];

        let mut buf = Vec::new();
        ChData::to_writer(&records, &mut buf).unwrap();
        assert_eq!(buf.len(), 84);

        let mut c = Cursor::new(&buf[..]);
        let parsed = ChData::parse(&mut c, 84).unwrap();
        assert_eq!(parsed.len(), 1);
        let p = &parsed[0];
        assert_eq!(p.unused_name, "Item");
        assert_eq!(p.warrior_strength, 10);
        assert_eq!(p.knight_constitution, 20);
        assert_eq!(p.archer_wisdom, 30);
        assert_eq!(p.mage_agility, 40);
        assert_eq!(p.reserved_stat, 99);
        assert_eq!(p.warrior_offense_bonus, 5);
        assert_eq!(p.mage_magic_power_bonus, 42);
    }
}
