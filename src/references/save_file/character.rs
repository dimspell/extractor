use dispel_macros::BinaryRecord;
use serde::{Deserialize, Serialize};

/// Parse actual position, character stats, and some unknown bytes (112 bytes)
///
/// Layout:
///   `[unknown_01: 8B][position_x: i16][position_y: i16]
///    [unknown_02: 5B][selected_spell_id: u32][unknown_03: 19BB]
///    [strength u16][agility u16][wisdom u16][constitution u16]
///    [morale u16][hp_cur u16][hp_max u16][mp_cur u16][mp_max u16]
///    [xp u32][level u16][gold u32][offense u16][defense u16]
///    [dodge u8][hit u8][magic_power u16][attack_mod u8]
///    [thievery u8][lockpick u8][haggle u8][perception u8][traps u8]
///    [sword_lv u8][sword_kills u16][axe_lv u8][axe_kills u16]
///    [archery_lv u8][archery_kills u16][polearm_lv u8][polearm_kills u16]
///    [magic_lv u8][magic_kills u16][holy_lv u8][holy_kills u16]
///    [dark_lv u8][dark_kills u16][unknown: 9B]`
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct CharacterData {
    pub unknown_01: [u8; 8], // TODO: Recognise them
    pub character_position_x: i16,
    pub character_position_y: i16,
    pub unknown_02: [u8; 5],    // TODO: Recognise them
    pub selected_spell_id: u32, // TODO: Verify me
    pub unknown_03: [u8; 19],   // TODO: Recognise them

    // Parsed character stats (core, combat, skills, weapon skills) (63 bytes).
    // ── Core attributes ──
    pub strength: u16,
    pub agility: u16,
    pub wisdom: u16,
    pub constitution: u16,
    pub morale: u16,
    pub hp_current: u16,
    pub hp_maximum: u16,
    pub mp_current: u16,
    pub mp_maximum: u16,
    pub experience: u32,
    pub level: u16,
    pub gold: u32,

    // ── Combat stats ──
    pub offense: u16,
    pub defense: u16,
    pub dodge_rate: u8,
    pub hit_rate: u8,
    pub magic_power: u16,
    pub attack_modifier: u8,

    // ── Skills (5 × u8) ──
    pub pickpocketing: u8,
    pub lockpicking: u8,
    pub haggling: u8,
    pub perception: u8,
    pub traps: u8,

    // ── Weapon skills (7 types × {level: u8, kills: u16}) ──
    pub swords_level: u8,
    pub swords_kills: u16,
    pub axes_level: u8,
    pub axes_kills: u16,
    pub archery_level: u8,
    pub archery_kills: u16,
    pub polearm_level: u8,
    pub polearm_kills: u16,
    pub magic_level: u8,
    pub magic_kills: u16,
    pub holy_magic_level: u8,
    pub holy_magic_kills: u16,
    pub dark_magic_level: u8,
    pub dark_magic_kills: u16,

    /// Unknown bytes after stats block (9 bytes).
    pub unknown_04: [u8; 9],
}

/// Character identity data (name, class, unknown bytes) - 131 bytes.
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct CharacterIdentity {
    /// Unknown block before player name (96 bytes).
    #[binary_record(size = 96)]
    pub unknown_00: Vec<u8>,
    /// Player name (11 bytes, null-terminated string).
    #[binary_record(string(encoding = "WINDOWS-1250", size = 11))]
    pub player_name: String,
    /// Player class ID.
    pub player_class_id: u16,
    /// Player class name (11-byte WINDOWS-1250 null-terminated).
    #[binary_record(string(encoding = "WINDOWS-1250", size = 11))]
    pub player_class_name: String,

    // -- Header block before equipment data (11 bytes).
    #[binary_record(size = 6)]
    pub unknown_02: Vec<u8>,
    pub unknown_03: u8, // TODO: It is likely strength attribute (maybe after modificators/curses)
    pub unknown_04: u8,
    pub unknown_05: u8, // TODO: It is likely agility attribute (maybe after modificators/curses)
    pub unknown_06: u8,
    pub unknown_07: u8,
}

/// Learned spells block (41 bytes).
///
/// One byte per spell, likely boolean flags indicating whether each
/// spell has been learned.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LearnedSpells {
    pub spells: Vec<u8>,
}
