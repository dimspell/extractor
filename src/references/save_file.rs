// Save file extraction and parsing for Dispel RPG
//
// This module provides comprehensive parsing of Dispel RPG save files (.sav)
// following the binary format documented in SAVE_FILE_RESEARCH.md

use byteorder::{LittleEndian, ReadBytesExt};
use dispel_macros::BinaryRecord;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, Write};
// use proptest::char::range;
use super::extractor::{read_null_terminated_windows_1250, Extractor};

/// Monster record from save file (surface or dungeon)
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct MonsterRecord {
    pub signature_a: u32,
    pub record_index: u32,
    pub signature_b: u32,
    #[binary_record(string(encoding = "WINDOWS-1250", size = 21))]
    pub name: String,
    pub monster_db_id: u32,
    pub hp_current: u16,
    pub hp_maximum: u16,
    pub mp_current: u16,
    pub mp_maximum: u16,
    pub walk_speed: u8,
    pub hit_rate: u8,
    pub dodge_rate: u8,
    pub offense_rate: u16,
    pub defense_rate: u16,
    pub magic_rate: u16,
    pub is_undead: u8,
    pub has_blood: u8,
    pub monster_ai_type: u8,
    pub experience_on_kill: u16,
    pub gold_drop_on_kill: u16,
    pub unknown_1: u8,
    pub sight_range: u8,
    pub attack_range: u8,
    pub spell_slot_1: i8,
    pub spell_slot_2: i8,
    pub spell_slot_3: i8,
    pub oversize: u8,
    pub magic_level: u8,
    pub unknown_2: u32,
    pub unknown_3: [u8; 25],
    pub unknown_4: u32,
    pub unknown_5: i32, // -1 if [255, 255, 255, 255]
    pub current_position_x: u16,
    pub current_position_y: u16,
    pub spawn_position_x: u16,
    pub spawn_position_y: u16,
    pub unknown_10_coordinate: u16,
    pub unknown_11_coordinate: u16,
    pub unknown_12: u8,
    pub unknown_13: u8,
    pub unknown_14: u8,
    pub unknown_15: u16,
    pub unknown_16: i16, // -1 if [255]
    pub unknown_17: u16,
    pub unknown_18: u32,
    pub unknown_19: [u8; 18],
    pub unknown_20: i32,
    pub unknown_21: u32,
    pub unknown_22: u32,
    pub loot_item1: i32,
    pub loot_item2: i32,
    pub loot_item3: i32,
    pub mon_ref_padding_12: u32,
    pub mon_ref_padding_13: u32,
    pub unknown_23: u32,
    pub unknown_24: u32,
    pub unknown_25: u32,
    pub unknown_26: u32,
    pub special_attack_chance: u32,
    pub special_attack_duration: u32,
    pub unknown_27: [u8; 8],
    pub boldness: u32,
    pub attack_speed: u32,
    pub unknown_28: [u8; 6],
    pub unknown_29: u32,
    #[binary_record(size = 98)]
    pub unknown_30: Vec<u8>,
}

/// NPC record from save file (349 bytes)
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct NpcRecord {
    #[binary_record(string(encoding = "WINDOWS-1250", size = 64))]
    pub name: String,
    #[binary_record(string(encoding = "WINDOWS-1250", size = 64))]
    pub role_description: String,
    pub unknown1: u32,
    pub unknown2: u32,
    pub unknown3: u32,
    pub unknown4: u16,
    pub unknown5: u16,
    pub unknown6: u16,
    pub unknown7: u16,
    pub unknown8: u16,
    pub unknown9: u16,
    pub unknown10: u16,
    pub unknown11: u16,
    pub unknown12: [u8; 15],
    pub npc_ini_id: u8,
    pub unknown13: [u8; 20],
    pub npc_ref_party_script_id: u16,
    pub npc_ref_show_on_event_id: u16,
    pub unknown14: u8,
    pub npc_ref_unknown_1: u8,
    pub npc_ref_waypoint1filled: u32,
    pub npc_ref_waypoint1x: u32,
    pub npc_ref_waypoint1y: u32,
    pub npc_ref_unknown_2: u32,
    pub npc_ref_look_direction: u32,
    pub npc_ref_unknown_9: u32,
    pub npc_ref_waypoint2filled: u32,
    pub npc_ref_waypoint2x: u32,
    pub npc_ref_waypoint2y: u32,
    pub npc_ref_unknown_3: u32,
    pub npc_ref_unknown_6: u32,
    pub npc_ref_unknown_10: u32,
    pub npc_ref_waypoint3filled: u32,
    pub npc_ref_waypoint3x: u32,
    pub npc_ref_waypoint3y: u32,
    pub npc_ref_unknown_4: u32,
    pub npc_ref_unknown_7: u32,
    pub npc_ref_unknown_11: u32,
    pub npc_ref_waypoint4filled: u32,
    pub npc_ref_waypoint4x: u32,
    pub npc_ref_waypoint4y: u32,
    pub npc_ref_unknown_5: u32,
    pub npc_ref_unknown_8: u32,
    pub npc_ref_unknown_12: u32,
    pub npc_ref_unknown_13: u32,
    pub npc_ref_unknown_14: u32,
    pub npc_ref_unknown_15: u32,
    pub npc_ref_unknown_16: u32,
    pub npc_ref_unknown_17: u32,
    pub unknown15: u16,
    pub npc_ref_dialog_id: u32,
    pub unknown16: [u8; 29],
}

/// Extra object record (200-byte data per record)
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct ExtraObjectRecord {
    pub unknown_1: u32,
    pub unknown_2: u32,
    pub unknown_3: u32,
    pub unknown_4: u16,
    pub unknown_5: u8,
    #[binary_record(string(encoding = "WINDOWS-1250", size = 32))]
    pub name: String,
    pub unknown_6: u8,
    pub unknown_7: u32, // possibly coordinate x
    pub unknown_8: u32, // possibly coordinate y
    pub unknown_9: u32,
    #[binary_record(size = 8)]
    pub unknown_10: Vec<u8>,
    pub unknown_11: u32,
    pub unknown_12: u32,
    pub unknown_13: u32,
    pub unknown_14: u32,
    pub unknown_15: u32,
    pub unknown_16: u32,
    pub unknown_17: u32,
    pub unknown_18: u32,
    pub unknown_19: u32,
    pub unknown_20: u32,
    pub unknown_21: u32,
    pub unknown_22: u32,
    #[binary_record(size = 24)]
    pub unknown_23: Vec<u8>,
    pub unknown_24: u32,
    pub unknown_25: u32,
    pub unknown_26: u32,
    pub unknown_27: u32,
    pub unknown_28: u32,
    pub unknown_29: i16,
    #[binary_record(size = 2)]
    pub unknown_30: Vec<u8>,
    #[binary_record(size = 8)]
    pub unknown_31: Vec<u8>,
    pub unknown_32: u32,
    pub unknown_33: u32,
    pub unknown_34: u32,
    pub unknown_35: u32,
    pub unknown_36: u32,
    pub unknown_37: u32,
    pub unknown_38: u32,
}

/// Event script record (save file format: 284 bytes each)
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct EventScript {
    pub event_id: u32,
    pub unknown_1: u32,
    pub unknown_2: u32,
    #[binary_record(string(encoding = "WINDOWS-1250", size = 272))]
    pub script_name: String,
}

/// Journal entry (37 bytes each)
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct JournalEntry {
    pub index: u8,
    #[binary_record(string(encoding = "WINDOWS-1250", size = 24))]
    pub name: String,
    #[binary_record(size = 12)]
    pub rest: Vec<u8>,
}

/// Data for one map section in a save file.
///
/// Each visited map records its monsters, NPCs, extra objects (chests, doors,
/// triggers), and items lying on the ground in five type-specific categories.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MapSectionData {
    /// Map index/ID referenced in AllMap.ini
    pub map_id: u32,
    /// Monsters present on this map
    pub monsters: Vec<MonsterRecord>,
    /// NPCs present on this map
    pub npcs: Vec<NpcRecord>,
    /// Extra objects (chests, triggers, etc.)
    pub extra_objects: Vec<ExtraObjectRecord>,
    /// Ground items — Weapon type (count × 296 bytes each)
    pub draw_items_weapon: Vec<DrawItemWeaponItem>,
    /// Ground items — Heal type (count × 264 bytes each)
    pub draw_items_heal: Vec<DrawItemHealItem>,
    /// Ground items — Edit type (count × 280 bytes each)
    pub draw_items_edit: Vec<DrawItemEditItem>,
    /// Ground items — Misc type (count × 268 bytes each)
    pub draw_items_misc: Vec<DrawItemMiscItem>,
    /// Ground items — Event type (count × 252 bytes each)
    pub draw_items_event: Vec<DrawItemEventItem>,
}

/// Parsed character stats from a save file.
///
/// Maps the binary stats block (~68 bytes of structured data) that follows
/// the belt-data section and precedes the inventory section.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

/// Raw inventory data from a save file (5 item categories).
///
/// Each category stores count-prefixed raw records of a fixed size.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventoryData {
    /// Event-type items (count × 244 bytes each)
    pub event_items: Vec<InventoryEventItem>,
    /// Misc-type items (count × 264 bytes each)
    pub misc_items: Vec<InventoryMiscItem>,
    /// Edit-type items (count × 272 bytes each)
    pub edit_items: Vec<InventoryEditItem>,
    /// Weapon-type items (count × 292 bytes each)
    pub weapon_items: Vec<InventoryWeaponItem>,
    /// Heal-type items (count × 256 bytes each)
    pub heal_items: Vec<InventoryHealItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct InventoryMiscItem {
    #[binary_record(string(encoding = "WINDOWS-1250", size = 30))]
    pub name: String,
    #[binary_record(string(encoding = "WINDOWS-1250", size = 202))]
    pub description: String,
    pub base_price: u32,
    #[binary_record(size = 16)]
    pub unknown_1: Vec<u8>,
    pub misc_item_id: u32, // misc_item_id
    pub unknown_3: u16,
    pub unknown_4: u16, // 260
    pub unknown_5: u8,  // inventory position
    pub unknown_6: u8,  // inventory position
    pub unknown_7: u16, // 264
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct DrawItemMiscItem {
    #[binary_record(string(encoding = "WINDOWS-1250", size = 30))]
    pub name: String, // 30
    #[binary_record(string(encoding = "WINDOWS-1250", size = 202))]
    pub description: String, // 232
    pub base_price: u32, // 236
    #[binary_record(size = 16)]
    pub unknown_1: Vec<u8>, // 252
    pub misc_item_id: u32,  // 256
    pub map_coordinate_x: u32,  // 260 coord-X
    pub map_coordinate_y: u32,  // 264 coord-Y
    pub unknown_7: u32,  // 268
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct InventoryEventItem {
    #[binary_record(string(encoding = "WINDOWS-1250", size = 30))]
    pub name: String, // 30
    #[binary_record(string(encoding = "WINDOWS-1250", size = 202))]
    pub description: String, // 232
    pub base_price: u32,    // 236
    pub event_item_id: u32, // 240
    pub unknown_2: u8,      // inventory position 241
    pub unknown_3: u8,      // inventory position 242
    pub unknown_4: u16,     // 244
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct DrawItemEventItem {
    #[binary_record(string(encoding = "WINDOWS-1250", size = 30))]
    pub name: String, // 30
    #[binary_record(string(encoding = "WINDOWS-1250", size = 202))]
    pub description: String, // 232
    pub base_price: u32,       // 236
    pub event_item_id: u32,    // 240
    pub map_coordinate_x: u32, // 244
    pub map_coordinate_y: u32, // 248
    pub unknown_1: u32,        // 252, event id?
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct InventoryEditItem {
    // 272
    #[binary_record(string(encoding = "WINDOWS-1250", size = 30))]
    pub name: String, // 30
    #[binary_record(string(encoding = "WINDOWS-1250", size = 202))]
    pub description: String, // 232
    pub base_price: u32,            // 236
    pub unknown_1: u16,             // 238
    pub unknown_2: u16,             // 240
    pub health_points: i16,         // 242
    pub mana_points: i16,           // 244
    pub strength: i16,              // 246
    pub agility: i16,               // 248
    pub wisdom: i16,                // 250
    pub constitution: i16,          // 252
    pub to_dodge: i16,              // 254
    pub to_hit: i16,                // 256
    pub offense: i16,               // 258
    pub defense: i16,               // 260
    pub magical_power: i16,         // 262
    pub item_destroying_power: i16, // 264
    pub unknown_3: u8,              // 265
    pub modifies_item: u8,          // 266
    pub additional_effect: i16,     // 268
    pub unknown_4: u8,              // inventory position 269
    pub unknown_5: u8,              // inventory position 270
    pub unknown_6: u16,             // 272
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct DrawItemEditItem {
    // 280
    #[binary_record(string(encoding = "WINDOWS-1250", size = 30))]
    pub name: String, // 30
    #[binary_record(string(encoding = "WINDOWS-1250", size = 202))]
    pub description: String, // 232
    pub base_price: u32,            // 236
    pub edit_item_id: u32,          // 240
    pub health_points: i16,         // 242
    pub mana_points: i16,           // 244
    pub strength: i16,              // 246
    pub agility: i16,               // 248
    pub wisdom: i16,                // 250
    pub constitution: i16,          // 252
    pub to_dodge: i16,              // 254
    pub to_hit: i16,                // 256
    pub offense: i16,               // 258
    pub defense: i16,               // 260
    pub magical_power: i16,         // 262
    pub item_destroying_power: i16, // 264
    pub unknown_3: u8,              // 265
    pub modifies_item: u8,          // 266
    pub additional_effect: i16,     // 268
    pub map_coordinate_x: u32,      // 272
    pub map_coordinate_y: u32,      // 276
    pub unknown_4: u32,             // 280
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct InventoryHealItem {
    // 256
    #[binary_record(string(encoding = "WINDOWS-1250", size = 30))]
    pub name: String,
    #[binary_record(string(encoding = "WINDOWS-1250", size = 202))]
    pub description: String, // 232
    pub base_price: u32,         // 236
    pub heal_item_id: u32,       // 240
    pub health_points: i16,      // 242
    pub mana_points: i16,        // 244
    pub restore_full_health: u8, // 245
    pub restore_full_mana: u8,   // 246
    pub poison_heal: u8,         // 247
    pub petrif_heal: u8,         // 248
    pub polimorph_heal: u8,      // 249
    pub unknown_1: u8,           // 250
    pub unknown_2: u16,          // 252
    pub unknown_3: u8,           // inventory position 253
    pub unknown_4: u8,           // inventory position 254
    pub unknown_5: u16,          // 6c 6c (108, 108) for the first row 256
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct DrawItemHealItem {
    // 264
    #[binary_record(string(encoding = "WINDOWS-1250", size = 30))]
    pub name: String,
    #[binary_record(string(encoding = "WINDOWS-1250", size = 202))]
    pub description: String, // 232
    pub base_price: u32,         // 236
    pub heal_item_id: u32,       // 240
    pub health_points: i16,      // 242
    pub mana_points: i16,        // 244
    pub restore_full_health: u8, // 245
    pub restore_full_mana: u8,   // 246
    pub poison_heal: u8,         // 247
    pub petrif_heal: u8,         // 248
    pub polimorph_heal: u8,      // 249
    pub unknown_1: u8,           // 250
    pub unknown_2: u16,          // 252
    pub map_coordinate_x: u32,   // 256
    pub map_coordinate_y: u32,   // 260
    pub unknown_3: u32,          // 264
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct InventoryWeaponItem {
    // 292
    #[binary_record(string(encoding = "WINDOWS-1250", size = 30))]
    pub name: String,
    #[binary_record(string(encoding = "WINDOWS-1250", size = 202))]
    pub description: String, // 232
    pub base_price: u32,       // 236
    pub weapon_item_id: u32,   // 240
    pub health_points: i16,    // 242
    pub mana_points: i16,      // 244
    pub strength: i16,         // 246
    pub agility: i16,          // 248
    pub wisdom: i16,           // 250
    pub constitution: i16,     // 252
    pub to_dodge: i16,         // 254
    pub to_hit: i16,           // 256
    pub attack: i16,           // 258
    pub defense: i16,          // 260
    pub magical_strength: i16, // 262
    pub durability: i16,       // 264
    pub padding2: i16,         // 266
    pub padding3: i16,         // 268
    pub req_strength: i16,     // 270
    pub padding4: i16,         // 272
    pub req_agility: i16,      // 274
    pub padding5: i16,         // 276
    pub req_wisdom: i16,       // 278
    pub padding6: i16,         // 280
    pub padding7: i16,         // 282
    pub padding8: i16,         // 284
    pub unknown_1: u32,        // 288
    pub unknown_2: u8,         // inventory position 289
    pub unknown_3: u8,         // inventory position 290
    pub unknown_4: u16,        // 292
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct DrawItemWeaponItem {
    // 296
    #[binary_record(string(encoding = "WINDOWS-1250", size = 30))]
    pub name: String,
    #[binary_record(string(encoding = "WINDOWS-1250", size = 202))]
    pub description: String, // 232
    pub base_price: u32,       // 236
    pub weapon_item_id: u32,   // 240
    pub health_points: i16,    // 242
    pub mana_points: i16,      // 244
    pub strength: i16,         // 246
    pub agility: i16,          // 248
    pub wisdom: i16,           // 250
    pub constitution: i16,     // 252
    pub to_dodge: i16,         // 254
    pub to_hit: i16,           // 256
    pub attack: i16,           // 258
    pub defense: i16,          // 260
    pub magical_strength: i16, // 262
    pub durability: i16,       // 264
    pub padding2: i16,         // 266
    pub padding3: i16,         // 268
    pub req_strength: i16,     // 270
    pub padding4: i16,         // 272
    pub req_agility: i16,      // 274
    pub padding5: i16,         // 276
    pub req_wisdom: i16,       // 278
    pub padding6: i16,         // 280
    pub padding7: i16,         // 282
    pub padding8: i16,         // 284
    pub map_coordinate_x: u32, // 288
    pub map_coordinate_y: u32, // 292
    pub unknown_1: u32,        // 296
}

/// Journal data from a save file (3 sections × 100 entries).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JournalData {
    /// Main quest entries (100 × 37 bytes)
    pub main: Vec<JournalEntry>,
    /// Side quest entries (100 × 37 bytes)
    pub side: Vec<JournalEntry>,
    /// Trading offer entries (100 × 37 bytes)
    pub trade: Vec<JournalEntry>,
}

/// Character identity data (name, class, surrounding unknown blocks).
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
    /// Large unknown data block after identity (4040 bytes).
    pub unknown_data: Vec<u8>,
}

/// Unknown data block between map data and sprite paths (section 3).
///
/// Layout: `[9 × u32 header][variable-size remainder]`.
/// The remainder size is calculated as `(10188 + 4 * num_visited_maps) - 36`.
/// The header values may encode sizes of sub-sections within the remainder
/// (monster_block_size, npc_block_size, extra_object_block_size observed as 329, 349, 200).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostMapsData {
    /// Possibly the save slot index.
    pub save_slot_id: u32,
    /// Possibly a Win32 timestamp of when this save was created.
    pub game_version: f32,
    /// 3 unknown u32 values (observed: 4, 8, 0).
    pub unknowns_a: [u32; 3],
    /// Possibly the size of the monster data block within the remainder.
    pub monster_block_size: u32,
    /// Possibly the size of the NPC data block within the remainder.
    pub npc_block_size: u32,
    /// Possibly the size of the extra object data block within the remainder.
    pub extra_object_block_size: u32,
    /// One more unknown u32 (observed: 0, sandwiched between npc and extra sizes).
    pub unknown_b: u32,
    /// The rest of the section after the header.
    pub unknown_block: Vec<u8>,
    pub number_of_visited_maps: u32,
    pub map_ids: Vec<u32>,
    pub unknown_c: [u32; 4],
}

/// Unknown data block between events and journal sections.
///
/// Structure: fixed 12 bytes + counter-prefixed 24-byte records + fixed 98 bytes.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostEventsData {
    /// Unknown fixed block (12 bytes).
    pub block_a: Vec<u8>,
    /// Unknown records (counter × 24 bytes each).
    pub records: Vec<u8>,
    /// Unknown fixed block (98 bytes).
    pub block_b: Vec<u8>,
}

/// Complete save file structure.
///
/// More fields will be added in future phases.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveFile {
    /// Jump address after all map data (first 4 bytes of the file).
    /// The maps section is followed by alignment to this address.
    pub jump_addr_after_maps: u32,
    /// Per-map world state.
    pub maps: Vec<MapSectionData>,
    /// Unknown data between maps and sprite paths (header + variable-size remainder).
    pub post_maps: PostMapsData,
    /// Character sprite paths (4 × 60-byte WINDOWS-1250 strings).
    pub sprite_paths: Vec<String>,
    /// Raw belt/quick-slot data (40 bytes before character stats).
    pub unknown_before_stats: Vec<u8>,
    /// Parsed character stats (core, combat, skills, weapon skills).
    pub character_stats: CharacterStats,
    /// Unknown bytes after stats block (9 bytes).
    pub unknown_after_stats: Vec<u8>,
    /// Raw inventory data (5 item categories).
    pub inventory: InventoryData,
    /// Journal entries (main, side, trade — 100 entries each).
    pub journal: JournalData,
    /// Event scripts (2251 × 284 bytes).
    pub events: Vec<EventScript>,
    /// Character identity (name, class, unknown blocks).
    pub character_identity: CharacterIdentity,
    /// Unknown data between events and journal (3 sub-blocks).
    pub post_events: PostEventsData,
}

impl SaveFile {
    /// Parse complete save file from binary data
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        let mut reader = std::io::Cursor::new(data);

        // ── 1. HEADER (4 bytes) ──
        let jump_addr_after_maps = reader.read_u32::<LittleEndian>()? as usize;

        // ── 2. Maps ──
        let number_of_visited_map = reader.read_u32::<LittleEndian>()?;
        let maps = Self::parse_maps_section(&mut reader, number_of_visited_map)?;

        if jump_addr_after_maps != reader.position() as usize {
            eprintln!(
                "jump_addr_after_maps ({:?}) != reader.position() {:?}",
                jump_addr_after_maps,
                reader.position() as usize
            );

            reader.set_position(jump_addr_after_maps as u64);
        }

        // ── 3. Unknown data between maps and sprite paths ──
        let post_maps = Self::parse_post_maps_data(&mut reader, number_of_visited_map)?;

        // ── 4. Character sprite paths (4 × 60-byte WINDOWS-1250 strings) ──
        let sprite_paths = Self::parse_sprite_paths(&mut reader)?;

        // ── 5 Character stats ──
        let (unknown_before_stats, character_stats, unknown_after_stats) =
            Self::parse_character_stats(&mut reader)?;

        // ── 6. Inventory (5 categories, each count-prefixed) ──
        let inventory = Self::parse_inventory_section(&mut reader)?;

        // ── 7. Character identity (unknown block + name + class + large unknown) ──
        let character_identity = Self::parse_character_identity(&mut reader)?;

        // ── 8. Events (2251 × 284 bytes) ──
        let events = Self::parse_events_section(&mut reader)?;

        // ── 9. Unknown data between events and journal ──
        let post_events = Self::parse_post_events_data(&mut reader)?;

        // ── 10. Journal (3 sections × 100 × 37 bytes) ──
        let journal = Self::parse_journal_section(&mut reader)?;

        Ok(SaveFile {
            jump_addr_after_maps: jump_addr_after_maps as u32,
            maps,
            post_maps,
            sprite_paths,
            unknown_before_stats,
            character_stats,
            unknown_after_stats,
            inventory,
            character_identity,
            events,
            post_events,
            journal,
        })
    }

    /// Generic count-prefixed item section reader.
    ///
    /// Each section is stored as `[count: u16][count × record_size bytes]`.
    /// Parses each record via the provided `parse` function.
    fn read_item_section<R: Read, T>(
        reader: &mut R,
        record_size: usize,
        parse: fn(&[u8]) -> std::io::Result<T>,
    ) -> std::io::Result<Vec<T>> {
        let count = reader.read_u16::<LittleEndian>()? as usize;
        let mut data = vec![0u8; count * record_size];
        reader.read_exact(&mut data)?;

        data.chunks_exact(record_size).map(parse).collect()
    }

    /// Parse all map sections from the reader.
    ///
    /// Each map has:
    ///   `[map_id: u32][monsters][npcs][sep: u32][extra_objects][sep: 11B]
    ///    [draw_items_weapon][draw_items_heal][draw_items_edit]
    ///    [draw_items_misc][draw_items_event][end_sep: u32]`
    fn parse_maps_section<R: Read>(
        reader: &mut R,
        map_count: u32,
    ) -> std::io::Result<Vec<MapSectionData>> {
        let mut maps = Vec::with_capacity(map_count as usize);

        for _ in 0..map_count {
            let map_id = reader.read_u32::<LittleEndian>()?;

            // ── 2.1. Monsters ──
            let monster_count = reader.read_u32::<LittleEndian>()? as usize;
            let mut monsters_data = vec![0u8; monster_count * 329];
            reader.read_exact(&mut monsters_data)?;
            let monsters = monsters_data
                .chunks_exact(329)
                .map(MonsterRecord::parse)
                .collect::<std::io::Result<Vec<_>>>()?;

            // ── 2.2. NPCs ──
            let npc_count = reader.read_u32::<LittleEndian>()? as usize;
            let mut npcs_data = vec![0u8; npc_count * 349];
            reader.read_exact(&mut npcs_data)?;
            let npcs = npcs_data
                .chunks_exact(349)
                .map(NpcRecord::parse)
                .collect::<std::io::Result<Vec<_>>>()?;

            // ── 2.3. Separator (always 0) ──
            let _separator = reader.read_u32::<LittleEndian>()?;

            // ── 2.4. Extra objects ──
            let extras_count = reader.read_u32::<LittleEndian>()? as usize;
            let mut extras_data = vec![0u8; extras_count * 200];
            reader.read_exact(&mut extras_data)?;
            let extra_objects = extras_data
                .chunks_exact(200)
                .map(ExtraObjectRecord::parse)
                .collect::<std::io::Result<Vec<_>>>()?;

            // ── 2.5. Separator (11 bytes, unknown meaning) ──
            let mut _separator = vec![0u8; 11];
            reader.read_exact(&mut _separator)?;

            // ── 2.6–2.10. Ground items (5 types) ──
            let draw_items_weapon =
                Self::read_item_section(reader, 296, DrawItemWeaponItem::parse)?;
            let draw_items_heal = Self::read_item_section(reader, 264, DrawItemHealItem::parse)?;
            let draw_items_edit = Self::read_item_section(reader, 280, DrawItemEditItem::parse)?;
            let draw_items_misc = Self::read_item_section(reader, 268, DrawItemMiscItem::parse)?;
            let draw_items_event = Self::read_item_section(reader, 252, DrawItemEventItem::parse)?;

            // ── 2.11. End-of-map separator (always 0) ──
            let _separator = reader.read_u32::<LittleEndian>()?;

            maps.push(MapSectionData {
                map_id,
                monsters,
                npcs,
                extra_objects,
                draw_items_weapon,
                draw_items_heal,
                draw_items_edit,
                draw_items_misc,
                draw_items_event,
            });
        }

        Ok(maps)
    }

    /// Parse the unknown data block between maps and sprite paths.
    ///
    /// Layout: `[9 × u32 header][variable-size remainder]`
    fn parse_post_maps_data<R: Read>(
        reader: &mut R,
        num_visited_maps: u32,
    ) -> std::io::Result<PostMapsData> {
        let maybe_save_slot_id= reader.read_u32::<LittleEndian>()?;
        let game_version = reader.read_f32::<LittleEndian>()?;
        let header = [
            reader.read_u32::<LittleEndian>()?, // 0: observed 4 or 6
            reader.read_u32::<LittleEndian>()?, // 1: observed 8 or 12
            reader.read_u32::<LittleEndian>()?, // 2: observed 0
            reader.read_u32::<LittleEndian>()?, // 3: monster_block_size (observed 329)
            reader.read_u32::<LittleEndian>()?, // 4: npc_block_size (observed 349)
            reader.read_u32::<LittleEndian>()?, // 5: observed 0
            reader.read_u32::<LittleEndian>()?, // 6: extra_object_block_size (observed 200)
            reader.read_u32::<LittleEndian>()?, // 7: number of visited maps
        ];

        let mut map_ids = vec![0u32; num_visited_maps as usize];
        for i in 0..num_visited_maps as usize {
            let map_id = reader.read_u32::<LittleEndian>()?;
            map_ids[i] = map_id;
        }

        let unknown_c = [
            reader.read_u32::<LittleEndian>()?, // 0: observed 128
            reader.read_u32::<LittleEndian>()?, // 0: observed 64
            reader.read_u32::<LittleEndian>()?, // 0: observed 768
            reader.read_u32::<LittleEndian>()?, // 0: observed 544
        ];

        let remainder = 10132;
        let mut unknown_block = vec![0u8; remainder];
        reader.read_exact(&mut unknown_block)?;

        Ok(PostMapsData {
            save_slot_id: maybe_save_slot_id,
            game_version,
            unknowns_a: [header[0], header[1], header[2]],
            monster_block_size: header[3],
            npc_block_size: header[4],
            extra_object_block_size: header[6],
            unknown_b: header[5],
            number_of_visited_maps: header[7],
            unknown_c,
            map_ids,
            unknown_block,
        })
    }

    /// Parse the 4 character sprite paths (4 × 60-byte fixed buffers).
    ///
    /// Each path is a null-terminated WINDOWS-1250 string, e.g.
    /// `"inter\\m_bald.spr"` or `"CharacterInGame\\m_warrior.spr"`.
    fn parse_sprite_paths<R: Read>(reader: &mut R) -> std::io::Result<Vec<String>> {
        let mut paths = Vec::with_capacity(4);
        for _ in 0..4 {
            let mut buf = [0u8; 60];
            reader.read_exact(&mut buf)?;
            paths.push(
                read_null_terminated_windows_1250(&buf)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?,
            );
        }
        Ok(paths)
    }

    /// Parse belt data, character stats, and trailing unknown bytes.
    ///
    /// Layout:
    ///   `[unknown_before_stats: 40B][strength u16][agility u16][wisdom u16][constitution u16]
    ///    [morale u16][hp_cur u16][hp_max u16][mp_cur u16][mp_max u16]
    ///    [xp u32][level u16][gold u32][offense u16][defense u16]
    ///    [dodge u8][hit u8][magic_power u16][attack_mod u8]
    ///    [thievery u8][lockpick u8][haggle u8][perception u8][traps u8]
    ///    [sword_lv u8][sword_kills u16][axe_lv u8][axe_kills u16]
    ///    [archery_lv u8][archery_kills u16][polearm_lv u8][polearm_kills u16]
    ///    [magic_lv u8][magic_kills u16][holy_lv u8][holy_kills u16]
    ///    [dark_lv u8][dark_kills u16][unknown: 9B]`
    fn parse_character_stats<R: Read>(
        reader: &mut R,
    ) -> std::io::Result<(Vec<u8>, CharacterStats, Vec<u8>)> {
        // ── Leading data (40 bytes, purpose unknown) ──
        let mut unknown_before_stats = vec![0u8; 40];
        reader.read_exact(&mut unknown_before_stats)?;

        // ── Structured stats block ──
        let character_stats = CharacterStats {
            strength: reader.read_u16::<LittleEndian>()?,
            agility: reader.read_u16::<LittleEndian>()?,
            wisdom: reader.read_u16::<LittleEndian>()?,
            constitution: reader.read_u16::<LittleEndian>()?,
            morale: reader.read_u16::<LittleEndian>()?,
            hp_current: reader.read_u16::<LittleEndian>()?,
            hp_maximum: reader.read_u16::<LittleEndian>()?,
            mp_current: reader.read_u16::<LittleEndian>()?,
            mp_maximum: reader.read_u16::<LittleEndian>()?,
            experience: reader.read_u32::<LittleEndian>()?,
            level: reader.read_u16::<LittleEndian>()?,
            gold: reader.read_u32::<LittleEndian>()?,
            offense: reader.read_u16::<LittleEndian>()?,
            defense: reader.read_u16::<LittleEndian>()?,
            dodge_rate: reader.read_u8()?,
            hit_rate: reader.read_u8()?,
            magic_power: reader.read_u16::<LittleEndian>()?,
            attack_modifier: reader.read_u8()?,
            pickpocketing: reader.read_u8()?,
            lockpicking: reader.read_u8()?,
            haggling: reader.read_u8()?,
            perception: reader.read_u8()?,
            traps: reader.read_u8()?,
            swords_level: reader.read_u8()?,
            swords_kills: reader.read_u16::<LittleEndian>()?,
            axes_level: reader.read_u8()?,
            axes_kills: reader.read_u16::<LittleEndian>()?,
            archery_level: reader.read_u8()?,
            archery_kills: reader.read_u16::<LittleEndian>()?,
            polearm_level: reader.read_u8()?,
            polearm_kills: reader.read_u16::<LittleEndian>()?,
            magic_level: reader.read_u8()?,
            magic_kills: reader.read_u16::<LittleEndian>()?,
            holy_magic_level: reader.read_u8()?,
            holy_magic_kills: reader.read_u16::<LittleEndian>()?,
            dark_magic_level: reader.read_u8()?,
            dark_magic_kills: reader.read_u16::<LittleEndian>()?,
        };

        // ── Trailing unknown bytes ──
        let mut unknown_after_stats = vec![0u8; 9];
        reader.read_exact(&mut unknown_after_stats)?;

        Ok((unknown_before_stats, character_stats, unknown_after_stats))
    }

    /// Parse the inventory section (5 count-prefixed item categories).
    ///
    /// Record sizes: Event=244, Misc=264, Edit=272, Weapon=292, Heal=256.
    fn parse_inventory_section<R: Read>(reader: &mut R) -> std::io::Result<InventoryData> {
        Ok(InventoryData {
            event_items: Self::read_item_section(reader, 244, InventoryEventItem::parse)?,
            misc_items: Self::read_item_section(reader, 264, InventoryMiscItem::parse)?,
            edit_items: Self::read_item_section(reader, 272, InventoryEditItem::parse)?,
            weapon_items: Self::read_item_section(reader, 292, InventoryWeaponItem::parse)?,
            heal_items: Self::read_item_section(reader, 256, InventoryHealItem::parse)?,
        })
    }

    /// Parse the journal section (3 × 100 × 37-byte entries).
    fn parse_journal_section<R: Read>(reader: &mut R) -> std::io::Result<JournalData> {
        const ENTRY_SIZE: usize = 37;
        const ENTRIES_PER_SECTION: usize = 100;
        const SECTION_SIZE: usize = ENTRY_SIZE * ENTRIES_PER_SECTION; // 3700

        let mut raw = vec![0u8; SECTION_SIZE];
        reader.read_exact(&mut raw)?;
        let main = Self::parse_journal_entries(&raw, ENTRIES_PER_SECTION)?;

        let mut raw = vec![0u8; SECTION_SIZE];
        reader.read_exact(&mut raw)?;
        let side = Self::parse_journal_entries(&raw, ENTRIES_PER_SECTION)?;

        let mut raw = vec![0u8; SECTION_SIZE];
        reader.read_exact(&mut raw)?;
        let trade = Self::parse_journal_entries(&raw, ENTRIES_PER_SECTION)?;

        Ok(JournalData { main, side, trade })
    }

    /// Parse the events section (2251 × 284-byte event records).
    fn parse_events_section<R: Read>(reader: &mut R) -> std::io::Result<Vec<EventScript>> {
        const EVENT_COUNT: usize = 2251;
        const EVENT_SIZE: usize = 284;

        let mut events = Vec::with_capacity(EVENT_COUNT);
        for _ in 0..EVENT_COUNT {
            let mut buf = [0u8; EVENT_SIZE];
            reader.read_exact(&mut buf)?;
            events.push(EventScript::parse(&buf)?);
        }
        Ok(events)
    }

    /// Parse character identity (unknown block + name + class + large unknown).
    ///
    /// Layout:
    ///   `[unknown_96B][name: 11B][class_id: u16][class_name: 11B][unknown_4040B]`
    fn parse_character_identity<R: Read>(reader: &mut R) -> std::io::Result<CharacterIdentity> {
        // ── 7.1. Unknown block (96 bytes before name) ──
        let mut unknown_block = vec![0u8; 96];
        reader.read_exact(&mut unknown_block)?;

        // ── 7.2. Player name (11-byte WINDOWS-1250 null-terminated) ──
        let mut name_raw = vec![0u8; 11];
        reader.read_exact(&mut name_raw)?;
        let player_name = read_null_terminated_windows_1250(&name_raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // ── 7.3. Player class ──
        let player_class_id = reader.read_u16::<LittleEndian>()?;
        let mut class_raw = vec![0u8; 11];
        reader.read_exact(&mut class_raw)?;
        let player_class_name = read_null_terminated_windows_1250(&class_raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // ── 7.4. Large unknown data block ──
        let mut unknown_data = vec![0u8; 4040];
        reader.read_exact(&mut unknown_data)?;

        Ok(CharacterIdentity {
            unknown_block,
            player_name,
            player_class_id,
            player_class_name,
            unknown_data,
        })
    }

    /// Parse the unknown section between events and journal.
    ///
    /// Layout: `[block_a: 12B][count: u32][count × 24B records][block_b: 98B]`
    fn parse_post_events_data<R: Read>(reader: &mut R) -> std::io::Result<PostEventsData> {
        let mut block_a = vec![0u8; 12];
        reader.read_exact(&mut block_a)?;

        let count = reader.read_u32::<LittleEndian>()? as usize;
        let mut records = vec![0u8; count * 24];
        reader.read_exact(&mut records)?;

        let mut block_b = vec![0u8; 98];
        reader.read_exact(&mut block_b)?;

        Ok(PostEventsData {
            block_a,
            records,
            block_b,
        })
    }

    /// Parse journal entries from raw binary data
    fn parse_journal_entries(data: &[u8], count: usize) -> std::io::Result<Vec<JournalEntry>> {
        let expected_len = count * 37;
        if data.len() < expected_len {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Journal data too short",
            ));
        }

        let mut entries = Vec::with_capacity(count);
        for i in 0..count {
            let offset = i * 37;
            let entry = JournalEntry::parse(&data[offset..offset + 37])?;
            entries.push(entry);
        }

        Ok(entries)
    }
}

impl Extractor for SaveFile {
    fn parse<R: Read + Seek>(reader: &mut R, _len: u64) -> std::io::Result<Vec<Self>> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        let save = SaveFile::parse(&data)?;
        Ok(vec![save])
    }

    fn to_writer<W: Write>(records: &[Self], _writer: &mut W) -> std::io::Result<()> {
        if records.len() != 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "SaveFile can only serialize one record at a time",
            ));
        }

        // let save = &records[0];

        // // Write header (12 bytes)
        // writer.write_all(&save.header)?;
        //
        // // Write surface monster count + raw data
        // writer.write_u32::<LittleEndian>(save.surface_monsters.len() as u32)?;
        // writer.write_all(&save.monsters_data)?;
        //
        // // Write NPC count + raw data
        // writer.write_u32::<LittleEndian>(save.npcs.len() as u32)?;
        // writer.write_all(&save.npcs_data)?;
        //
        // // Write surface objects: separator (u32=0) + count + raw data
        // writer.write_u32::<LittleEndian>(0)?;
        // writer.write_u32::<LittleEndian>(save.surface_objects.len() as u32)?;
        // writer.write_all(&save.extras_data)?;
        //
        // // Write remaining data (everything from after surface objects to EOF)
        // writer.write_all(&save.remaining_data)?;

        Ok(())
    }
}
