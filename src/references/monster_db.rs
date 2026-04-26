use std::io::{Read, Seek, Write};
use std::path::Path;

use crate::references::enums::{MonsterAiType, PropertyFlag};
use crate::references::extractor::{read_mapper, Extractor};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use encoding_rs::EUC_KR;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

// ===========================================================================
// MONSTER.DB FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | Monster.db - Monster Statistics      |
// +--------------------------------------+
// | Encoding: Binary (Little-Endian)     |
// | Text Encoding: EUC-KR                |
// | Record Size: 160 bytes               |
// | No header - count from file size     |
// +--------------------------------------+
// | [Record 1]                           |
// | - name: 24 bytes (EUC-KR, null-padded)|
// | - health_points_max: i32             |
// | - health_points_min: i32             |
// | - mana_points_max: i32              |
// | - mana_points_min: i32              |
// | - walk_speed: i32                    |
// | - to_hit_max: i32                    |
// | - to_hit_min: i32                    |
// | - to_dodge_max: i32                 |
// | - to_dodge_min: i32                 |
// | - offense_max: i32                   |
// | - offense_min: i32                   |
// | - defense_max: i32                  |
// | - defense_min: i32                  |
// | - magic_attack_max: i32              |
// | - magic_attack_min: i32              |
// | - is_undead: i32 (0/1)               |
// | - has_blood: i32 (0/1)               |
// | - ai_type: i32 (behavior enum)       |
// | - exp_gain_max: i32                  |
// | - exp_gain_min: i32                  |
// | - gold_drop_max: i32                 |
// | - gold_drop_min: i32                 |
// | - detection_sight_size: i32         |
// | - distance_range_size: i32          |
// | - known_spell_slot1: i32            |
// | - known_spell_slot2: i32            |
// | - known_spell_slot3: i32            |
// | - is_oversize: i32                  |
// | - magic_level: i32                   |
// | - special_attack: i32              |
// | - special_attack_chance: i32       |
// | - special_attack_duration: i32     |
// | - boldness: i32                     |
// | - attack_speed: i32                 |
// +--------------------------------------+
// | [Record 2]                           |
// | ... (same structure) ...             |
// +--------------------------------------+
//
// FIELD CATEGORIES:
// - Identification: name (24 bytes EUC-KR)
// - Vital Stats: HP/MP max/min ranges
// - Combat Stats: accuracy, evasion, offense, defense
// - Magic Stats: magic attack range, spells
// - Properties: undead, blood, AI type, size
// - Rewards: EXP and gold drop ranges
// - Behavior: detection range, attack range, AI
// - Special: special attacks with chance/duration
//
// SPECIAL VALUES:
// - is_undead: 0=normal, 1=undead (holy weakness)
// - has_blood: 0=no blood, 1=bleeds on hit
// - ai_type: 1=melee, 2=archer, 3=caster, 5=passive
// - to_dodge_max/min: usually both = 10
// - magic_level: usually = 1
// - boldness: usually = 10
// - is_oversize: 1 for large monsters (dragons, etc.)
//
// FILE PURPOSE:
// Defines all monster types with complete combat statistics, behavior patterns,
// and reward systems. Used by game engine for monster spawning and combat AI.
//
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Monster {
    /// Unique monster archetype tracking ID.
    pub id: i32,
    /// Localization name for the monster encounter.
    pub name: String,
    /// Maximum HP ceiling.
    pub health_points_max: i32,
    /// Minimum HP floor.
    pub health_points_min: i32,
    /// Maximum MP limit.
    pub mana_points_max: i32,
    /// Minimum MP limit.
    pub mana_points_min: i32,
    /// Baseline tiles moved per tick.
    pub walk_speed: i32,
    /// Upper bound of accuracy.
    pub to_hit_max: i32,
    /// Lower bound of accuracy.
    pub to_hit_min: i32,
    /// Upper bound for evasion rate.
    pub to_dodge_max: i32,
    /// Lower bound for evasion rate.
    pub to_dodge_min: i32,
    /// Maximum physical damage.
    pub offense_max: i32,
    /// Minimum physical damage.
    pub offense_min: i32,
    /// Upper bound armor class.
    pub defense_max: i32,
    /// Lower bound armor class.
    pub defense_min: i32,
    /// Maximum magical intensity.
    pub magic_attack_max: i32,
    /// Minimum magical intensity.
    pub magic_attack_min: i32,
    /// Flag indicating undead affiliation (holy weakness).
    pub is_undead: PropertyFlag,
    /// Controls visual gore upon hit.
    pub has_blood: PropertyFlag,
    /// Combat behavioral script type.
    pub ai_type: MonsterAiType,
    /// High roll for experience points.
    pub exp_gain_max: i32,
    /// Low roll for experience points.
    pub exp_gain_min: i32,
    /// Maximum gold drop.
    pub gold_drop_max: i32,
    /// Minimum gold drop.
    pub gold_drop_min: i32,
    /// Aggro radius in tiles.
    pub detection_sight_size: i32,
    /// Maximum engage distance for attacks.
    pub distance_range_size: i32,
    /// Primary magic spell index.
    pub known_spell_slot1: i32,
    /// Secondary magic spell index.
    pub known_spell_slot2: i32,
    /// Tertiary magic spell index.
    pub known_spell_slot3: i32,
    /// Controls collision size overlay.
    pub is_oversize: i32,
    /// Potency tracking for enemy spellcasting.
    pub magic_level: i32,
    /// Identifier for unique monster skills.
    pub special_attack: i32,
    /// Percentage likelihood to cast special.
    pub special_attack_chance: i32,
    /// Length special effect lingers.
    pub special_attack_duration: i32,
    /// Courage metric defining retreat threshold.
    pub boldness: i32,
    /// Delay ticks between swings.
    pub attack_speed: i32,
}

/// Stores base stats, attacks, and defense values for monsters.
///
/// Reads file: `MonsterInGame/Monster.db`
/// # File Format: `MonsterInGame/Monster.db`
///
/// Binary file, little-endian. No header (record count derived from file size).
/// Each record is exactly `40 × 4 = 160` bytes:
/// - `name`   : 24 bytes, null-padded, EUC-KR
/// - Stats    : 36 × i32 fields (HP max/min, MP max/min, speed, to-hit max/min,
///   dodge max/min, offense max/min, defense max/min, magic atk max/min,
///   undead, blood, AI type, EXP max/min, gold max/min,
///   sight, range, spell slots ×3, oversize, magic level,
///   special atk / chance / duration, boldness, attack speed)
impl Extractor for Monster {
    fn parse<R: Read + Seek>(reader: &mut R, len: u64) -> std::io::Result<Vec<Self>> {
        const COUNTER_SIZE: u8 = 0;
        const PROPERTY_ITEM_SIZE: i32 = 40 * 4; // Each object has 160 bytes (name - 24 bytes and 34 i32 fields)

        let elements = read_mapper(reader, len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
        let mut monsters: Vec<Monster> = Vec::with_capacity(elements as usize);

        for i in 0..elements {
            let mut buffer = [0u8; 24];
            reader.read_exact(&mut buffer)?;
            let dst = EUC_KR.decode(&buffer);
            let stripped: String = dst.0.chars().filter(|&c| c != '\0').collect();
            let name = stripped.trim();

            let health_points_max = reader.read_i32::<LittleEndian>()?;
            let health_points_min = reader.read_i32::<LittleEndian>()?;
            let mana_points_max = reader.read_i32::<LittleEndian>()?;
            let mana_points_min = reader.read_i32::<LittleEndian>()?;

            let walk_speed = reader.read_i32::<LittleEndian>()?;

            let to_hit_max = reader.read_i32::<LittleEndian>()?;
            let to_hit_min = reader.read_i32::<LittleEndian>()?;

            let to_dodge_max = reader.read_i32::<LittleEndian>()?; // always = 10
            let to_dodge_min = reader.read_i32::<LittleEndian>()?; // always = 10

            let offense_max = reader.read_i32::<LittleEndian>()?;
            let offense_min = reader.read_i32::<LittleEndian>()?;

            let defense_max = reader.read_i32::<LittleEndian>()?;
            let defense_min = reader.read_i32::<LittleEndian>()?;

            let magic_attack_max = reader.read_i32::<LittleEndian>()?; // max
            let magic_attack_min = reader.read_i32::<LittleEndian>()?; // min

            let is_undead_raw = reader.read_i32::<LittleEndian>()?; // "0 or 1"
            let has_blood_raw = reader.read_i32::<LittleEndian>()?; // "0 or 1, golem is not alive and not undead"
            let ai_type_raw = reader.read_i32::<LittleEndian>()?; // "goblin and chicken = 1,archers = 2, worm bot no zombie =3, deer and dog = 5"

            // Convert raw integers to type-safe enums
            let is_undead = PropertyFlag::from_i32(is_undead_raw).unwrap_or(PropertyFlag::Absent);
            let has_blood = PropertyFlag::from_i32(has_blood_raw).unwrap_or(PropertyFlag::Absent);
            let ai_type = MonsterAiType::from_i32(ai_type_raw).unwrap_or(MonsterAiType::Passive);

            let exp_gain_max = reader.read_i32::<LittleEndian>()?;
            let exp_gain_min = reader.read_i32::<LittleEndian>()?;

            let gold_drop_max = reader.read_i32::<LittleEndian>()?;
            let gold_drop_min = reader.read_i32::<LittleEndian>()?;

            let detection_sight_size = reader.read_i32::<LittleEndian>()?; // "9 or 10 - only goblin king have 10"
            let distance_range_size = reader.read_i32::<LittleEndian>()?; // "1 or 6 if archer

            let known_spell_slot1 = reader.read_i32::<LittleEndian>()?;
            let known_spell_slot2 = reader.read_i32::<LittleEndian>()?;
            let known_spell_slot3 = reader.read_i32::<LittleEndian>()?;

            let is_oversize = reader.read_i32::<LittleEndian>()?; // redDragon, balrog, beholder, = 1

            let magic_level = reader.read_i32::<LittleEndian>()?; // always = 1

            let special_attack = reader.read_i32::<LittleEndian>()?; // 0 = none, 1 = bat/zombie/biteworm, 2 = basilisk
            let special_attack_chance = reader.read_i32::<LittleEndian>()?;
            let special_attack_duration = reader.read_i32::<LittleEndian>()?;

            let boldness = reader.read_i32::<LittleEndian>()?; // always = 10
            let attack_speed = reader.read_i32::<LittleEndian>()?;

            monsters.push(Monster {
                id: i,
                name: name.to_string(),
                health_points_max,
                health_points_min,
                mana_points_max,
                mana_points_min,
                walk_speed,
                to_hit_max,
                to_hit_min,
                to_dodge_max,
                to_dodge_min,
                offense_max,
                offense_min,
                defense_max,
                defense_min,
                magic_attack_max,
                magic_attack_min,
                is_undead,
                has_blood,
                ai_type,
                exp_gain_max,
                exp_gain_min,
                gold_drop_max,
                gold_drop_min,
                detection_sight_size,
                distance_range_size,
                known_spell_slot1,
                known_spell_slot2,
                known_spell_slot3,
                is_oversize,
                magic_level,
                special_attack,
                special_attack_chance,
                special_attack_duration,
                boldness,
                attack_speed,
            })
        }

        Ok(monsters)
    }

    fn to_writer<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
        // Since COUNTER_SIZE is 0, we don't write the number of elements

        for record in records {
            let mut name_buf = [0u8; 24];
            let (cow, _, _) = EUC_KR.encode(&record.name);
            let len = std::cmp::min(cow.len(), 24);
            name_buf[..len].copy_from_slice(&cow[..len]);
            writer.write_all(&name_buf)?;

            writer.write_i32::<LittleEndian>(record.health_points_max)?;
            writer.write_i32::<LittleEndian>(record.health_points_min)?;
            writer.write_i32::<LittleEndian>(record.mana_points_max)?;
            writer.write_i32::<LittleEndian>(record.mana_points_min)?;
            writer.write_i32::<LittleEndian>(record.walk_speed)?;
            writer.write_i32::<LittleEndian>(record.to_hit_max)?;
            writer.write_i32::<LittleEndian>(record.to_hit_min)?;
            writer.write_i32::<LittleEndian>(record.to_dodge_max)?;
            writer.write_i32::<LittleEndian>(record.to_dodge_min)?;
            writer.write_i32::<LittleEndian>(record.offense_max)?;
            writer.write_i32::<LittleEndian>(record.offense_min)?;
            writer.write_i32::<LittleEndian>(record.defense_max)?;
            writer.write_i32::<LittleEndian>(record.defense_min)?;
            writer.write_i32::<LittleEndian>(record.magic_attack_max)?;
            writer.write_i32::<LittleEndian>(record.magic_attack_min)?;
            writer.write_i32::<LittleEndian>(i32::from(record.is_undead))?;
            writer.write_i32::<LittleEndian>(i32::from(record.has_blood))?;
            writer.write_i32::<LittleEndian>(i32::from(record.ai_type))?;
            writer.write_i32::<LittleEndian>(record.exp_gain_max)?;
            writer.write_i32::<LittleEndian>(record.exp_gain_min)?;
            writer.write_i32::<LittleEndian>(record.gold_drop_max)?;
            writer.write_i32::<LittleEndian>(record.gold_drop_min)?;
            writer.write_i32::<LittleEndian>(record.detection_sight_size)?;
            writer.write_i32::<LittleEndian>(record.distance_range_size)?;
            writer.write_i32::<LittleEndian>(record.known_spell_slot1)?;
            writer.write_i32::<LittleEndian>(record.known_spell_slot2)?;
            writer.write_i32::<LittleEndian>(record.known_spell_slot3)?;
            writer.write_i32::<LittleEndian>(record.is_oversize)?;
            writer.write_i32::<LittleEndian>(record.magic_level)?;
            writer.write_i32::<LittleEndian>(record.special_attack)?;
            writer.write_i32::<LittleEndian>(record.special_attack_chance)?;
            writer.write_i32::<LittleEndian>(record.special_attack_duration)?;
            writer.write_i32::<LittleEndian>(record.boldness)?;
            writer.write_i32::<LittleEndian>(record.attack_speed)?;
        }
        Ok(())
    }
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
