use crate::references::extractor::read_null_terminated_windows_1250;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use dispel_macros::BinaryRecord;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use crate::references::save_file::inventory::InventoryPlacements;

/// Data immediately before the character stats block (28 bytes).
///
/// Layout: `[unknown_a: u8][unknown_b: u32][selected_spell_id: u32][unknown_block: 19 bytes]`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterStatsHeader {
    pub unknown_a: u8,
    pub unknown_b: u32,
    /// ID of the spell currently selected by the player.
    pub selected_spell_id: u32,
    /// Remaining unknown bytes in the header.
    pub unknown_block: [u8; 19],
}

/// Parsed character stats from a save file.
///
/// Maps the binary stats block (~68 bytes of structured data) that follows
/// the belt-data section and precedes the inventory section.
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct CharacterStats {
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
}

/// Character data header block (11 bytes).
///
/// Read immediately after the player class name and before the
/// equipment/belt/inventory/spells blocks. Internal field meanings are
/// not yet decoded.
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct CharacterDataHeader {
    pub unknown_a: u32,
    pub unknown_b: u16,
    pub unknown_c: u8, // TODO: It is likely strength attribute (maybe after modificators/curses)
    pub unknown_c2: u8,
    pub unknown_d: u8, // TODO: It is likely agility attribute (maybe after modificators/curses)
    pub unknown_e: u8,
    pub unknown_f: u8,
}

/// Character identity data (name, class, equipment, spells, party).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterIdentity {
    /// Unknown block before player name (96 bytes).
    pub unknown_block: Vec<u8>,
    /// Player name (11-byte WINDOWS-1250 null-terminated).
    pub player_name: String,
    /// Player class ID.
    pub player_class_id: u16,
    /// Player class name (11-byte WINDOWS-1250 null-terminated).
    pub player_class_name: String,
    /// Header block before equipment data (11 bytes).
    pub character_data_header: CharacterDataHeader,

    pub inventory_placements: InventoryPlacements,
    /// Learned spells — 41 bytes (one flag per spell).
    pub learned_spells: LearnedSpells,
}

/// Learned spells block (41 bytes).
///
/// One byte per spell, likely boolean flags indicating whether each
/// spell has been learned.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LearnedSpells {
    pub spells: Vec<u8>,
}
