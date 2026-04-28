use std::path::Path;

use crate::references::enums::{MonsterAiType, PropertyFlag};
use crate::references::extractor::Extractor;
use dispel_macros::Extractor;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

/// Monster.db - Monster Statistics
///
/// Stores base stats, attacks, and defense values for monsters.
///
/// Reads file: `MonsterInGame/Monster.db`
///
/// # Binary Format
///
/// - **Encoding**: Little-endian for all numeric values
/// - **Text Encoding**: EUC-KR for `name` field (24 bytes, null-padded)
/// - **Record Size**: 160 bytes (24 + 34 × 4)
/// - **Header**: None; record count = file size / 160
///
/// ```text
/// +--------------------------------------+
/// | Monster.db - Monster Statistics      |
/// +--------------------------------------+
/// | Encoding: Binary (Little-Endian)     |
/// | Text Encoding: EUC-KR                |
/// | Record Size: 160 bytes               |
/// | No header - count from file size     |
/// +--------------------------------------+
/// | [Record 1] - 160 bytes              |
/// | - name: 24 bytes (EUC-KR, null-padded)|
/// | - health_points_max: i32             |
/// | - health_points_min: i32             |
/// | - mana_points_max: i32              |
/// | - mana_points_min: i32              |
/// | - walk_speed: i32                    |
/// | - to_hit_max: i32                    |
/// | - to_hit_min: i32                    |
/// | - to_dodge_max: i32                 |
/// | - to_dodge_min: i32                 |
/// | - offense_max: i32                   |
/// | - offense_min: i32                   |
/// | - defense_max: i32                  |
/// | - defense_min: i32                  |
/// | - magic_attack_max: i32              |
/// | - magic_attack_min: i32              |
/// | - is_undead: i32 (PropertyFlag)     |
/// | - has_blood: i32 (PropertyFlag)     |
/// | - ai_type: i32 (MonsterAiType)      |
/// | - exp_gain_max: i32                  |
/// | - exp_gain_min: i32                  |
/// | - gold_drop_max: i32                 |
/// | - gold_drop_min: i32                 |
/// | - detection_sight_size: i32         |
/// | - distance_range_size: i32          |
/// | - known_spell_slot1: i32            |
/// | - known_spell_slot2: i32            |
/// | - known_spell_slot3: i32            |
/// | - is_oversize: i32                  |
/// | - magic_level: i32                   |
/// | - special_attack: i32              |
/// | - special_attack_chance: i32       |
/// | - special_attack_duration: i32     |
/// | - boldness: i32                     |
/// | - attack_speed: i32                 |
/// +--------------------------------------+
/// | [Record 2]                           |
/// | ... (same structure) ...             |
/// +--------------------------------------+
/// ```
///
/// # Field Categories
///
/// - **Identification**: `name` (24 bytes EUC-KR, null-padded), `id` (auto-generated from position)
/// - **Vital Stats**: HP/MP max/min ranges
/// - **Combat Stats**: accuracy (`to_hit`), evasion (`to_dodge`), offense, defense
/// - **Magic Stats**: magic attack range, spell slots
/// - **Properties**: `is_undead` (holy weakness), `has_blood` (gore), `ai_type`, `is_oversize`
/// - **Rewards**: EXP (`exp_gain`) and gold drop ranges
/// - **Behavior**: `detection_sight_size` (aggro), `distance_range_size` (attack range), `attack_speed`
/// - **Special**: `special_attack` with `chance` and `duration`
///
/// # Special Values
///
/// - `is_undead`: `PropertyFlag::Absent` (0) = normal, `Present` (1) = undead
/// - `has_blood`: `PropertyFlag::Absent` (0) = no blood, `Present` (1) = bleeds on hit
/// - `ai_type`: 1 = melee, 2 = archer, 3 = caster, 5 = passive
/// - `to_dodge_max/min`: usually both = 10
/// - `magic_level`: usually = 1
/// - `boldness`: usually = 10 (courage/retreat threshold)
/// - `is_oversize`: 1 for large monsters (dragons, etc.)
///
/// # File Purpose
///
/// Defines all monster types with complete combat statistics, behavior patterns,
/// and reward systems. Used by game engine for monster spawning and combat AI.
#[derive(Debug, Clone, Serialize, Deserialize, Default, Extractor)]
#[extractor(property_item_size = 160, counter_size = 0)]
pub struct Monster {
    /// Unique monster archetype tracking ID.
    #[extractor(index)]
    pub id: i32,
    /// Localization name for the monster encounter.
    #[extractor(string(encoding = "EUC-KR", size = 24))]
    pub name: String,
    /// Maximum HP ceiling.
    #[extractor(primitive(type = "i32"))]
    pub health_points_max: i32,
    /// Minimum HP floor.
    #[extractor(primitive(type = "i32"))]
    pub health_points_min: i32,
    /// Maximum MP limit.
    #[extractor(primitive(type = "i32"))]
    pub mana_points_max: i32,
    /// Minimum MP limit.
    #[extractor(primitive(type = "i32"))]
    pub mana_points_min: i32,
    /// Baseline tiles moved per tick.
    #[extractor(primitive(type = "i32"))]
    pub walk_speed: i32,
    /// Upper bound of accuracy.
    #[extractor(primitive(type = "i32"))]
    pub to_hit_max: i32,
    /// Lower bound of accuracy.
    #[extractor(primitive(type = "i32"))]
    pub to_hit_min: i32,
    /// Upper bound for evasion rate.
    #[extractor(primitive(type = "i32"))]
    pub to_dodge_max: i32,
    /// Lower bound for evasion rate.
    #[extractor(primitive(type = "i32"))]
    pub to_dodge_min: i32,
    /// Maximum physical damage.
    #[extractor(primitive(type = "i32"))]
    pub offense_max: i32,
    /// Minimum physical damage.
    #[extractor(primitive(type = "i32"))]
    pub offense_min: i32,
    /// Upper bound armor class.
    #[extractor(primitive(type = "i32"))]
    pub defense_max: i32,
    /// Lower bound armor class.
    #[extractor(primitive(type = "i32"))]
    pub defense_min: i32,
    /// Maximum magical intensity.
    #[extractor(primitive(type = "i32"))]
    pub magic_attack_max: i32,
    /// Minimum magical intensity.
    #[extractor(primitive(type = "i32"))]
    pub magic_attack_min: i32,
    /// Flag indicating undead affiliation (holy weakness).
    #[extractor(enum_from_i32(type = "PropertyFlag"))]
    pub is_undead: PropertyFlag,
    /// Controls visual gore upon hit.
    #[extractor(enum_from_i32(type = "PropertyFlag"))]
    pub has_blood: PropertyFlag,
    /// Combat behavioral script type.
    #[extractor(enum_from_i32(type = "MonsterAiType"))]
    pub ai_type: MonsterAiType,
    /// High roll for experience points.
    #[extractor(primitive(type = "i32"))]
    pub exp_gain_max: i32,
    /// Low roll for experience points.
    #[extractor(primitive(type = "i32"))]
    pub exp_gain_min: i32,
    /// Maximum gold drop.
    #[extractor(primitive(type = "i32"))]
    pub gold_drop_max: i32,
    /// Minimum gold drop.
    #[extractor(primitive(type = "i32"))]
    pub gold_drop_min: i32,
    /// Aggro radius in tiles.
    #[extractor(primitive(type = "i32"))]
    pub detection_sight_size: i32,
    /// Maximum engage distance for attacks.
    #[extractor(primitive(type = "i32"))]
    pub distance_range_size: i32,
    /// Primary magic spell index.
    #[extractor(primitive(type = "i32"))]
    pub known_spell_slot1: i32,
    /// Secondary magic spell index.
    #[extractor(primitive(type = "i32"))]
    pub known_spell_slot2: i32,
    /// Tertiary magic spell index.
    #[extractor(primitive(type = "i32"))]
    pub known_spell_slot3: i32,
    /// Controls collision size overlay.
    #[extractor(primitive(type = "i32"))]
    pub is_oversize: i32,
    /// Potency tracking for enemy spellcasting.
    #[extractor(primitive(type = "i32"))]
    pub magic_level: i32,
    /// Identifier for unique monster skills.
    #[extractor(primitive(type = "i32"))]
    pub special_attack: i32,
    /// Percentage likelihood to cast special.
    #[extractor(primitive(type = "i32"))]
    pub special_attack_chance: i32,
    /// Length special effect lingers.
    #[extractor(primitive(type = "i32"))]
    pub special_attack_duration: i32,
    /// Courage metric defining retreat threshold.
    #[extractor(primitive(type = "i32"))]
    pub boldness: i32,
    /// Delay ticks between swings.
    #[extractor(primitive(type = "i32"))]
    pub attack_speed: i32,
}

pub fn read_monster_db(source_path: &Path) -> std::io::Result<Vec<Monster>> {
    Monster::read_file(source_path)
}

pub fn save_monsters(conn: &mut Connection, monsters: &[Monster]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_monster.sql"))?;
        for monster in monsters {
            stmt.execute(params![
                monster.id,
                monster.name,
                monster.health_points_max,
                monster.health_points_min,
                monster.mana_points_max,
                monster.mana_points_min,
                monster.walk_speed,
                monster.to_hit_max,
                monster.to_hit_min,
                monster.to_dodge_max,
                monster.to_dodge_min,
                monster.offense_max,
                monster.offense_min,
                monster.defense_max,
                monster.defense_min,
                monster.magic_attack_max,
                monster.magic_attack_min,
                i32::from(monster.is_undead),
                i32::from(monster.has_blood),
                i32::from(monster.ai_type),
                monster.exp_gain_max,
                monster.exp_gain_min,
                monster.gold_drop_max,
                monster.gold_drop_min,
                monster.detection_sight_size,
                monster.distance_range_size,
                monster.known_spell_slot1,
                monster.known_spell_slot2,
                monster.known_spell_slot3,
                monster.is_oversize,
                monster.magic_level,
                monster.special_attack,
                monster.special_attack_chance,
                monster.special_attack_duration,
                monster.boldness,
                monster.attack_speed,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

impl std::fmt::Display for Monster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Monster({} - {} HP)", self.id, self.health_points_max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::references::enums::{MonsterAiType, PropertyFlag};
    use std::io::Cursor;

    fn monster_bytes(name: &str, stats: &[i32; 34]) -> Vec<u8> {
        let mut buf = Vec::with_capacity(160);
        let mut name_buf = [0u8; 24];
        let (encoded, _, _) = encoding_rs::EUC_KR.encode(name);
        let len = encoded.len().min(24);
        name_buf[..len].copy_from_slice(&encoded[..len]);
        buf.extend_from_slice(&name_buf);
        for &v in stats {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        buf
    }

    #[test]
    fn parse_single_record() {
        #[rustfmt::skip]
        let stats: [i32; 34] = [
            100, 80,  // hp max/min
            50,  30,  // mp max/min
            5,        // walk_speed
            70,  60,  // to_hit max/min
            10,  10,  // to_dodge max/min
            20,  15,  // offense max/min
            12,  8,   // defense max/min
            5,   3,   // magic_attack max/min
            0,        // is_undead (Absent)
            1,        // has_blood (Present)
            1,        // ai_type (Melee)
            30,  20,  // exp max/min
            10,  5,   // gold max/min
            9,   1,   // detection_sight, distance_range
            0,   0,   0, // spell slots
            0,        // is_oversize
            1,        // magic_level
            0,   0,   0, // special_attack / chance / duration
            10,       // boldness
            4,        // attack_speed
        ];
        let data = monster_bytes("Goblin", &stats);
        assert_eq!(data.len(), 160);

        let mut cursor = Cursor::new(data);
        let monsters = Monster::parse(&mut cursor, 160).unwrap();

        assert_eq!(monsters.len(), 1);
        let m = &monsters[0];
        assert_eq!(m.id, 0);
        assert_eq!(m.name, "Goblin");
        assert_eq!(m.health_points_max, 100);
        assert_eq!(m.health_points_min, 80);
        assert_eq!(m.is_undead, PropertyFlag::Absent);
        assert_eq!(m.has_blood, PropertyFlag::Present);
        assert_eq!(m.ai_type, MonsterAiType::Aggressive);
        assert_eq!(m.attack_speed, 4);
    }

    #[test]
    fn parse_two_records() {
        let stats = [0i32; 34];
        let mut data = monster_bytes("Rat", &stats);
        data.extend(monster_bytes("Dragon", &stats));
        assert_eq!(data.len(), 320);

        let mut cursor = Cursor::new(data);
        let monsters = Monster::parse(&mut cursor, 320).unwrap();

        assert_eq!(monsters.len(), 2);
        assert_eq!(monsters[0].name, "Rat");
        assert_eq!(monsters[1].name, "Dragon");
        assert_eq!(monsters[0].id, 0);
        assert_eq!(monsters[1].id, 1);
    }

    #[test]
    fn parse_empty() {
        let mut cursor = Cursor::new(b"" as &[u8]);
        let monsters = Monster::parse(&mut cursor, 0).unwrap();
        assert!(monsters.is_empty());
    }

    #[test]
    fn serialize_round_trip() {
        let stats = [0i32; 34];
        let data = monster_bytes("Goblin", &stats);
        let mut c = Cursor::new(&data[..]);
        let records = Monster::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        Monster::to_writer(&records, &mut out).unwrap();
        assert_eq!(out, data);
    }
}
