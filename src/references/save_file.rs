// Save file extraction and parsing for Dispel RPG
//
// This module provides comprehensive parsing of Dispel RPG save files (.sav)
// following the binary format documented in SAVE_FILE_RESEARCH.md

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use dispel_macros::BinaryRecord;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, Write};
// use proptest::char::range;
use super::extractor::{Extractor, read_null_terminated_windows_1250};

/// Fixed size of the player runtime-state snapshot stored after the map-ID list.
pub const PLAYER_RUNTIME_STATE_SIZE: usize = 10_148;

/// Monster record from save file (surface or dungeon)
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct MonsterRecord {
    /// Runtime state. Zero is alive, one appears to be patrolling, and eight is dead.
    pub monster_state: u32,
    /// Index of this monster in the map record list.
    pub record_index: u32,
    /// ID of the current sprite frame.
    pub sprite_frame_id: u32,
    /// Monster name in Windows-1250 encoding.
    #[binary_record(string(encoding = "WINDOWS-1250", size = 21))]
    pub name: String,
    /// Zero-based ID of the Monster.db record.
    pub monster_db_id: u32,
    /// Current health points.
    pub hp_current: u16,
    /// Maximum health points.
    pub hp_maximum: u16,
    /// Current mana points.
    pub mp_current: u16,
    /// Maximum mana points.
    pub mp_maximum: u16,
    /// Movement speed.
    pub walk_speed: u8,
    /// Hit rate.
    pub hit_rate: u8,
    /// Dodge rate.
    pub dodge_rate: u8,
    /// Physical offense rate.
    pub offense_rate: u16,
    /// Physical defense rate.
    pub defense_rate: u16,
    /// Magic rate.
    pub magic_rate: u16,
    /// Set for undead monsters.
    pub is_undead: u8,
    /// Set for monsters that have blood.
    pub has_blood: u8,
    /// AI type from Monster.db. MonsterRef can override it for a new monster.
    pub monster_ai_type: u8,
    /// Experience awarded when this monster dies.
    pub experience_on_kill: u16,
    /// Gold awarded when this monster dies.
    pub gold_drop_on_kill: u16,
    /// Chase distance from Monster.db.
    pub distance_range_size: u8,
    /// Detection distance from Monster.db.
    pub detection_sight_size: u8,
    /// Computed aggression flag: zero for AI types 5/6, one otherwise.
    pub aggression_flag: u8,
    pub spell_slot_1: i8,
    pub spell_slot_2: i8,
    pub spell_slot_3: i8,
    pub oversize: u8,
    /// Magic level from Monster.db.
    pub magic_level: u32,
    /// Countdown used while scanning/patrolling; initialized from MonsterRef padding 1.
    pub patrol_countdown: u8,
    /// Behaviour flag; one skips an AI action. Initialized from MonsterRef padding 2.
    pub behavior_flag: u8,
    /// Current AI state (`0xff` means not spawned).
    pub ai_state: u8,
    /// Current AI sub-state (`0xfc` is a runtime marker).
    pub ai_sub_state: u8,
    /// Current movement direction.
    pub movement_direction: u8,
    /// Target tile X coordinate.
    pub target_position_x: u32,
    /// Target tile Y coordinate.
    pub target_position_y: u32,
    pub unknown_runtime_1: u32,
    pub unknown_runtime_2: u32,
    /// Active/awake flag, initialized from MonsterRef padding 3.
    pub awake_flag: u8,
    pub unknown_runtime_3: u32,
    /// Event ID that runs when this monster dies.
    pub event_id_on_kill: u32,
    /// An unknown constructor field. The constructor initializes it to `-1`.
    pub unknown_5: i32,
    /// Current tile X coordinate.
    pub current_position_x: u16,
    /// Current tile Y coordinate.
    pub current_position_y: u16,
    /// Spawn tile X coordinate.
    pub spawn_position_x: u16,
    /// Spawn tile Y coordinate.
    pub spawn_position_y: u16,
    /// Home tile X coordinate used for respawn.
    pub home_position_x: u16,
    /// Home tile Y coordinate used for respawn.
    pub home_position_y: u16,
    pub unknown_patrol_flag: u8,
    /// This value is cleared when the monster dies.
    pub unknown_cleared_on_death_1: u8,
    /// This value is cleared when the monster dies.
    pub unknown_cleared_on_death_2: u8,
    /// Spawn/group ID.
    pub spawn_group_id: u16,
    /// Constructor-initialized to `0xff`.
    pub constructor_marker: u8,
    /// This value is cleared when the monster dies.
    pub unknown_cleared_on_death_3: u8,
    /// Set when the monster is dead or removed.
    pub dead_or_removed_flag: u8,
    pub unknown_runtime_flag_0: u8,
    /// Unknown value loaded from map data.
    pub unknown_map_data: u32,
    pub unknown_runtime_4: u32,
    pub unknown_runtime_5: u32,
    pub unknown_runtime_flag_1: u8,
    pub unknown_runtime_6: u32,
    pub unknown_runtime_flag_2: u8,
    pub unknown_runtime_7: u32,
    /// An unknown constructor field. The constructor initializes it to `-1`.
    pub constructor_unknown_negative_one: i32,
    /// Whether the following path-buffer position is present.
    pub path_buffer_present_flag: u32,
    /// This value is cleared when the monster dies.
    pub unknown_cleared_on_death_4: u32,
    /// First item that this monster can drop.
    #[binary_record(inventory_item(wire_type = "i32"))]
    pub loot_item1: crate::references::enums::InventoryItem,
    /// Second item that this monster can drop.
    #[binary_record(inventory_item(wire_type = "i32"))]
    pub loot_item2: crate::references::enums::InventoryItem,
    /// Third item that this monster can drop.
    #[binary_record(inventory_item(wire_type = "i32"))]
    pub loot_item3: crate::references::enums::InventoryItem,
    /// MonsterRef `force_ai_update`. The save format stores it before `drop_all_loot`.
    pub force_ai_update: u32,
    /// MonsterRef `drop_all_loot`. The save format stores it after `force_ai_update`.
    pub drop_all_loot: u32,
    /// Initialized to 12,000 by the constructor.
    pub respawn_timer: u32,
    pub unknown_runtime_8: u32,
    pub unknown_runtime_9: u32,
    /// Special attack ID from Monster.db.
    pub special_attack: u32,
    /// Chance that the monster uses its special attack.
    pub special_attack_chance: u32,
    /// Duration of the special attack.
    pub special_attack_duration: u32,
    pub unknown_runtime_10: u32,
    pub unknown_runtime_11: u32,
    /// Boldness value from Monster.db.
    pub boldness: u32,
    /// Attack speed from Monster.db.
    pub attack_speed: u32,
    /// One for guard monsters.
    pub guard_flag: u8,
    pub unknown_runtime_flag_3: u8,
    pub unknown_runtime_12: u32,
    pub unknown_runtime_flag_4: u8,
    pub unknown_runtime_13: u32,
    pub unknown_runtime_14: u32,
    pub unknown_runtime_flag_5: u8,
    pub unknown_runtime_15: u32,
    /// AI update/tick counter.
    pub ai_tick_counter: u32,
    /// Backup of `detection_sight_size`.
    pub sight_backup: u8,
    /// Backup of `patrol_countdown`.
    pub patrol_countdown_backup: u8,
    /// Hides the monster from the active list when set.
    pub hidden_or_delisted_flag: u8,
    /// Path-buffer tile X coordinate.
    pub path_buffer_position_x: u32,
    /// Path-buffer tile Y coordinate.
    pub path_buffer_position_y: u32,
    /// A nested 72-byte summon record follows when this is non-zero.
    pub nested_summon_flag: u8,
    /// Opaque nested summon record. No observed saves contain one yet.
    #[binary_record(size = 72)]
    pub nested_summon_record: Vec<u8>,
}

/// NPC record from save file (349 bytes)
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct NpcRecord {
    /// NPC name in Windows-1250 encoding.
    #[binary_record(string(encoding = "WINDOWS-1250", size = 64))]
    pub name: String,
    /// NPC role or description in Windows-1250 encoding.
    #[binary_record(string(encoding = "WINDOWS-1250", size = 64))]
    pub role_description: String,
    pub movement_state: u32,
    pub tile_data_entry: u32,
    pub path_progress: u32,
    pub current_position_x: u16,
    pub current_position_y: u16,
    pub last_position_x: u16,
    pub last_position_y: u16,
    pub target_position_x: u16,
    pub target_position_y: u16,
    pub path_destination_x: u16,
    pub path_destination_y: u16,
    /// Runtime render parameter.
    pub render_direction_flag: u8,
    pub cell_offset_x: u8,
    pub cell_offset_y: u8,
    /// Persistent NPC index plus 500.
    pub map_npc_index_plus_500: u16,
    pub runtime_state_78: u8,
    pub runtime_state_79: u8,
    pub path_handle: u32,
    pub path_step_counter: u32,
    /// NPC ID from the NpcRef record.
    pub npc_ini_id: u8,
    pub patrol_waypoint_count: u8,
    pub current_patrol_waypoint_index: u8,
    pub unknown_runtime_7d: u8,
    pub unknown_runtime_7e: u8,
    pub unknown_runtime_7f: u8,
    pub unknown_runtime_80: u8,
    pub current_waypoint_index: u8,
    pub unknown_runtime_82: u8,
    pub wait_tick_counter: u32,
    pub unknown_runtime_90: u32,
    pub unknown_runtime_94: u32,
    /// Party-member slot from NpcRef.
    pub npc_ref_party_member_slot: u8,
    /// Event ID that controls NPC visibility.
    pub npc_ref_show_on_event_id: u32,
    /// NpcRef movement mode: static, waypoint, or random-in-rectangle.
    pub npc_ref_movement_mode: u8,
    pub waypoint1_filled: u32,
    pub waypoint1_x: u32,
    pub waypoint1_y: u32,
    pub waypoint1_wait_time: u32,
    pub waypoint1_facing_direction: u32,
    pub waypoint1_reserved: u32,
    pub waypoint2_filled: u32,
    pub waypoint2_x: u32,
    pub waypoint2_y: u32,
    pub waypoint2_wait_time: u32,
    pub waypoint2_facing_direction: u32,
    pub waypoint2_reserved: u32,
    pub waypoint3_filled: u32,
    pub waypoint3_x: u32,
    pub waypoint3_y: u32,
    pub waypoint3_wait_time: u32,
    pub waypoint3_facing_direction: u32,
    pub waypoint3_reserved: u32,
    pub waypoint4_filled: u32,
    pub waypoint4_x: u32,
    pub waypoint4_y: u32,
    pub waypoint4_wait_time: u32,
    pub waypoint4_facing_direction: u32,
    pub waypoint4_reserved: u32,
    /// Activation rectangle, first X coordinate.
    pub activation_rect_x1: u32,
    /// Activation rectangle, first Y coordinate.
    pub activation_rect_y1: u32,
    /// Activation rectangle, second X coordinate.
    pub activation_rect_x2: u32,
    /// Activation rectangle, second Y coordinate.
    pub activation_rect_y2: u32,
    /// NpcRef interaction mode.
    pub npc_ref_interaction_mode: u8,
    /// Packed NpcRef interaction result (`item | parameter << 16`).
    pub npc_ref_interaction_result: u32,
    /// NpcRef interaction range offset plus one.
    pub npc_ref_interaction_range: u8,
    pub npc_ref_dialog_id: u32,
    pub dialogue_face_sprite_id: u8,
    /// Zero is normal movement. One moves to the target.
    pub move_mode: u32,
    pub unknown_runtime_1ac: u32,
    pub runtime_target_position_x: u32,
    pub runtime_target_position_y: u32,
    pub unknown_runtime_1b8: u32,
    pub freeze_flag: u32,
    pub freeze_counter: u32,
}

/// Extra object record (200-byte data per record)
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct ExtraObjectRecord {
    /// Active render-state slot (0–2).
    ///
    /// The engine selects this from the object type and activation state, then
    /// uses it to index the object's render-state table.
    pub render_state_slot: u32,
    /// Render variant selected from [`Self::render_state_slot`].
    ///
    /// This is the table value that chooses the sprite/renderer variant saved
    /// for the active render-state slot.
    pub render_variant_index: u32,
    /// Current frame index in the object's sprite animation.
    pub current_sprite_frame: u32,
    /// Maps to `ExtraRef.map_object_id`.
    pub map_object_id: u16,
    /// Extra.ini ID - Extra.ini stores the canonical `id` field; every named
    /// extra in the save maps to exactly one Extra.ini record via this value
    /// (e.g. extra_definition_id=1 -> chest1.spr, 2 -> door.spr)
    pub extra_definition_id: u8,
    #[binary_record(string(encoding = "WINDOWS-1250", size = 32))]
    pub object_name: String,
    pub object_type: u8,
    /// Tile coordinate X — structural parallel to `ExtraRef.map_x`.
    pub map_x: u32,
    /// Tile coordinate Y — structural parallel to `ExtraRef.map_y`.
    pub map_y: u32,
    /// Structural parallel to `ExtraRef.direction`.
    pub direction: u8,
    // Always 205, 205, 205
    pub direction_padding: [u8; 3],
    /// `ExtraRef.interaction_state`; the object's mutable activation state.
    pub interaction_state: u32,
    /// Key/requirement configuration from `ExtraRef.requires_key`.
    pub requires_key: u32,
    /// Packed `ExtraRef.required_item` followed by its two-byte padding.
    pub required_item_and_padding: u32,
    /// Packed `ExtraRef.required_item2` followed by its two-byte padding.
    pub required_item2_and_padding: u32,
    pub requirement_range_2_start: u32,
    pub requirement_range_2_end: u32,
    pub requirement_range_3_start: u32,
    pub requirement_range_3_end: u32,
    pub gold_amount: u32,
    /// Packed `ExtraRef.loot_item` followed by its two-byte padding.
    pub loot_item_and_padding: u32,
    pub loot_item_count: u32,
    pub additional_loot_1: u32,
    pub additional_loot_1_count: u32,
    pub additional_loot_2: u32,
    /// Third loot quantity plus interaction configuration. See `ExtraRef`.
    pub additional_loot_2_count_and_config: [u8; 28],
    pub interaction_event_id: u32,
    pub interaction_message_id: u32,
    pub footprint_width: u32,
    pub footprint_height: u32,
    pub footprint_orientation: u8,
    /// `ExtraRef.interaction_range`.
    pub interaction_range: u8,
    pub interaction_range_padding: [u8; 2],
    pub is_quest_element: u32,
    pub post_activation_tile_flag: u32,
    pub post_activation_footprint_mode: u32,
    pub preserve_final_sprite_frame: u32,
    pub alternate_render_mode: u32,
    pub activation_effect_id: u8,
    pub unresolved_activation_effect_flag: u8,
    pub activation_effect_padding: i16,
    pub active_overlay_enabled: u32,
    pub map_object_active: u32,
    /// Pending interaction latch.
    ///
    /// The engine sets this when activation is requested, processes the
    /// object-specific interaction on the next update, then clears it.
    pub interaction_pending: u32,
}

/// A pending ground-item placement (24 bytes).
///
/// The game creates these records when an entity drops an item. It later
/// materializes them in one of the five ground-item sections.
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct ExtraObjectTrailerRecord {
    /// Ground-item category: 1 weapon, 2 heal, 3 edit, 4 misc, or 5 event.
    pub item_category: u8,
    /// Reserved constructor byte. Preserve it verbatim.
    pub reserved_1: u8,
    /// Index across all five ground-item categories.
    pub global_item_index: u16,
    /// Number of placement attempts already made.
    pub placement_attempt_count: u8,
    /// Maximum placement attempts; the constructor initializes it to three.
    pub placement_attempt_limit: u8,
    /// These bytes are not initialized by the constructor and must be preserved.
    pub unknown_6_7: [u8; 2],
    /// Index into the selected category's ground-item section.
    pub category_item_index: u32,
    /// ID of the entity that created this pending item placement.
    pub source_entity_id: u16,
    /// These bytes are not initialized by the constructor and must be preserved.
    pub unknown_14_15: [u8; 2],
    /// Map X coordinate of the deferred item.
    pub map_x: i32,
    /// Map Y coordinate of the deferred item.
    pub map_y: i32,
}

/// Ground-item manager data after a map's extra-object records.
///
/// Its seven-byte header contains a pending-placement count and five runtime
/// control bytes. Including the five empty ground-item section counts, its
/// smallest payload is 17 bytes. `tail_size` excludes its own four bytes and
/// covers everything through the five item sections.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MapExtraObjectsTrailer {
    pub tail_size: u32,
    pub records: Vec<ExtraObjectTrailerRecord>,
    /// Runtime flag used while placing an item automatically.
    pub automatic_placement_active: u8,
    /// Runtime value used while placing an item automatically.
    pub automatic_placement_value: u16,
    /// Global item index used while placing an item automatically.
    pub automatic_placement_global_item_index: u16,
}

/// PartyMember is 321 bytes long
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct PartyMember {
    #[binary_record(string(encoding = "WINDOWS-1250", size = 21))]
    pub name: String,
    #[binary_record(size = 300)]
    pub unknown_1: Vec<u8>,
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
    /// Zero-based slot within this journal section.
    pub entry_index: u8,
    /// Title copied from the corresponding `Quest.scr` entry.
    #[binary_record(string(encoding = "WINDOWS-1250", size = 24))]
    pub quest_title: String,
    /// Eight bytes of quest-specific state. The game does not access them in the
    /// journal code path, so their individual meanings are not yet known.
    pub quest_state: [u8; 8],
    /// ID of the quest from `ExtraInGame/Quest.scr`.
    pub quest_id: u8,
    /// Quest ID recorded when this quest advances to its first follow-up stage (multi-stage quest).
    pub progress_quest_id_1: u8,
    /// Quest ID recorded when this quest advances to its second follow-up stage (multi-stage quest).
    pub progress_quest_id_2: u8,
    /// Set when the game marks this journal entry as completed.
    pub is_completed: u8,
}

/// The 42-byte journal header before the three 100-entry journal sections.
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct JournalHeader {
    /// Runtime flag controlled by the journal UI; the game meaning is unknown.
    pub runtime_unknown_flag: u8,
    /// Journal section selected by the UI (zero-based).
    pub selected_section: u8,
    /// Per-section, per-visible-row selection flags (three sections × ten rows).
    pub visible_entry_selection_flags: [u8; 30],
    /// Journal section currently being displayed (zero-based).
    pub active_section: u8,
    /// Page offset in each journal section.
    pub section_page_offsets: [u8; 3],
    /// Selected entry offset in each journal section.
    pub section_selected_entry_offsets: [u8; 3],
    /// Number of active entries in main, side, and trade sections respectively.
    pub section_entry_counts: [u8; 3],
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
    /// Opaque data after extra objects and before ground items.
    pub extra_objects_trailer: MapExtraObjectsTrailer,
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
    pub item_type_id: u16,
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
    pub misc_item_id: u32, // 256
    pub map_coordinate_x: u32, // 260 coord-X
    pub map_coordinate_y: u32, // 264 coord-Y
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
    pub item_type_id: u8,   // inventory position 241
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
    pub item_type_id: u8,           // inventory position 269
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
    pub item_type_id: u16,       // 252
    pub position_index: u16,     // inventory position 254
    pub unknown_4: u8,           // inventory position 255
    pub unknown_5: u8,           // 6c 6c (108, 108) for the first row 256
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
    /// Per-save identity of this weapon inventory record, referenced by equipped slots.
    pub inventory_instance_id: u32, // 288
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

/// Journal data from a save file (42-byte header + 3 sections × 100 entries).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JournalData {
    /// Journal UI state and per-section entry counts.
    pub header: JournalHeader,
    /// Main quest entries (100 × 37 bytes)
    pub main: Vec<JournalEntry>,
    /// Side quest entries (100 × 37 bytes)
    pub side: Vec<JournalEntry>,
    /// Trading offer entries (100 × 37 bytes)
    pub trade: Vec<JournalEntry>,
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
    /// Number of NPCs that accompany the player on their adventures.
    pub party_members_count: u32,
    /// Party members (321 bytes each).
    pub party_members: Vec<PartyMember>,
}

/// Save-world header and the player runtime-state snapshot after map data.
///
/// Layout: `[map-section terminator: u32][8 × 4-byte header values]
/// [visited-map count][visited map IDs][player runtime state: 10,148 bytes]`.
///
/// The three record-size fields are 329, 349, and 200 in known saves; they
/// match the monster, NPC, and extra-object record sizes in the map section.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostMapsData {
    /// Terminator after the final map section. Known saves store zero.
    pub map_section_terminator: u32,
    /// Save-format version, observed as 1.45.
    pub game_version: f32,
    /// Unknown header value. Preserve it verbatim.
    pub unknown_header_value_1: u32,
    /// ID reference in AllMap.ini.
    pub all_map_ini_id: u32,
    /// ID reference in Ref/Map.ini.
    pub ref_map_ini_id: u32,
    /// Size of a MonsterRecord in the map section.
    pub monster_block_size: u32,
    /// Size of an NpcRecord in the map section.
    pub npc_block_size: u32,
    /// Unknown header value. Preserve it verbatim.
    pub unknown_header_value_2: u32,
    /// Size of an ExtraObjectRecord in the map section.
    pub extra_object_block_size: u32,
    /// Number of visited maps, which must match the preceding map section.
    pub number_of_visited_maps: u32,
    /// IDs of the visited maps.
    pub map_ids: Vec<u32>,
    /// Fixed-size serialized player runtime state. Its internal fields remain
    /// to be decoded, but its boundary is confirmed by every known fixture.
    pub player_runtime_state: Vec<u8>,
}

/// Unknown data block between events and journal sections.
///
/// Structure: fixed 12 bytes + counter-prefixed 24-byte records + fixed 56 bytes.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostEventsData {
    /// Unknown fixed block (12 bytes).
    pub block_a: Vec<u8>,
    /// Unknown records (counter × 24 bytes each).
    pub records: Vec<u8>,
    /// Unknown fixed block (56 bytes).
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
    /// Unknown 8 bytes
    pub unknown_before_stats_a: Vec<u8>,
    pub character_position_x: i16,
    pub character_position_y: i16,
    /// Header before character stats, including the currently selected spell.
    pub character_stats_header: CharacterStatsHeader,
    /// Parsed character stats (core, combat, skills, weapon skills).
    pub character_stats: CharacterStats,
    /// Unknown bytes after stats block (9 bytes).
    pub unknown_after_stats: Vec<u8>,
    /// Raw inventory data (5 item categories).
    pub inventory: InventoryData,
    /// Character identity (name, class, unknown blocks).
    pub character_identity: CharacterIdentity,
    /// Event scripts (2251 × 284 bytes).
    pub events: Vec<EventScript>,
    /// Unknown data between events and journal (3 sub-blocks).
    pub post_events: PostEventsData,
    /// Journal entries (main, side, trade — 100 entries each).
    pub journal: JournalData,
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
        let (
            unknown_before_stats,
            character_position_x,
            character_position_y,
            character_stats_header,
            character_stats,
            unknown_after_stats,
        ) = Self::parse_character_stats(&mut reader)?;

        // ── 6. Inventory (5 categories, each count-prefixed) ──
        let inventory = Self::parse_inventory_section(&mut reader)?;

        // ── 7. Character identity (unknown block + name + class + large unknown) ──
        let character_identity = Self::parse_character_identity(&mut reader)?;

        // ── 8. Events (2251 × 284 bytes) ──
        let events = Self::parse_events_section(&mut reader)?;

        // ── 9. Unknown data between events and journal ──
        let post_events = Self::parse_post_events_data(&mut reader)?;

        // ── 10. Journal (42-byte header + 3 sections × 100 × 37 bytes) ──
        let journal = Self::parse_journal_section(&mut reader)?;

        Ok(SaveFile {
            jump_addr_after_maps: jump_addr_after_maps as u32,
            maps,
            post_maps,
            sprite_paths,
            unknown_before_stats_a: unknown_before_stats,
            character_position_x,
            character_position_y,
            character_stats_header,
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
    ///   `[map_id: u32][monsters][npcs][sep: u32][extra_objects][trailer]
    ///    [draw_items_weapon][draw_items_heal][draw_items_edit]
    ///    [draw_items_misc][draw_items_event][end_sep: u32]`
    fn parse_maps_section<R: Read + Seek>(
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

            // ── 2.5. Extra-object trailer ──
            let tail_size = reader.read_u32::<LittleEndian>()?;
            let trailer_record_count = reader.read_u16::<LittleEndian>()? as usize;
            let mut trailer_records_data = vec![0u8; trailer_record_count * 24];
            reader.read_exact(&mut trailer_records_data)?;
            let records = trailer_records_data
                .chunks_exact(24)
                .map(ExtraObjectTrailerRecord::parse)
                .collect::<std::io::Result<Vec<_>>>()?;
            let automatic_placement_active = reader.read_u8()?;
            let automatic_placement_value = reader.read_u16::<LittleEndian>()?;
            let automatic_placement_global_item_index = reader.read_u16::<LittleEndian>()?;
            let extra_objects_trailer = MapExtraObjectsTrailer {
                tail_size,
                records,
                automatic_placement_active,
                automatic_placement_value,
                automatic_placement_global_item_index,
            };

            // ── 2.6–2.10. Ground items (5 types) ──
            let draw_items_weapon =
                Self::read_item_section(reader, 296, DrawItemWeaponItem::parse)?;
            let draw_items_heal = Self::read_item_section(reader, 264, DrawItemHealItem::parse)?;
            let draw_items_edit = Self::read_item_section(reader, 280, DrawItemEditItem::parse)?;
            let draw_items_misc = Self::read_item_section(reader, 268, DrawItemMiscItem::parse)?;
            let draw_items_event = Self::read_item_section(reader, 252, DrawItemEventItem::parse)?;

            let expected_tail_size = 17usize
                + extra_objects_trailer.records.len() * 24
                + draw_items_weapon.len() * 296
                + draw_items_heal.len() * 264
                + draw_items_edit.len() * 280
                + draw_items_misc.len() * 268
                + draw_items_event.len() * 252;
            if extra_objects_trailer.tail_size as usize != expected_tail_size {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "map extra-object trailer size is {}, expected {expected_tail_size}",
                        extra_objects_trailer.tail_size
                    ),
                ));
            }

            // ── 2.11. End-of-map separator (always 0) ──
            let _separator = reader.read_u32::<LittleEndian>()?;

            maps.push(MapSectionData {
                map_id,
                monsters,
                npcs,
                extra_objects,
                extra_objects_trailer,
                draw_items_weapon,
                draw_items_heal,
                draw_items_edit,
                draw_items_misc,
                draw_items_event,
            });
        }

        Ok(maps)
    }

    /// Parse the save-world header and player runtime-state snapshot.
    ///
    /// Layout: `[map-section terminator: u32][8 × 4-byte header values]
    /// [visited-map count][visited map IDs][player runtime state: 10,148 bytes]`.
    fn parse_post_maps_data<R: Read>(
        reader: &mut R,
        num_visited_maps: u32,
    ) -> std::io::Result<PostMapsData> {
        let map_section_terminator = reader.read_u32::<LittleEndian>()?;
        let game_version = reader.read_f32::<LittleEndian>()?;
        let unknown_header_value_1 = reader.read_u32::<LittleEndian>()?;
        let all_map_ini_id = reader.read_u32::<LittleEndian>()?;
        let ref_map_ini_id = reader.read_u32::<LittleEndian>()?;
        let monster_block_size = reader.read_u32::<LittleEndian>()?;
        let npc_block_size = reader.read_u32::<LittleEndian>()?;
        let unknown_header_value_2 = reader.read_u32::<LittleEndian>()?;
        let extra_object_block_size = reader.read_u32::<LittleEndian>()?;

        let number_of_visited_maps = reader.read_u32::<LittleEndian>()?;
        if number_of_visited_maps != num_visited_maps {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "post-maps visited-map count is {number_of_visited_maps}, expected {num_visited_maps}"
                ),
            ));
        }

        let mut map_ids = vec![0u32; number_of_visited_maps as usize];
        for map_id in &mut map_ids {
            *map_id = reader.read_u32::<LittleEndian>()?;
        }

        let mut player_runtime_state = vec![0u8; PLAYER_RUNTIME_STATE_SIZE];
        reader.read_exact(&mut player_runtime_state)?;

        Ok(PostMapsData {
            map_section_terminator,
            game_version,
            unknown_header_value_1,
            all_map_ini_id,
            ref_map_ini_id,
            monster_block_size,
            npc_block_size,
            unknown_header_value_2,
            extra_object_block_size,
            number_of_visited_maps,
            map_ids,
            player_runtime_state,
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
    ///   `[unknown_before_stats_a: 8B][position_x: i16][position_y: i16]
    ///    [character_stats_header: 28B][strength u16][agility u16][wisdom u16][constitution u16]
    ///    [morale u16][hp_cur u16][hp_max u16][mp_cur u16][mp_max u16]
    ///    [xp u32][level u16][gold u32][offense u16][defense u16]
    ///    [dodge u8][hit u8][magic_power u16][attack_mod u8]
    ///    [thievery u8][lockpick u8][haggle u8][perception u8][traps u8]
    ///    [sword_lv u8][sword_kills u16][axe_lv u8][axe_kills u16]
    ///    [archery_lv u8][archery_kills u16][polearm_lv u8][polearm_kills u16]
    ///    [magic_lv u8][magic_kills u16][holy_lv u8][holy_kills u16]
    ///    [dark_lv u8][dark_kills u16][unknown: 9B]`
    #[allow(clippy::type_complexity)]
    fn parse_character_stats<R: Read>(
        reader: &mut R,
    ) -> std::io::Result<(
        Vec<u8>,
        i16,
        i16,
        CharacterStatsHeader,
        CharacterStats,
        Vec<u8>,
    )> {
        // ── Leading data (8 bytes, purpose unknown) ──
        let mut unknown_before_stats = vec![0u8; 8];
        reader.read_exact(&mut unknown_before_stats)?;

        let character_position_x = reader.read_i16::<LittleEndian>()?;
        let character_position_y = reader.read_i16::<LittleEndian>()?;

        // ── Character stats header (28 bytes) ──
        let character_stats_header = CharacterStatsHeader {
            unknown_a: reader.read_u8()?,
            unknown_b: reader.read_u32::<LittleEndian>()?,
            selected_spell_id: reader.read_u32::<LittleEndian>()?,
            unknown_block: {
                let mut bytes = [0u8; 19];
                reader.read_exact(&mut bytes)?;
                bytes
            },
        };

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

        Ok((
            unknown_before_stats,
            character_position_x,
            character_position_y,
            character_stats_header,
            character_stats,
            unknown_after_stats,
        ))
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

    /// Parse the journal section (42-byte header + 3 × 100 × 37-byte entries).
    fn parse_journal_section<R: Read>(reader: &mut R) -> std::io::Result<JournalData> {
        const HEADER_SIZE: usize = 42;
        const ENTRY_SIZE: usize = 37;
        const ENTRIES_PER_SECTION: usize = 100;
        const SECTION_SIZE: usize = ENTRY_SIZE * ENTRIES_PER_SECTION; // 3700

        let mut header_data = [0u8; HEADER_SIZE];
        reader.read_exact(&mut header_data)?;
        let header = JournalHeader::parse(&header_data)?;

        let mut raw = vec![0u8; SECTION_SIZE];
        reader.read_exact(&mut raw)?;
        let main = Self::parse_journal_entries(&raw, ENTRIES_PER_SECTION)?;

        let mut raw = vec![0u8; SECTION_SIZE];
        reader.read_exact(&mut raw)?;
        let side = Self::parse_journal_entries(&raw, ENTRIES_PER_SECTION)?;

        let mut raw = vec![0u8; SECTION_SIZE];
        reader.read_exact(&mut raw)?;
        let trade = Self::parse_journal_entries(&raw, ENTRIES_PER_SECTION)?;

        Ok(JournalData {
            header,
            main,
            side,
            trade,
        })
    }

    /// Parse the events section (2251 × 284-byte event records).
    fn parse_events_section<R: Read>(reader: &mut R) -> std::io::Result<Vec<EventScript>> {
        const EVENT_COUNT: usize = 2251;
        const EVENT_SIZE: usize = 284;

        let mut events: Vec<EventScript> = Vec::with_capacity(EVENT_COUNT);
        for _ in 0..EVENT_COUNT {
            let mut buf = [0u8; EVENT_SIZE];
            reader.read_exact(&mut buf)?;
            events.push(EventScript::parse(&buf)?);
        }
        Ok(events)
    }

    /// Parse character identity (name, class, equipment, spells, party).
    ///
    /// Layout:
    ///   `[unknown_96B][name: 11B][class_id: u16][class_name: 11B]
    ///    [header: 11B][equipment: 108B][belt: 96B][inventory: 3780B][spells: 41B]
    ///    [party_count: u32][party_members]`
    fn parse_character_identity<R: Read + Seek>(
        reader: &mut R,
    ) -> std::io::Result<CharacterIdentity> {
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

        // ── 7.4. Character data blocks ──
        // TODO: Add writer method for these blocks

        // Header: u32 + u16 + u16 + u8 + u8 + u8 = 11 bytes
        let mut header_buf = [0u8; 11];
        reader.read_exact(&mut header_buf)?;
        let character_data_header = CharacterDataHeader::parse(&header_buf)?;

        // Equipped weapon items: 12 slots × 9 bytes = 108 bytes.
        let mut equipment_raw = vec![0u8; 12 * 9];
        reader.read_exact(&mut equipment_raw)?;
        let equipped_equipment: Vec<EquipmentSlot> = equipment_raw
            .chunks_exact(9)
            .map(|chunk| {
                let mut c = std::io::Cursor::new(chunk);
                EquipmentSlot {
                    panel_slot_marker: c.read_u8().unwrap(),
                    weapon_catalog_index: c.read_i32::<LittleEndian>().unwrap(),
                    weapon_inventory_instance_id: c.read_i32::<LittleEndian>().unwrap(),
                }
            })
            .collect();

        // Belt item placements: 6 cells × 16 bytes = 96 bytes.
        let mut belt_raw = vec![0u8; 6 * 16];
        reader.read_exact(&mut belt_raw)?;
        let belt_potions: Vec<BeltPotionSlot> = belt_raw
            .chunks_exact(16)
            .map(|chunk| {
                let mut c = std::io::Cursor::new(chunk);
                BeltPotionSlot {
                    item_category: c.read_i32::<LittleEndian>().unwrap(),
                    item_catalog_index: c.read_i32::<LittleEndian>().unwrap(),
                    icon_x: c.read_i32::<LittleEndian>().unwrap(),
                    icon_y: c.read_i32::<LittleEndian>().unwrap(),
                }
            })
            .collect();

        // Inventory placement: 3 pages × 7 columns × 9 cells × 20 bytes.
        let mut inventory_raw = vec![0u8; 189 * 20];
        reader.read_exact(&mut inventory_raw)?;
        let inventory_placement: Vec<InventoryPlacementEntry> = inventory_raw
            .chunks_exact(20)
            .map(|chunk| {
                let mut c = std::io::Cursor::new(chunk);
                InventoryPlacementEntry {
                    item_category: c.read_i32::<LittleEndian>().unwrap(),
                    item_catalog_index: c.read_i32::<LittleEndian>().unwrap(),
                    icon_x: c.read_i32::<LittleEndian>().unwrap(),
                    icon_y: c.read_i32::<LittleEndian>().unwrap(),
                    item_instance_index: c.read_i32::<LittleEndian>().unwrap(),
                }
            })
            .collect();

        // Learned spells: 41 bytes (one flag per spell)
        let mut spells_buf = vec![0u8; 41];
        reader.read_exact(&mut spells_buf)?;
        let learned_spells = LearnedSpells { spells: spells_buf };

        // ── 7.5. Party members ──
        let party_members_count = reader.read_u32::<LittleEndian>()?;
        let mut party_members = Vec::with_capacity(party_members_count as usize);
        for _i in 0..party_members_count {
            let mut party_member_data = vec![0u8; 321];
            reader.read_exact(&mut party_member_data)?;

            let entry = PartyMember::parse(&party_member_data)?;
            party_members.push(entry);
        }

        Ok(CharacterIdentity {
            unknown_block,
            player_name,
            player_class_id,
            player_class_name,
            character_data_header,
            equipped_equipment,
            belt_potions,
            inventory_placement,
            learned_spells,
            party_members_count,
            party_members,
        })
    }

    /// Parse the unknown section between events and journal.
    ///
    /// Layout: `[block_a: 12B][count: u32][count × 24B records][block_b: 56B]`
    fn parse_post_events_data<R: Read>(reader: &mut R) -> std::io::Result<PostEventsData> {
        let mut block_a = vec![0u8; 12];
        reader.read_exact(&mut block_a)?;

        let count = reader.read_u32::<LittleEndian>()? as usize;
        let mut records = vec![0u8; count * 24];
        reader.read_exact(&mut records)?;

        let mut block_b = vec![0u8; 56];
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

    // ── Write helpers ─────────────────────────────────────────────────────────

    /// Write the maps section to a writer (used internally to pre-compute size).
    fn write_maps_section<W: Write>(
        maps: &[MapSectionData],
        writer: &mut W,
    ) -> std::io::Result<()> {
        for map in maps {
            writer.write_u32::<LittleEndian>(map.map_id)?;

            // Monsters: u32 count + 329-byte records
            writer.write_u32::<LittleEndian>(map.monsters.len() as u32)?;
            for m in &map.monsters {
                m.write(writer)?;
            }

            // NPCs: u32 count + 349-byte records
            writer.write_u32::<LittleEndian>(map.npcs.len() as u32)?;
            for n in &map.npcs {
                n.write(writer)?;
            }

            // Separator (always 0)
            writer.write_u32::<LittleEndian>(0)?;

            // Extra objects: u32 count + 200-byte records
            writer.write_u32::<LittleEndian>(map.extra_objects.len() as u32)?;
            for e in &map.extra_objects {
                e.write(writer)?;
            }

            // Extra-object trailer: size, count, records, then controls.
            let record_count =
                u16::try_from(map.extra_objects_trailer.records.len()).map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "map extra-object trailer has more than u16::MAX records",
                    )
                })?;
            let expected_tail_size = 17usize
                + map.extra_objects_trailer.records.len() * 24
                + map.draw_items_weapon.len() * 296
                + map.draw_items_heal.len() * 264
                + map.draw_items_edit.len() * 280
                + map.draw_items_misc.len() * 268
                + map.draw_items_event.len() * 252;
            if map.extra_objects_trailer.tail_size as usize != expected_tail_size {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "map extra-object trailer size is {}, expected {expected_tail_size}",
                        map.extra_objects_trailer.tail_size
                    ),
                ));
            }
            writer.write_u32::<LittleEndian>(map.extra_objects_trailer.tail_size)?;
            writer.write_u16::<LittleEndian>(record_count)?;
            for record in &map.extra_objects_trailer.records {
                record.write(writer)?;
            }
            writer.write_u8(map.extra_objects_trailer.automatic_placement_active)?;
            writer
                .write_u16::<LittleEndian>(map.extra_objects_trailer.automatic_placement_value)?;
            writer.write_u16::<LittleEndian>(
                map.extra_objects_trailer
                    .automatic_placement_global_item_index,
            )?;

            // Ground items (5 types, each u16 count + fixed-size records)
            writer.write_u16::<LittleEndian>(map.draw_items_weapon.len() as u16)?;
            for d in &map.draw_items_weapon {
                d.write(writer)?;
            }
            writer.write_u16::<LittleEndian>(map.draw_items_heal.len() as u16)?;
            for d in &map.draw_items_heal {
                d.write(writer)?;
            }
            writer.write_u16::<LittleEndian>(map.draw_items_edit.len() as u16)?;
            for d in &map.draw_items_edit {
                d.write(writer)?;
            }
            writer.write_u16::<LittleEndian>(map.draw_items_misc.len() as u16)?;
            for d in &map.draw_items_misc {
                d.write(writer)?;
            }
            writer.write_u16::<LittleEndian>(map.draw_items_event.len() as u16)?;
            for d in &map.draw_items_event {
                d.write(writer)?;
            }

            // End-of-map separator (always 0)
            writer.write_u32::<LittleEndian>(0)?;
        }
        Ok(())
    }

    /// Write post-maps data block.
    fn write_post_maps_data<W: Write>(data: &PostMapsData, writer: &mut W) -> std::io::Result<()> {
        writer.write_u32::<LittleEndian>(data.map_section_terminator)?;
        writer.write_f32::<LittleEndian>(data.game_version)?;
        writer.write_u32::<LittleEndian>(data.unknown_header_value_1)?;
        writer.write_u32::<LittleEndian>(data.all_map_ini_id)?;
        writer.write_u32::<LittleEndian>(data.ref_map_ini_id)?;
        writer.write_u32::<LittleEndian>(data.monster_block_size)?;
        writer.write_u32::<LittleEndian>(data.npc_block_size)?;
        writer.write_u32::<LittleEndian>(data.unknown_header_value_2)?;
        writer.write_u32::<LittleEndian>(data.extra_object_block_size)?;
        let map_id_count = u32::try_from(data.map_ids.len()).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "post-maps has more than u32::MAX map IDs",
            )
        })?;
        if data.number_of_visited_maps != map_id_count {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "post-maps visited-map count is {}, but {} map IDs were provided",
                    data.number_of_visited_maps,
                    data.map_ids.len()
                ),
            ));
        }
        if data.player_runtime_state.len() != PLAYER_RUNTIME_STATE_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "player runtime state is {} bytes, expected {PLAYER_RUNTIME_STATE_SIZE}",
                    data.player_runtime_state.len()
                ),
            ));
        }
        writer.write_u32::<LittleEndian>(data.number_of_visited_maps)?;
        for id in &data.map_ids {
            writer.write_u32::<LittleEndian>(*id)?;
        }
        writer.write_all(&data.player_runtime_state)?;
        Ok(())
    }

    /// Write sprite paths (always 4 × 60-byte fixed buffers).
    fn write_sprite_paths<W: Write>(paths: &[String], writer: &mut W) -> std::io::Result<()> {
        for i in 0..4 {
            let s = paths.get(i).map(|s| s.as_str()).unwrap_or("");
            let mut buf = [0u8; 60];
            let (cow, _, _) = encoding_rs::WINDOWS_1250.encode(s);
            let len = std::cmp::min(cow.len(), 60);
            buf[..len].copy_from_slice(&cow[..len]);
            writer.write_all(&buf)?;
        }
        Ok(())
    }

    /// Write position data, character stats, and trailing unknown bytes.
    fn write_character_stats<W: Write>(
        unknown_before_a: &[u8],
        character_position_x: i16,
        character_position_y: i16,
        character_stats_header: &CharacterStatsHeader,
        stats: &CharacterStats,
        unknown_after: &[u8],
        writer: &mut W,
    ) -> std::io::Result<()> {
        writer.write_all(unknown_before_a)?;
        writer.write_i16::<LittleEndian>(character_position_x)?;
        writer.write_i16::<LittleEndian>(character_position_y)?;
        writer.write_u8(character_stats_header.unknown_a)?;
        writer.write_u32::<LittleEndian>(character_stats_header.unknown_b)?;
        writer.write_u32::<LittleEndian>(character_stats_header.selected_spell_id)?;
        writer.write_all(&character_stats_header.unknown_block)?;
        writer.write_u16::<LittleEndian>(stats.strength)?;
        writer.write_u16::<LittleEndian>(stats.agility)?;
        writer.write_u16::<LittleEndian>(stats.wisdom)?;
        writer.write_u16::<LittleEndian>(stats.constitution)?;
        writer.write_u16::<LittleEndian>(stats.morale)?;
        writer.write_u16::<LittleEndian>(stats.hp_current)?;
        writer.write_u16::<LittleEndian>(stats.hp_maximum)?;
        writer.write_u16::<LittleEndian>(stats.mp_current)?;
        writer.write_u16::<LittleEndian>(stats.mp_maximum)?;
        writer.write_u32::<LittleEndian>(stats.experience)?;
        writer.write_u16::<LittleEndian>(stats.level)?;
        writer.write_u32::<LittleEndian>(stats.gold)?;
        writer.write_u16::<LittleEndian>(stats.offense)?;
        writer.write_u16::<LittleEndian>(stats.defense)?;
        writer.write_u8(stats.dodge_rate)?;
        writer.write_u8(stats.hit_rate)?;
        writer.write_u16::<LittleEndian>(stats.magic_power)?;
        writer.write_u8(stats.attack_modifier)?;
        writer.write_u8(stats.pickpocketing)?;
        writer.write_u8(stats.lockpicking)?;
        writer.write_u8(stats.haggling)?;
        writer.write_u8(stats.perception)?;
        writer.write_u8(stats.traps)?;
        writer.write_u8(stats.swords_level)?;
        writer.write_u16::<LittleEndian>(stats.swords_kills)?;
        writer.write_u8(stats.axes_level)?;
        writer.write_u16::<LittleEndian>(stats.axes_kills)?;
        writer.write_u8(stats.archery_level)?;
        writer.write_u16::<LittleEndian>(stats.archery_kills)?;
        writer.write_u8(stats.polearm_level)?;
        writer.write_u16::<LittleEndian>(stats.polearm_kills)?;
        writer.write_u8(stats.magic_level)?;
        writer.write_u16::<LittleEndian>(stats.magic_kills)?;
        writer.write_u8(stats.holy_magic_level)?;
        writer.write_u16::<LittleEndian>(stats.holy_magic_kills)?;
        writer.write_u8(stats.dark_magic_level)?;
        writer.write_u16::<LittleEndian>(stats.dark_magic_kills)?;
        writer.write_all(unknown_after)?;
        Ok(())
    }

    /// Write inventory (5 categories, each u16 count + fixed-size records).
    fn write_inventory<W: Write>(inv: &InventoryData, writer: &mut W) -> std::io::Result<()> {
        writer.write_u16::<LittleEndian>(inv.event_items.len() as u16)?;
        for item in &inv.event_items {
            item.write(writer)?;
        }
        writer.write_u16::<LittleEndian>(inv.misc_items.len() as u16)?;
        for item in &inv.misc_items {
            item.write(writer)?;
        }
        writer.write_u16::<LittleEndian>(inv.edit_items.len() as u16)?;
        for item in &inv.edit_items {
            item.write(writer)?;
        }
        writer.write_u16::<LittleEndian>(inv.weapon_items.len() as u16)?;
        for item in &inv.weapon_items {
            item.write(writer)?;
        }
        writer.write_u16::<LittleEndian>(inv.heal_items.len() as u16)?;
        for item in &inv.heal_items {
            item.write(writer)?;
        }
        Ok(())
    }

    /// Write character identity (96B unknown + 11B name + u16 class + 11B class name + character data blocks + party).
    fn write_character_identity<W: Write>(
        identity: &CharacterIdentity,
        writer: &mut W,
    ) -> std::io::Result<()> {
        writer.write_all(&identity.unknown_block)?;

        // Player name: 11-byte WINDOWS-1250 fixed buffer
        let mut name_buf = [0u8; 11];
        let (cow, _, _) = encoding_rs::WINDOWS_1250.encode(&identity.player_name);
        let len = std::cmp::min(cow.len(), 11);
        name_buf[..len].copy_from_slice(&cow[..len]);
        writer.write_all(&name_buf)?;

        writer.write_u16::<LittleEndian>(identity.player_class_id)?;

        // Class name: 11-byte WINDOWS-1250 fixed buffer
        let mut class_buf = [0u8; 11];
        let (cow, _, _) = encoding_rs::WINDOWS_1250.encode(&identity.player_class_name);
        let len = std::cmp::min(cow.len(), 11);
        class_buf[..len].copy_from_slice(&cow[..len]);
        writer.write_all(&class_buf)?;

        // ── Character data blocks ──

        // Header: u32 + u16 + u16 + u8 + u8 + u8 = 11 bytes
        identity.character_data_header.write(writer)?;

        // Equipped weapon items: 12 slots × 9 bytes = 108 bytes.
        for slot in &identity.equipped_equipment {
            writer.write_u8(slot.panel_slot_marker)?;
            writer.write_i32::<LittleEndian>(slot.weapon_catalog_index)?;
            writer.write_i32::<LittleEndian>(slot.weapon_inventory_instance_id)?;
        }

        // Belt item placements: 6 cells × 16 bytes = 96 bytes.
        for slot in &identity.belt_potions {
            writer.write_i32::<LittleEndian>(slot.item_category)?;
            writer.write_i32::<LittleEndian>(slot.item_catalog_index)?;
            writer.write_i32::<LittleEndian>(slot.icon_x)?;
            writer.write_i32::<LittleEndian>(slot.icon_y)?;
        }

        // Inventory placement: 3 pages × 7 columns × 9 cells × 20 bytes.
        for entry in &identity.inventory_placement {
            writer.write_i32::<LittleEndian>(entry.item_category)?;
            writer.write_i32::<LittleEndian>(entry.item_catalog_index)?;
            writer.write_i32::<LittleEndian>(entry.icon_x)?;
            writer.write_i32::<LittleEndian>(entry.icon_y)?;
            writer.write_i32::<LittleEndian>(entry.item_instance_index)?;
        }

        // Learned spells: 41 bytes
        writer.write_all(&identity.learned_spells.spells)?;

        // ── Party members ──
        writer.write_u32::<LittleEndian>(identity.party_members_count)?;
        for member in &identity.party_members {
            member.write(writer)?;
        }
        Ok(())
    }

    /// Write event scripts in order.
    fn write_events<W: Write>(events: &[EventScript], writer: &mut W) -> std::io::Result<()> {
        for event in events {
            event.write(writer)?;
        }
        Ok(())
    }

    /// Write post-events unknown data block.
    fn write_post_events<W: Write>(data: &PostEventsData, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&data.block_a)?;
        let count = data.records.len() / 24;
        writer.write_u32::<LittleEndian>(count as u32)?;
        writer.write_all(&data.records)?;
        writer.write_all(&data.block_b)?;
        Ok(())
    }

    /// Write journal (3 sections × entries in order).
    fn write_journal<W: Write>(journal: &JournalData, writer: &mut W) -> std::io::Result<()> {
        journal.header.write(writer)?;
        for entry in &journal.main {
            entry.write(writer)?;
        }
        for entry in &journal.side {
            entry.write(writer)?;
        }
        for entry in &journal.trade {
            entry.write(writer)?;
        }
        Ok(())
    }
}

impl Extractor for SaveFile {
    fn parse<R: Read + Seek>(reader: &mut R, _len: u64) -> std::io::Result<Vec<Self>> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        let save = SaveFile::parse(&data)?;
        Ok(vec![save])
    }

    fn to_writer<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
        if records.len() != 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "SaveFile can only serialize one record at a time",
            ));
        }
        let save = &records[0];

        // Pre-compute maps section to determine jump_addr_after_maps
        let mut maps_buf = Vec::new();
        Self::write_maps_section(&save.maps, &mut maps_buf)?;
        let jump_addr = 8u32 + maps_buf.len() as u32;

        // 1. Header: jump address after all maps data
        writer.write_u32::<LittleEndian>(jump_addr)?;

        // 2. Map count + maps data
        writer.write_u32::<LittleEndian>(save.maps.len() as u32)?;
        writer.write_all(&maps_buf)?;

        // 3. Post-maps data
        Self::write_post_maps_data(&save.post_maps, writer)?;

        // 4. Sprite paths (always 4 × 60-byte fixed buffers)
        Self::write_sprite_paths(&save.sprite_paths, writer)?;

        // 5. Belt data + character stats + trailing bytes
        Self::write_character_stats(
            &save.unknown_before_stats_a,
            save.character_position_x,
            save.character_position_y,
            &save.character_stats_header,
            &save.character_stats,
            &save.unknown_after_stats,
            writer,
        )?;

        // 6. Inventory (5 categories)
        Self::write_inventory(&save.inventory, writer)?;

        // 7. Character identity
        Self::write_character_identity(&save.character_identity, writer)?;

        // 8. Events
        Self::write_events(&save.events, writer)?;

        // 9. Post-events data
        Self::write_post_events(&save.post_events, writer)?;

        // 10. Journal (3 sections)
        Self::write_journal(&save.journal, writer)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monster_record_preserves_verified_329_byte_layout() {
        let mut bytes = [0u8; 329];
        bytes[68..72].copy_from_slice(&0x0102_0304u32.to_le_bytes());
        bytes[72] = 5;
        bytes[73] = 1;
        bytes[74] = 0xfe;
        bytes[75] = 0xfc;
        bytes[76] = 7;
        bytes[77..81].copy_from_slice(&123u32.to_le_bytes());
        bytes[81..85].copy_from_slice(&456u32.to_le_bytes());
        bytes[93] = 1;
        bytes[121..123].copy_from_slice(&10u16.to_le_bytes());
        bytes[125] = 1;
        bytes[173..177].copy_from_slice(&1u32.to_le_bytes());
        bytes[177..181].copy_from_slice(&(-1i32).to_le_bytes());
        bytes[181..185].copy_from_slice(&12_000u32.to_le_bytes());
        bytes[193..197].copy_from_slice(&99u32.to_le_bytes());
        bytes[245] = 9;
        bytes[246] = 5;
        bytes[247] = 1;
        bytes[248..252].copy_from_slice(&123u32.to_le_bytes());
        bytes[252..256].copy_from_slice(&456u32.to_le_bytes());
        bytes[256] = 1;
        bytes[257..329].fill(0xaa);

        let record = MonsterRecord::parse(&bytes).unwrap();

        assert_eq!(record.magic_level, 0x0102_0304);
        assert_eq!(record.patrol_countdown, 5);
        assert_eq!(record.target_position_x, 123);
        assert_eq!(record.target_position_y, 456);
        assert_eq!(record.awake_flag, 1);
        assert_eq!(record.spawn_group_id, 10);
        assert_eq!(record.dead_or_removed_flag, 1);
        assert_eq!(record.force_ai_update, 1);
        assert_eq!(record.drop_all_loot, u32::MAX);
        assert_eq!(record.respawn_timer, 12_000);
        assert_eq!(record.special_attack, 99);
        assert_eq!(record.path_buffer_position_x, 123);
        assert_eq!(record.path_buffer_position_y, 456);
        assert_eq!(record.nested_summon_flag, 1);
        assert_eq!(record.nested_summon_record, vec![0xaa; 72]);

        let mut serialized = Vec::new();
        record.write(&mut serialized).unwrap();
        assert_eq!(serialized, bytes);
    }

    #[test]
    fn test_npc_record_preserves_verified_349_byte_layout() {
        let mut bytes = [0u8; 349];
        bytes[192] = 9;
        bytes[193..197].copy_from_slice(&42u32.to_le_bytes());
        bytes[197] = 2;
        bytes[198..202].copy_from_slice(&1u32.to_le_bytes());
        bytes[202..206].copy_from_slice(&100u32.to_le_bytes());
        bytes[206..210].copy_from_slice(&200u32.to_le_bytes());
        bytes[210..214].copy_from_slice(&30u32.to_le_bytes());
        bytes[214..218].copy_from_slice(&7u32.to_le_bytes());
        bytes[294..298].copy_from_slice(&10u32.to_le_bytes());
        bytes[298..302].copy_from_slice(&20u32.to_le_bytes());
        bytes[302..306].copy_from_slice(&30u32.to_le_bytes());
        bytes[306..310].copy_from_slice(&40u32.to_le_bytes());
        bytes[310] = 1;
        bytes[311..315].copy_from_slice(&0x0010_0401u32.to_le_bytes());
        bytes[315] = 11;
        bytes[316..320].copy_from_slice(&81u32.to_le_bytes());
        bytes[320] = 6;
        bytes[321..325].copy_from_slice(&1u32.to_le_bytes());
        bytes[329..333].copy_from_slice(&300u32.to_le_bytes());
        bytes[333..337].copy_from_slice(&400u32.to_le_bytes());
        bytes[341..345].copy_from_slice(&1u32.to_le_bytes());
        bytes[345..349].copy_from_slice(&99u32.to_le_bytes());

        let record = NpcRecord::parse(&bytes).unwrap();

        assert_eq!(record.npc_ref_party_member_slot, 9);
        assert_eq!(record.npc_ref_show_on_event_id, 42);
        assert_eq!(record.npc_ref_movement_mode, 2);
        assert_eq!(record.waypoint1_wait_time, 30);
        assert_eq!(record.waypoint1_facing_direction, 7);
        assert_eq!(record.activation_rect_x1, 10);
        assert_eq!(record.activation_rect_y2, 40);
        assert_eq!(record.npc_ref_interaction_mode, 1);
        assert_eq!(record.npc_ref_interaction_result, 0x0010_0401);
        assert_eq!(record.npc_ref_interaction_range, 11);
        assert_eq!(record.npc_ref_dialog_id, 81);
        assert_eq!(record.dialogue_face_sprite_id, 6);
        assert_eq!(record.move_mode, 1);
        assert_eq!(record.runtime_target_position_x, 300);
        assert_eq!(record.runtime_target_position_y, 400);
        assert_eq!(record.freeze_flag, 1);
        assert_eq!(record.freeze_counter, 99);

        let mut serialized = Vec::new();
        record.write(&mut serialized).unwrap();
        assert_eq!(serialized, bytes);
    }

    #[test]
    fn test_write_post_maps_data_matches_recognized_header_layout() {
        let post_maps = PostMapsData {
            map_section_terminator: 0,
            game_version: 1.5,
            unknown_header_value_1: 1,
            all_map_ini_id: 2,
            ref_map_ini_id: 3,
            monster_block_size: 5,
            npc_block_size: 6,
            unknown_header_value_2: 7,
            extra_object_block_size: 8,
            number_of_visited_maps: 2,
            map_ids: vec![9, 10],
            player_runtime_state: vec![11; PLAYER_RUNTIME_STATE_SIZE],
        };
        let mut bytes = Vec::new();

        SaveFile::write_post_maps_data(&post_maps, &mut bytes).unwrap();

        let mut reader = std::io::Cursor::new(bytes);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 0);
        assert_eq!(reader.read_f32::<LittleEndian>().unwrap(), 1.5);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 1);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 2);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 3);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 5);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 6);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 7);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 8);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 2);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 9);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 10);
        let mut player_runtime_state = vec![0u8; PLAYER_RUNTIME_STATE_SIZE];
        reader.read_exact(&mut player_runtime_state).unwrap();
        assert_eq!(player_runtime_state, vec![11; PLAYER_RUNTIME_STATE_SIZE]);
    }

    #[test]
    fn test_write_character_stats_preserves_position_and_surrounding_blocks() {
        let mut bytes = Vec::new();
        let header = CharacterStatsHeader {
            unknown_a: 2,
            unknown_b: 3,
            selected_spell_id: 4,
            unknown_block: [5; 19],
        };

        SaveFile::write_character_stats(
            &[1; 8],
            -123,
            456,
            &header,
            &CharacterStats::default(),
            &[3; 9],
            &mut bytes,
        )
        .unwrap();

        assert_eq!(&bytes[..8], &[1; 8]);
        let mut reader = std::io::Cursor::new(&bytes[8..12]);
        assert_eq!(reader.read_i16::<LittleEndian>().unwrap(), -123);
        assert_eq!(reader.read_i16::<LittleEndian>().unwrap(), 456);
        assert_eq!(bytes[12], 2);
        assert_eq!(u32::from_le_bytes(bytes[13..17].try_into().unwrap()), 3);
        assert_eq!(u32::from_le_bytes(bytes[17..21].try_into().unwrap()), 4);
        assert_eq!(&bytes[21..40], &[5; 19]);
        assert_eq!(&bytes[bytes.len() - 9..], &[3; 9]);

        let (_, position_x, position_y, parsed_header, _, _) =
            SaveFile::parse_character_stats(&mut std::io::Cursor::new(bytes)).unwrap();
        assert_eq!(position_x, -123);
        assert_eq!(position_y, 456);
        assert_eq!(parsed_header.selected_spell_id, header.selected_spell_id);
        assert_eq!(parsed_header.unknown_block, header.unknown_block);
    }

    #[test]
    fn test_maps_section_round_trips_extra_object_trailer() {
        assert_eq!(ExtraObjectTrailerRecord::record_size(), 24);
        let map = MapSectionData {
            map_id: 42,
            extra_objects_trailer: MapExtraObjectsTrailer {
                tail_size: 65,
                records: vec![
                    ExtraObjectTrailerRecord {
                        item_category: 4,
                        reserved_1: 0x80,
                        global_item_index: 780,
                        placement_attempt_count: 0,
                        placement_attempt_limit: 3,
                        unknown_6_7: [0xAA, 0xBB],
                        category_item_index: 7,
                        source_entity_id: 631,
                        unknown_14_15: [0xCC, 0xDD],
                        map_x: -1120,
                        map_y: -80,
                    },
                    ExtraObjectTrailerRecord {
                        item_category: 1,
                        ..Default::default()
                    },
                ],
                automatic_placement_active: 0,
                automatic_placement_value: 773,
                automatic_placement_global_item_index: 780,
            },
            ..Default::default()
        };
        let mut bytes = Vec::new();

        SaveFile::write_maps_section(&[map], &mut bytes).unwrap();
        let parsed = SaveFile::parse_maps_section(&mut std::io::Cursor::new(bytes), 1).unwrap();

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].extra_objects_trailer.tail_size, 65);
        assert_eq!(parsed[0].extra_objects_trailer.records.len(), 2);
        assert_eq!(parsed[0].extra_objects_trailer.records[0].item_category, 4);
        assert_eq!(
            parsed[0].extra_objects_trailer.records[0].global_item_index,
            780
        );
        assert_eq!(
            parsed[0].extra_objects_trailer.records[0].placement_attempt_limit,
            3
        );
        assert_eq!(parsed[0].extra_objects_trailer.records[0].map_x, -1120);
        assert_eq!(
            parsed[0].extra_objects_trailer.automatic_placement_value,
            773
        );
        assert_eq!(
            parsed[0]
                .extra_objects_trailer
                .automatic_placement_global_item_index,
            780
        );
    }
}
