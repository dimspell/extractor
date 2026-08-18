use dispel_macros::BinaryRecord;
use serde::{Deserialize, Serialize};

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
pub struct DrawItemEditItem {
    // 280
    #[binary_record(string(encoding = "WINDOWS-1250", size = 30))]
    pub name: String, // 30
    #[binary_record(string(encoding = "WINDOWS-1250", size = 202))]
    pub description: String, // 232
    pub base_price: u32,              // 236
    pub edit_item_id: u32,            // 240
    pub health_points: i16,           // 242
    pub mana_points: i16,             // 244
    pub strength: i16,                // 246
    pub agility: i16,                 // 248
    pub wisdom: i16,                  // 250
    pub constitution: i16,            // 252
    pub to_dodge: i16,                // 254
    pub to_hit: i16,                  // 256
    pub offense: i16,                 // 258
    pub defense: i16,                 // 260
    pub magical_power: i16,           // 262
    pub modification_resistance: i16, // 264
    /// Reserved byte; observed as zero and not used by the game.
    pub reserved_byte: u8, // 265
    pub modifies_item: u8,            // 266
    pub additional_effect: i16,       // 268
    pub map_coordinate_x: u32,        // 272
    pub map_coordinate_y: u32,        // 276
    pub unknown_4: u32,               // 280
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
