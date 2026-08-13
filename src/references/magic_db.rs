use std::path::Path;

use crate::references::enums::{MagicSchool, MagicSpellFlag, SpellTargetType};
use crate::references::extractor::Extractor;
use dispel_macros::{Extractor, RecordPatcher};
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

/// MagicSpell.db - Magic Spells
///
/// Stores the binary spell definitions used by the combat engine.
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
/// | - effect_visual_blends_with_background: u32 |
/// | - base_damage: u32                    |
/// | - base_success_rate: u32              |
/// | - mana_cost: u32                      |
/// | - reserved_0x14: u32                  |
/// | - reserved_0x18: u32                  |
/// | - effect_animation_repeats: u32       |
/// | - range: u32                          |
/// | - reserved_0x24: u32                  |
/// | - cast_duration: u32                  |
/// | - unused_constant_one: u32            |
/// | - effect_value: u32                   |
/// | - effect_type: u32                   |
/// | - effect_modifier: u32                |
/// | - reserved_0x3c: u32                  |
/// | - magic_school: u32 (MagicSchool)     |
/// | - target_animation_blends_with_background: u32 |
/// | - animation_set_id: u32               |
/// | - effect_visual_id: u32               |
/// | - icon_id: u32                       |
/// | - targeting_mode: u32 (SpellTargetType)|
/// +--------------------------------------+
/// | [Record 2]                           |
/// | ... (same structure) ...             |
/// +--------------------------------------+
/// ```
///
/// # Reverse-engineered behavior
///
/// The combat code reads `base_damage`, `base_success_rate`, `mana_cost`,
/// `range`, `cast_duration`, `magic_school`, `animation_set_id`, and
/// `effect_visual_id`. Effective mana cost is reduced by the caster's
/// magic-school skill, with a minimum of 5; effective success chance also
/// includes that skill. Offset-based names are retained only for effect
/// configuration words whose behavior remains unconfirmed.
///
/// # File Purpose
///
/// Defines all magic spells with costs, effects, targeting,
/// and visual properties. Used for spell casting system,
/// magic combat, and spell learning mechanics.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Extractor, RecordPatcher)]
#[extractor(property_item_size = 88, counter_size = 0)]
#[patcher(filename = "Magic.db")]
pub struct MagicSpell {
    /// Record index (0-based)
    #[extractor(index)]
    pub id: i32,

    /// Whether the spell is enabled/available
    #[extractor(enum_from_u32(type = "MagicSpellFlag"))]
    pub enabled: MagicSpellFlag,

    /// Selects blended rendering (`1`) rather than direct blitting (`0`) for
    /// the spell's initial visual effect.
    #[extractor(enum_from_u32(type = "MagicSpellFlag"))]
    pub effect_visual_blends_with_background: MagicSpellFlag,

    /// Base damage used in the spell-damage calculation.
    #[extractor(primitive(type = "u32"))]
    pub base_damage: u32,

    /// Base casting-success chance before the magic-school skill adjustment.
    #[extractor(primitive(type = "u32"))]
    pub base_success_rate: u32,

    /// Base mana cost before the magic-school skill reduction (minimum 5).
    #[extractor(primitive(type = "u32"))]
    pub mana_cost: u32,

    /// Reserved word at record offset `0x14`; zero in the shipped `Magic.db`.
    #[extractor(primitive(type = "u32"))]
    pub reserved_0x14: u32,

    /// Reserved word at record offset `0x18`; zero in the shipped `Magic.db`.
    #[extractor(primitive(type = "u32"))]
    pub reserved_0x18: u32,

    /// Repeats the target-effect animation after its final frame while the
    /// target remains valid. When clear, the effect stops at the final frame.
    #[extractor(enum_from_u32(type = "MagicSpellFlag"))]
    pub effect_animation_repeats: MagicSpellFlag,

    /// Maximum target distance checked by the casting code.
    #[extractor(primitive(type = "u32"))]
    pub range: u32,

    /// Reserved word at record offset `0x24`; zero in the shipped `Magic.db`.
    #[extractor(primitive(type = "u32"))]
    pub reserved_0x24: u32,

    /// Casting/action duration, expressed as the maximum progress counter.
    #[extractor(primitive(type = "u32"))]
    pub cast_duration: u32,

    /// Compatibility constant at record offset `0x2c` (always 1 in shipped
    /// data and not read by this executable).
    #[extractor(primitive(type = "u32"))]
    pub unused_constant_one: u32,

    /// Secondary effect value
    #[extractor(primitive(type = "u32"))]
    pub effect_value: u32,

    /// Effect type ID (determines what the spell does)
    #[extractor(primitive(type = "u32"))]
    pub effect_type: u32,

    /// Effect modifier value
    #[extractor(primitive(type = "u32"))]
    pub effect_modifier: u32,

    /// Reserved word at record offset `0x3c`; zero in the shipped `Magic.db`.
    #[extractor(primitive(type = "u32"))]
    pub reserved_0x3c: u32,

    /// Magic-school/stat category used in cost and success calculations.
    #[extractor(enum_from_u32(type = "MagicSchool"))]
    pub magic_school: MagicSchool,

    /// Selects blended rendering (`1`) rather than direct blitting (`0`) for
    /// the target animation.
    #[extractor(enum_from_u32(type = "MagicSpellFlag"))]
    pub target_animation_blends_with_background: MagicSpellFlag,

    /// Animation-set ID used for this spell's cast animation.
    #[extractor(primitive(type = "u32"))]
    pub animation_set_id: u32,

    /// Visual/projectile mapping ID used when the spell is cast.
    #[extractor(primitive(type = "u32"))]
    pub effect_visual_id: u32,

    /// Icon or sprite ID for the spell
    #[extractor(primitive(type = "u32"))]
    pub icon_id: u32,

    /// Targeting-mode configuration. Exact values need further confirmation.
    #[extractor(enum_from_u32(type = "SpellTargetType"))]
    pub targeting_mode: SpellTargetType,
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
                u32::from(spell.effect_visual_blends_with_background),
                spell.base_damage,
                spell.base_success_rate,
                spell.mana_cost,
                spell.reserved_0x14,
                spell.reserved_0x18,
                u32::from(spell.effect_animation_repeats),
                spell.range,
                spell.reserved_0x24,
                spell.cast_duration,
                spell.unused_constant_one,
                spell.effect_value,
                spell.effect_type,
                spell.effect_modifier,
                spell.reserved_0x3c,
                u32::from(spell.magic_school),
                u32::from(spell.target_animation_blends_with_background),
                spell.animation_set_id,
                spell.effect_visual_id,
                spell.icon_id,
                u32::from(spell.targeting_mode),
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

    fn spell_bytes(base_damage: u32, mana_cost: u32, targeting_mode: u32) -> Vec<u8> {
        let fields: [u32; 22] = [
            1, // enabled
            1, // effect_visual_blends_with_background
            base_damage,
            100, // base_success_rate
            mana_cost,
            0,
            0,  // reserved_0x14, reserved_0x18
            0,  // effect_animation_repeats
            10, // range
            0,  // reserved_0x24
            1,  // cast_duration
            1,  // unused_constant_one
            0,  // effect_value
            1,  // effect_type
            0,  // effect_modifier
            0,  // reserved_0x3c
            0,  // magic_school (Unknown)
            0,  // target_animation_blends_with_background
            1,  // animation_set_id
            2,  // effect_visual_id
            3,  // icon_id
            targeting_mode,
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
        assert_eq!(spells[0].base_damage, 20);
        assert_eq!(spells[0].mana_cost, 50);
        assert_eq!(spells[0].magic_school, MagicSchool::Unknown);
        assert_eq!(spells[0].targeting_mode, SpellTargetType::Single);
    }

    #[test]
    fn parse_two_spells() {
        let mut data = spell_bytes(10, 30, 2);
        data.extend(spell_bytes(40, 80, 3));
        assert_eq!(data.len(), 176);

        let mut c = Cursor::new(&data[..]);
        let spells = MagicSpell::parse(&mut c, 176).unwrap();

        assert_eq!(spells.len(), 2);
        assert_eq!(spells[0].mana_cost, 30);
        assert_eq!(spells[1].mana_cost, 80);
        assert_eq!(spells[1].targeting_mode, SpellTargetType::AreaOfEffect);
    }

    #[test]
    fn parse_invalid_size_returns_partial() {
        let data = [0u8; 90]; // not a multiple of 88
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
