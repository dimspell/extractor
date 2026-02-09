use std::io::BufReader;
use std::{fs::File, path::Path};

use byteorder::{LittleEndian, ReadBytesExt};
use serde::Serialize;

/// Record size for Magic.db entries: 88 bytes (22 × u32)
const MAGIC_RECORD_SIZE: usize = 88;

/// A magic spell entry from Magic.db
///
/// This file contains spell definitions for the game's magic system.
/// Each record is 88 bytes consisting of 22 little-endian u32 fields.
#[derive(Debug, Serialize)]
pub struct MagicSpell {
    /// Record index (0-based)
    pub id: i32,

    /// Whether the spell is enabled/available
    pub enabled: bool,

    /// Unknown flag, always 1 for valid spells
    pub flag1: u32,

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
    pub flag2: u32,

    /// Range or duration (999 = maximum/unlimited)
    pub range: u32,

    /// Reserved field (always 0)
    pub reserved3: u32,

    /// Required level to learn/cast this spell
    pub level_required: u32,

    /// Constant value (always 1)
    pub constant1: u32,

    /// Secondary effect value
    pub effect_value: u32,

    /// Effect type ID (determines what the spell does)
    pub effect_type: u32,

    /// Effect modifier value
    pub effect_modifier: u32,

    /// Reserved field (always 0)
    pub reserved4: u32,

    /// School of magic (0-6)
    /// 0=Unknown, 1-2=?, 3-4=?, 5-6=?
    pub magic_school: u32,

    /// Unknown flag (0 or 1)
    pub flag3: u32,

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
    pub target_type: u32,
}

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
pub fn read_magic_db(source_path: &Path) -> std::io::Result<Vec<MagicSpell>> {
    let file = File::open(source_path)?;
    let metadata = file.metadata()?;
    let file_len = metadata.len() as usize;

    // Validate file size is a multiple of record size
    if file_len % MAGIC_RECORD_SIZE != 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "Invalid Magic.db file size: {} bytes is not a multiple of {} byte record size",
                file_len, MAGIC_RECORD_SIZE
            ),
        ));
    }

    let num_records = file_len / MAGIC_RECORD_SIZE; // 43
    let mut reader = BufReader::new(file);
    let mut spells: Vec<MagicSpell> = Vec::with_capacity(num_records);

    for i in 0..num_records {
        let enabled_u32 = reader.read_u32::<LittleEndian>()?;
        let flag1 = reader.read_u32::<LittleEndian>()?;
        let mana_cost = reader.read_u32::<LittleEndian>()?;
        let success_rate = reader.read_u32::<LittleEndian>()?;
        let base_damage = reader.read_u32::<LittleEndian>()?;
        let reserved1 = reader.read_u32::<LittleEndian>()?;
        let reserved2 = reader.read_u32::<LittleEndian>()?;
        let flag2 = reader.read_u32::<LittleEndian>()?;
        let range = reader.read_u32::<LittleEndian>()?;
        let reserved3 = reader.read_u32::<LittleEndian>()?;
        let level_required = reader.read_u32::<LittleEndian>()?;
        let constant1 = reader.read_u32::<LittleEndian>()?;
        let effect_value = reader.read_u32::<LittleEndian>()?;
        let effect_type = reader.read_u32::<LittleEndian>()?;
        let effect_modifier = reader.read_u32::<LittleEndian>()?;
        let reserved4 = reader.read_u32::<LittleEndian>()?;
        let magic_school = reader.read_u32::<LittleEndian>()?;
        let flag3 = reader.read_u32::<LittleEndian>()?;
        let animation_id = reader.read_u32::<LittleEndian>()?;
        let visual_id = reader.read_u32::<LittleEndian>()?;
        let icon_id = reader.read_u32::<LittleEndian>()?;
        let target_type = reader.read_u32::<LittleEndian>()?;

        spells.push(MagicSpell {
            id: i as i32,
            enabled: enabled_u32 != 0,
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
