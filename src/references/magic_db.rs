use std::io::{BufReader, BufWriter};
use std::{fs::File, path::Path};

use crate::references::enums::{MagicSchool, MagicSpellConstant, MagicSpellFlag, SpellTargetType};
use crate::references::references::Extractor;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rusqlite::{params, Connection, Result};
use serde::Serialize;

const MAGIC_RECORD_SIZE: usize = 88;

// ===========================================================================
// MAGIC.DB FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | Magic.db - Spell Database           |
// +--------------------------------------+
// | Encoding: Binary (Little-Endian)     |
// | Record Size: 88 bytes (22 × u32)     |
// | No header - count from file size     |
// +--------------------------------------+
// | [Record 1]                           |
// | - enabled: u32 (0/1)                 |
// | - flag1: u32 (always 1)              |
// | - mana_cost: u32 (999=unlimited)     |
// | - success_rate: u32 (0-100%)         |
// | - base_damage: u32                    |
// | - reserved1: u32 (always 0)          |
// | - reserved2: u32 (always 0)          |
// | - flag2: u32 (0/1)                   |
// | - range: u32 (999=unlimited)          |
// | - reserved3: u32 (always 0)          |
// | - level_required: u32                 |
// | - constant1: u32 (always 1)          |
// | - effect_value: u32                   |
// | - effect_type: u32                    |
// | - effect_modifier: u32                |
// | - reserved4: u32 (always 0)          |
// | - magic_school: u32 (0-6)            |
// | - flag3: u32 (0/1)                   |
// | - animation_id: u32                  |
// | - visual_id: u32                      |
// | - icon_id: u32                        |
// | - target_type: u32 (1-4)             |
// +--------------------------------------+
// | [Record 2]                           |
// | ... (same structure) ...             |
// +--------------------------------------+
//
//
// FILE PURPOSE:
// Complete spell database defining all magical abilities with statistics,
// requirements, effects, and visual/audio assets. Used for combat magic,
// character progression, and spellcasting systems.
//
// ===========================================================================

#[derive(Debug, Serialize)]
pub struct MagicSpell {
    /// Record index (0-based)
    pub id: i32,

    /// Whether the spell is enabled/available
    pub enabled: MagicSpellFlag,

    /// Unknown flag, always 1 for valid spells
    pub flag1: MagicSpellFlag,

    /// Mana cost (999 = special/unlimited)
    pub mana_cost: u32,

    /// Success rate / accuracy percentage (0-100)
    pub success_rate: u32,

    /// Base damage or primary effect value
    pub base_damage: u32,

    /// Reserved field (always 0)
    pub reserved1: u32,

    /// Reserved field (always 0)
    pub reserved2: u32,

    /// Unknown flag (0 or 1)
    pub flag2: MagicSpellFlag,

    /// Range or duration (999 = maximum/unlimited)
    pub range: u32,

    /// Reserved field (always 0)
    pub reserved3: u32,

    /// Required level to learn/cast this spell
    pub level_required: u32,

    /// Constant value (always 1)
    pub constant1: MagicSpellConstant,

    /// Secondary effect value
    pub effect_value: u32,

    /// Effect type ID (determines what the spell does)
    pub effect_type: u32,

    /// Effect modifier value
    pub effect_modifier: u32,

    /// Reserved field (always 0)
    pub reserved4: u32,

    /// School of magic (0-6)
    pub magic_school: MagicSchool,

    /// Unknown flag (0 or 1)
    pub flag3: MagicSpellFlag,

    /// Animation or visual effect ID
    pub animation_id: u32,

    /// Sound/visual reference ID
    pub visual_id: u32,

    /// Icon or sprite ID for the spell
    pub icon_id: u32,

    /// Target type:
    /// 1 = Single target
    /// 2 = Self
    /// 3 = Area of effect
    /// 4 = Multi-target
    pub target_type: SpellTargetType,
}

/// Stores data for magic spells, including mana cost, effect types, animations, and base damage.
///
/// Reads file: `MagicInGame/Magic.db or MulMagic.db`
/// # File Format: `MagicInGame/Magic.db`
///
/// Binary file, little-endian. No header — record count derived from file size.
/// Each record is exactly `88 bytes = 22 × u32`:
/// - `enabled`         : u32 (0 = disabled, non-zero = active)
/// - `flag1`           : u32 (always 1 for valid spells)
/// - `mana_cost`       : u32 (999 = special/unlimited)
/// - `success_rate`    : u32 (0–100 %)
/// - `base_damage`     : u32
/// - `reserved1/2`     : u32 × 2 (always 0)
/// - `flag2`           : u32
/// - `range`           : u32 (999 = unlimited)
/// - `reserved3`       : u32 (always 0)
/// - `level_required`  : u32
/// - `constant1`       : u32 (always 1)
/// - `effect_value`    : u32
/// - `effect_type`     : u32
/// - `effect_modifier` : u32
/// - `reserved4`       : u32 (always 0)
/// - `magic_school`    : u32 (0–6)
/// - `flag3`           : u32
/// - `animation_id`    : u32
/// - `visual_id`       : u32
/// - `icon_id`         : u32
/// - `target_type`     : u32 (1=single, 2=self, 3=AoE, 4=multi)
impl Extractor for MagicSpell {
    /// Reads the Magic.db file and returns a vector of magic spell records.
    ///
    /// # File Format
    ///
    /// The Magic.db file uses a simple fixed-record format:
    /// - No header (no record count prefix)
    /// - Each record is 88 bytes (22 × u32 little-endian)
    /// - File size / 88 = number of records
    ///
    /// # Arguments
    ///
    /// * `source_path` - Path to the Magic.db file
    ///
    /// # Returns
    ///
    /// A vector of `MagicSpell` structs representing all spells in the database.
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let file = File::open(source_path)?;
        let metadata = file.metadata()?;
        let file_len = metadata.len() as usize;

        if !file_len.is_multiple_of(MAGIC_RECORD_SIZE) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Invalid Magic.db file size: {} bytes is not a multiple of {} byte record size",
                    file_len, MAGIC_RECORD_SIZE
                ),
            ));
        }

        let num_records = file_len / MAGIC_RECORD_SIZE;
        let mut reader = BufReader::new(file);
        let mut spells: Vec<MagicSpell> = Vec::with_capacity(num_records);

        for i in 0..num_records {
            let enabled_raw = reader.read_u32::<LittleEndian>()?;
            let flag1_raw = reader.read_u32::<LittleEndian>()?;
            let mana_cost = reader.read_u32::<LittleEndian>()?;
            let success_rate = reader.read_u32::<LittleEndian>()?;
            let base_damage = reader.read_u32::<LittleEndian>()?;
            let reserved1 = reader.read_u32::<LittleEndian>()?;
            let reserved2 = reader.read_u32::<LittleEndian>()?;
            let flag2_raw = reader.read_u32::<LittleEndian>()?;
            let range = reader.read_u32::<LittleEndian>()?;
            let reserved3 = reader.read_u32::<LittleEndian>()?;
            let level_required = reader.read_u32::<LittleEndian>()?;
            let constant1_raw = reader.read_u32::<LittleEndian>()?;
            let effect_value = reader.read_u32::<LittleEndian>()?;
            let effect_type = reader.read_u32::<LittleEndian>()?;
            let effect_modifier = reader.read_u32::<LittleEndian>()?;
            let reserved4 = reader.read_u32::<LittleEndian>()?;
            let magic_school_raw = reader.read_u32::<LittleEndian>()?;
            let flag3_raw = reader.read_u32::<LittleEndian>()?;
            let animation_id = reader.read_u32::<LittleEndian>()?;
            let visual_id = reader.read_u32::<LittleEndian>()?;
            let icon_id = reader.read_u32::<LittleEndian>()?;
            let target_type_raw = reader.read_u32::<LittleEndian>()?;

            let enabled = MagicSpellFlag::from_u32(enabled_raw).unwrap_or(MagicSpellFlag::Disabled);
            let flag1 = MagicSpellFlag::from_u32(flag1_raw).unwrap_or(MagicSpellFlag::Disabled);
            let flag2 = MagicSpellFlag::from_u32(flag2_raw).unwrap_or(MagicSpellFlag::Disabled);
            let constant1 =
                MagicSpellConstant::from_u32(constant1_raw).unwrap_or(MagicSpellConstant::Invalid);
            let magic_school =
                MagicSchool::from_u32(magic_school_raw).unwrap_or(MagicSchool::Unknown);
            let flag3 = MagicSpellFlag::from_u32(flag3_raw).unwrap_or(MagicSpellFlag::Disabled);
            let target_type =
                SpellTargetType::from_u32(target_type_raw).unwrap_or(SpellTargetType::Single);

            spells.push(MagicSpell {
                id: i as i32,
                enabled,
                flag1,
                mana_cost,
                success_rate,
                base_damage,
                reserved1,
                reserved2,
                flag2,
                range,
                reserved3,
                level_required,
                constant1,
                effect_value,
                effect_type,
                effect_modifier,
                reserved4,
                magic_school,
                flag3,
                animation_id,
                visual_id,
                icon_id,
                target_type,
            });
        }

        Ok(spells)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let file = File::create(dest_path)?;
        let mut writer = BufWriter::new(file);

        for spell in records {
            writer.write_u32::<LittleEndian>(u32::from(spell.enabled))?;
            writer.write_u32::<LittleEndian>(u32::from(spell.flag1))?;
            writer.write_u32::<LittleEndian>(spell.mana_cost)?;
            writer.write_u32::<LittleEndian>(spell.success_rate)?;
            writer.write_u32::<LittleEndian>(spell.base_damage)?;
            writer.write_u32::<LittleEndian>(spell.reserved1)?;
            writer.write_u32::<LittleEndian>(spell.reserved2)?;
            writer.write_u32::<LittleEndian>(u32::from(spell.flag2))?;
            writer.write_u32::<LittleEndian>(spell.range)?;
            writer.write_u32::<LittleEndian>(spell.reserved3)?;
            writer.write_u32::<LittleEndian>(spell.level_required)?;
            writer.write_u32::<LittleEndian>(u32::from(spell.constant1))?;
            writer.write_u32::<LittleEndian>(spell.effect_value)?;
            writer.write_u32::<LittleEndian>(spell.effect_type)?;
            writer.write_u32::<LittleEndian>(spell.effect_modifier)?;
            writer.write_u32::<LittleEndian>(spell.reserved4)?;
            writer.write_u32::<LittleEndian>(u32::from(spell.magic_school))?;
            writer.write_u32::<LittleEndian>(u32::from(spell.flag3))?;
            writer.write_u32::<LittleEndian>(spell.animation_id)?;
            writer.write_u32::<LittleEndian>(spell.visual_id)?;
            writer.write_u32::<LittleEndian>(spell.icon_id)?;
            writer.write_u32::<LittleEndian>(u32::from(spell.target_type))?;
        }
        Ok(())
    }
}

pub fn read_magic_db(source_path: &Path) -> std::io::Result<Vec<MagicSpell>> {
    MagicSpell::read_file(source_path)
}

pub fn save_magic_spells(conn: &mut Connection, spells: &Vec<MagicSpell>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_magic_spell.sql"))?;
        for spell in spells {
            stmt.execute(params![
                spell.id,
                u32::from(spell.enabled),
                u32::from(spell.flag1),
                spell.mana_cost,
                spell.success_rate,
                spell.base_damage,
                spell.reserved1,
                spell.reserved2,
                u32::from(spell.flag2),
                spell.range,
                spell.reserved3,
                spell.level_required,
                u32::from(spell.constant1),
                spell.effect_value,
                spell.effect_type,
                spell.effect_modifier,
                spell.reserved4,
                u32::from(spell.magic_school),
                u32::from(spell.flag3),
                spell.animation_id,
                spell.visual_id,
                spell.icon_id,
                u32::from(spell.target_type),
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
