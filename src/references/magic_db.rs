use std::path::Path;

use crate::references::enums::{MagicSchool, MagicSpellConstant, MagicSpellFlag, SpellTargetType};
use crate::references::extractor::Extractor;
use dispel_macros::Extractor;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

/// MagicSpell.db - Magic Spells
///
/// Stores data for magic spells, including mana cost, effect types, animations, and base damage.
///
/// Reads file: `MagicInGame/Magic.db` or `MagicInGame/MulMagic.db`
///
/// # Binary Format
///
/// - **Encoding**: Little-endian for all values (u32)
/// - **Record Size**: 88 bytes (22 × u32)
/// - **Header**: None; parse until EOF (no record count)
///
/// ```text
/// +--------------------------------------+
/// | MagicSpell.db - Magic Spells       |
/// +--------------------------------------+
/// | Encoding: Binary (Little-Endian)     |
/// | Record Size: 88 bytes (22 × u32)    |
/// | Header: None (parse until EOF)       |
/// +--------------------------------------+
/// | [Record 1] - 88 bytes               |
/// | - id: i32 (auto-generated)           |
/// | - enabled: u32 (MagicSpellFlag)      |
/// | - flag1: u32 (MagicSpellFlag)        |
/// | - mana_cost: u32 (999=unlimited)    |
/// | - success_rate: u32 (0-100%)         |
/// | - base_damage: u32                   |
/// | - reserved1: u32 (always 0)          |
/// | - reserved2: u32 (always 0)          |
/// | - flag2: u32 (MagicSpellFlag)        |
/// | - range: u32 (999=unlimited)         |
/// | - reserved3: u32 (always 0)          |
/// | - level_required: u32                 |
/// | - constant1: u32 (MagicSpellConstant)|
/// | - effect_value: u32                   |
/// | - effect_type: u32                   |
/// | - effect_modifier: u32                |
/// | - reserved4: u32 (always 0)          |
/// | - magic_school: u32 (MagicSchool)    |
/// | - flag3: u32 (MagicSpellFlag)        |
/// | - animation_id: u32                  |
/// | - visual_id: u32                     |
/// | - icon_id: u32                       |
/// | - target_type: u32 (SpellTargetType) |
/// +--------------------------------------+
/// | [Record 2]                           |
/// | ... (same structure) ...             |
/// +--------------------------------------+
/// ```
///
/// # Field Categories
///
/// - **Identification**: `id` (auto-generated from position)
/// - **State**: `enabled`, `flag1`, `flag2`, `flag3` (MagicSpellFlag)
/// - **Cost & Requirements**: `mana_cost`, `level_required`, `success_rate`
/// - **Combat**: `base_damage`, `effect_type`, `effect_value`, `effect_modifier`
/// - **Range & Targeting**: `range`, `target_type` (Single/Area/Multi)
/// - **Visuals**: `animation_id`, `visual_id`, `icon_id`
/// - **School**: `magic_school` (0-6, e.g., Fire, Water, etc.)
/// - **Constants**: `constant1` (MagicSpellConstant, always 1)
/// - **Reserved**: `reserved1-4` (always 0)
///
/// # Special Values
///
/// - `enabled`: `MagicSpellFlag::Enabled` (1) or `Disabled` (0)
/// - `mana_cost = 999`: Unlimited/special mana cost
/// - `range = 999`: Maximum/unlimited range
/// - `success_rate`: Percentage 0-100
/// - `magic_school`: 0-6 (Fire, Water, Earth, Air, Light, Dark, etc.)
/// - `target_type`: 1=Single, 2=Self, 3=Area, 4=Multi-target
///
/// # File Purpose
///
/// Defines all magic spells with costs, effects, targeting,
/// and visual properties. Used for spell casting system,
/// magic combat, and spell learning mechanics.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Extractor)]
#[extractor(property_item_size = 88, counter_size = 0)]
pub struct MagicSpell {
    /// Record index (0-based)
    #[extractor(index)]
    pub id: i32,

    /// Whether the spell is enabled/available
    #[extractor(enum_from_u32(type = "MagicSpellFlag"))]
    pub enabled: MagicSpellFlag,

    /// Unknown flag, always 1 for valid spells
    #[extractor(enum_from_u32(type = "MagicSpellFlag"))]
    pub flag1: MagicSpellFlag,

    /// Mana cost (999 = special/unlimited)
    #[extractor(primitive(type = "u32"))]
    pub mana_cost: u32,

    /// Success rate / accuracy percentage (0-100)
    #[extractor(primitive(type = "u32"))]
    pub success_rate: u32,

    /// Base damage or primary effect value
    #[extractor(primitive(type = "u32"))]
    pub base_damage: u32,

    /// Reserved field (always 0)
    #[extractor(primitive(type = "u32"))]
    pub reserved1: u32,

    /// Reserved field (always 0)
    #[extractor(primitive(type = "u32"))]
    pub reserved2: u32,

    /// Unknown flag (0 or 1)
    #[extractor(enum_from_u32(type = "MagicSpellFlag"))]
    pub flag2: MagicSpellFlag,

    /// Range or duration (999 = maximum/unlimited)
    #[extractor(primitive(type = "u32"))]
    pub range: u32,

    /// Reserved field (always 0)
    #[extractor(primitive(type = "u32"))]
    pub reserved3: u32,

    /// Required level to learn/cast this spell
    #[extractor(primitive(type = "u32"))]
    pub level_required: u32,

    /// Constant value (always 1)
    #[extractor(enum_from_u32(type = "MagicSpellConstant"))]
    pub constant1: MagicSpellConstant,

    /// Secondary effect value
    #[extractor(primitive(type = "u32"))]
    pub effect_value: u32,

    /// Effect type ID (determines what the spell does)
    #[extractor(primitive(type = "u32"))]
    pub effect_type: u32,

    /// Effect modifier value
    #[extractor(primitive(type = "u32"))]
    pub effect_modifier: u32,

    /// Reserved field (always 0)
    #[extractor(primitive(type = "u32"))]
    pub reserved4: u32,

    /// School of magic (0-6)
    #[extractor(enum_from_u32(type = "MagicSchool"))]
    pub magic_school: MagicSchool,

    /// Unknown flag (0 or 1)
    #[extractor(enum_from_u32(type = "MagicSpellFlag"))]
    pub flag3: MagicSpellFlag,

    /// Animation or visual effect ID
    #[extractor(primitive(type = "u32"))]
    pub animation_id: u32,

    /// Sound/visual reference ID
    #[extractor(primitive(type = "u32"))]
    pub visual_id: u32,

    /// Icon or sprite ID for the spell
    #[extractor(primitive(type = "u32"))]
    pub icon_id: u32,

    /// Target type:
    /// 1 = Single target
    /// 2 = Self
    /// 3 = Area of effect
    /// 4 = Multi-target
    #[extractor(enum_from_u32(type = "SpellTargetType"))]
    pub target_type: SpellTargetType,
}

pub fn read_magic_db(source_path: &Path) -> std::io::Result<Vec<MagicSpell>> {
    MagicSpell::read_file(source_path)
}

pub fn save_magic_spells(conn: &mut Connection, spells: &[MagicSpell]) -> Result<()> {
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

impl std::fmt::Display for MagicSpell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MagicSpell({} - mana: {}, damage: {})",
            self.id, self.mana_cost, self.base_damage
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::references::enums::{MagicSchool, MagicSpellFlag, SpellTargetType};
    use std::io::Cursor;

    fn spell_bytes(mana_cost: u32, base_damage: u32, target_type: u32) -> Vec<u8> {
        let fields: [u32; 22] = [
            1, // enabled
            1, // flag1
            mana_cost,
            100, // success_rate
            base_damage,
            0,
            0,  // reserved1, reserved2
            0,  // flag2
            10, // range
            0,  // reserved3
            1,  // level_required
            1,  // constant1
            0,  // effect_value
            1,  // effect_type
            0,  // effect_modifier
            0,  // reserved4
            0,  // magic_school (Unknown)
            0,  // flag3
            1,  // animation_id
            2,  // visual_id
            3,  // icon_id
            target_type,
        ];
        fields.iter().flat_map(|&v| v.to_le_bytes()).collect()
    }

    #[test]
    fn parse_single_spell() {
        let data = spell_bytes(20, 50, 1);
        assert_eq!(data.len(), 88);

        let mut c = Cursor::new(&data[..]);
        let spells = MagicSpell::parse(&mut c, 88).unwrap();

        assert_eq!(spells.len(), 1);
        assert_eq!(spells[0].id, 0);
        assert_eq!(spells[0].enabled, MagicSpellFlag::Enabled);
        assert_eq!(spells[0].mana_cost, 20);
        assert_eq!(spells[0].base_damage, 50);
        assert_eq!(spells[0].magic_school, MagicSchool::Unknown);
        assert_eq!(spells[0].target_type, SpellTargetType::Single);
    }

    #[test]
    fn parse_two_spells() {
        let mut data = spell_bytes(10, 30, 2);
        data.extend(spell_bytes(40, 80, 3));
        assert_eq!(data.len(), 176);

        let mut c = Cursor::new(&data[..]);
        let spells = MagicSpell::parse(&mut c, 176).unwrap();

        assert_eq!(spells.len(), 2);
        assert_eq!(spells[0].mana_cost, 10);
        assert_eq!(spells[1].mana_cost, 40);
        assert_eq!(spells[1].target_type, SpellTargetType::AreaOfEffect);
    }

    #[test]
    fn parse_invalid_size_returns_partial() {
        let data = vec![0u8; 90]; // not a multiple of 88
        let mut c = Cursor::new(&data[..]);
        // The macro doesn't validate file size - it just parses what it can (1 record from 90 bytes)
        let result = MagicSpell::parse(&mut c, 90);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn serialize_round_trip() {
        let data = spell_bytes(20, 50, 1);
        let mut c = Cursor::new(&data[..]);
        let records = MagicSpell::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        MagicSpell::to_writer(&records, &mut out).unwrap();
        assert_eq!(out, data);
    }
}
