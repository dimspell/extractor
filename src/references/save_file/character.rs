use crate::references::extractor::read_null_terminated_windows_1250;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use dispel_macros::BinaryRecord;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

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

/// Character data header block (11 bytes).
///
/// Read immediately after the player class name and before the
/// equipment/belt/inventory/spells blocks. Internal field meanings are
/// not yet decoded.
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct CharacterDataHeader {
    pub unknown_a: u32,
    pub unknown_b: u16,
    pub unknown_c: u16,
    pub unknown_d: u8,
    pub unknown_e: u8,
    pub unknown_f: u8,
}

/// One equipped weapon-item reference (9 bytes).
///
/// Part of the 12-slot equipment array (12 × 9 = 108 bytes total).
/// An empty entry has catalog index `100` and panel marker `0xff`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EquipmentSlot {
    /// Equipment-panel marker used by the game to restore this slot's UI state; `0xff` is empty.
    pub panel_slot_marker: u8,
    /// Zero-based index in the weapon-item catalog; `100` is empty.
    pub weapon_catalog_index: i32,
    /// `InventoryWeaponItem::inventory_instance_id` of the equipped weapon; zero is empty.
    pub weapon_inventory_instance_id: i32,
}

/// One belt item placement cell (16 bytes).
///
/// Part of the 6-cell belt array (6 × 16 = 96 bytes total). Larger items can
/// occupy consecutive cells with the same catalog index and icon position.
/// Empty cells use category `10` and catalog index `100`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BeltPotionSlot {
    /// Item category; the belt uses category `1` for an occupied item.
    pub item_category: i32,
    /// Zero-based index in that category's catalog; `100` is empty.
    pub item_catalog_index: i32,
    /// Horizontal pixel coordinate at which the belt icon is drawn.
    pub icon_x: i32,
    /// Vertical pixel coordinate at which the belt icon is drawn.
    pub icon_y: i32,
}

/// One item reference and its position in the inventory placement grid (20 bytes).
///
/// The grid is serialized as three pages, each with seven 9-cell columns:
/// `[3 pages][7 columns][9 cells]`. Empty cells use category `10` and catalog
/// index `100`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventoryPlacementEntry {
    /// Zero-based item category used to select an item collection; `10` marks an empty cell.
    pub item_category: i32,
    /// Zero-based index of the item's definition within `item_category`; `100` marks an empty cell.
    pub item_catalog_index: i32,
    /// Horizontal pixel coordinate at which the inventory icon is drawn.
    pub icon_x: i32,
    /// Vertical pixel coordinate at which the inventory icon is drawn.
    pub icon_y: i32,
    /// Category-local index of the instantiated inventory item represented by this placement.
    pub item_instance_index: i32,
}

/// Learned spells block (41 bytes).
///
/// One byte per spell, likely boolean flags indicating whether each
/// spell has been learned.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LearnedSpells {
    pub spells: Vec<u8>,
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
    /// Equipped weapon items — 12 slots × 9 bytes = 108 bytes.
    pub equipped_equipment: Vec<EquipmentSlot>,
    /// Belt item placements — 6 cells × 16 bytes = 96 bytes.
    pub belt_potions: Vec<BeltPotionSlot>,
    /// Inventory item placements — 3 pages × 7 columns × 9 cells × 20 bytes.
    pub inventory_placement: Vec<InventoryPlacementEntry>,
    /// Learned spells — 41 bytes (one flag per spell).
    pub learned_spells: LearnedSpells,
}
