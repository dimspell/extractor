# Save File Documentation

## File Information

- **Location**: `Dispel/Save/` directory (relative to game installation directory)
- **Encoding**: WINDOWS-1250 (for text fields)
- **Format**: Binary (little-endian)
- **Extension**: `.sav` (or similar save file extension)

The save file is a binary serialization of the player's game state, including character stats, inventory, party members, journal entries, and map-specific data.

## Structure

The save file is parsed sequentially from start to end. The top-level structure is defined by the `SaveFile` struct in `src/references/save_file/mod.rs`.

### Top-Level Layout

| Offset | Field | Type | Description |
|--------|-------|------|-------------|
| 0 | `game_tmp_blob_size` | `u32` | Jump address after all map data (first 4 bytes of the file) |
| 4 | `number_of_visited_maps` | `u32` | Count of map sections that follow |
| 8 | `maps` | `Vec<MapSectionData>` | Per-map world state (variable length) |
| — | `post_maps` | `PostMapsData` | Save-world header and map viewport state |
| — | `map_viewport_state` | `MapViewportState` | Fixed-size isometric map viewport state (10,148 bytes) |
| — | `sprite_paths` | `Vec<String>` | Character sprite paths (4 × 60 bytes) |
| — | `character` | `CharacterData` | Character stats, position, and runtime state (112 bytes) |
| — | `inventory` | `InventoryData` | Raw inventory data (5 item categories) |
| — | `character_identity` | `CharacterIdentity` | Character name, class, and spell-bar state |
| — | `inventory_slots` | `InventorySlots` | Equipment, belt, and inventory placement state |
| — | `learned_spells` | `LearnedSpells` | One flag per learned spell (41 bytes) |
| — | `party_members_count` | `u32` | Serialized party-member count |
| — | `party_members` | `Vec<PartyMember>` | Recruited party members (variable length) |
| — | `events` | `Vec<EventRecord>` | Event scripts (2,251 × 284 bytes) |
| — | `post_events` | `PostEventsData` | Character walk-event log between events and journal |
| — | `journal` | `JournalData` | Journal entries (42-byte header + 3 × 100 entries) |

## Section Details

### Header

The first 8 bytes contain:
1. `game_tmp_blob_size` (4 bytes, `u32`): A jump address pointing to the byte offset after all map data. The reader uses this to seek past the map sections if needed.
2. `number_of_visited_maps` (4 bytes, `u32`): The number of `MapSectionData` records that follow.

### Map Sections (`maps`)

Each visited map is serialized as a `MapSectionData` record containing:

| Field | Type | Description |
|-------|------|-------------|
| `map_id` | `u32` | Map index/ID referenced in AllMap.ini |
| `monsters` | `Vec<MonsterRecord>` | Monsters present on this map (329 bytes each) |
| `npcs` | `Vec<NpcRecord>` | NPCs present on this map (349 bytes each) |
| `extra_objects` | `Vec<ExtraObjectRecord>` | Extra objects like chests, doors, triggers (200 bytes each) |
| `extra_objects_trailer` | `MapExtraObjectsTrailer` | Ground-item manager data |
| `draw_items_weapon` | `Vec<DrawItemWeaponItem>` | Ground items — Weapon type (296 bytes each) |
| `draw_items_heal` | `Vec<DrawItemHealItem>` | Ground items — Heal type (264 bytes each) |
| `draw_items_edit` | `Vec<DrawItemEditItem>` | Ground items — Edit type (280 bytes each) |
| `draw_items_misc` | `Vec<DrawItemMiscItem>` | Ground items — Misc type (268 bytes each) |
| `draw_items_event` | `Vec<DrawItemEventItem>` | Ground items — Event type (252 bytes each) |

Each collection is prefixed with a count (`u32` for monsters/NPCs/extra objects, `u16` for ground items).

Each ground-item record ends with a one-based `u16` ground-object ID and two preserved padding bytes.
The item category determines the ID range used by the game.

An NPC record stores an active path-step direction at byte 161 and its animation frame at byte 162.
The direction values are `0=(0,+32)`, `1=(-32,+16)`, `2=(-64,0)`, `3=(-32,-16)`,
`4=(0,-32)`, `5=(+32,-16)`, `6=(+64,0)`, `7=(+32,+16)`, and `255=inactive`.
The pairs are screen-space offsets from the destination cell.

Monster runtime data includes the current combat target, timed status-effect
type and duration, movement-animation frame and offsets, path-buffer length and
index, and temporary visual effects. Temporary visuals have separate active,
frame, and duration fields for status, ground, special-attack, guard, blood,
and timed-overlay effects. Blood-effect directions use values `0..=7`.

The monster status-effect type uses `0=none`; observed active values are `1`,
`2`, `3`, `4`, `7`, and `12`. Its parameter is `-1` when unused. Type `3`
stores a monster identifier in the parameter field.

NPC runtime flags use these values:

- `world_active`: `0=inactive`, `1=active`.
- `transient_spawn`: `0=map-defined`, `1=dynamically created`.
- `removed_from_world`: `0=normal`, `1=persistently removed`.
- `event_npc_origin`: `0=regular map NPC`, `1=event-created NPC`.
- `player_interaction_latched`: `0=not interacting`, `1=interaction started`.
- `start_dialogue_on_arrival`: `0=normal arrival`, `1=start arrival dialogue`.

The arrival dialogue ID follows the arrival-action flag. Two intervening runtime
words are reserved, initialized to zero, and preserved without interpretation.

An extra-object record begins with its selected render state, render variant,
and current sprite frame. It ends with the activation effect ID, one reserved
byte, two padding bytes, the overlay flag, the map-active flag, and the pending
interaction latch. Boolean controls use `0=disabled` and non-zero `enabled`;
the pending latch uses `0=no request` and `1=activation requested`.

### Post-Maps Data (`post_maps`)

Contains save-world header values and the list of visited map IDs:

| Field | Type | Description |
|-------|------|-------------|
| `map_section_terminator` | `u32` | Terminator after the final map section (known saves store zero) |
| `game_version` | `f32` | Save-format version (observed as 1.45) |
| `all_map_ini_id` | `u32` | `AllMap.ini.id` for the loaded map's files, geometry, name, dialogue, and lighting |
| `ref_map_ini_id` | `u32` | `Ref/Map.ini.id` for the entrance configuration, including spawn coordinates and placement files |
| `reserved_header_word` | `u32` | Reserved header word (observed as zero) |
| `monster_block_size` | `u32` | Size of a MonsterRecord (329 in known saves) |
| `npc_block_size` | `u32` | Size of an NpcRecord (349 in known saves) |
| `unused_map_object_block_size` | `u32` | Record size for an unused map-object section (observed as zero) |
| `extra_object_block_size` | `u32` | Size of an ExtraObjectRecord (200 in known saves) |
| `number_of_visited_maps` | `u32` | Number of visited maps (must match the map section count) |
| `map_ids` | `Vec<u32>` | IDs of the visited maps |

The two map identifiers address different tables. `all_map_ini_id` selects the
map itself. `ref_map_ini_id` selects the route or entrance used to initialize
that map. Multiple `Ref/Map.ini` records can target the same `AllMap.ini` map
while providing different spawn coordinates.

### Map Viewport State (`map_viewport_state`)

A fixed-size (10,148 bytes) serialized state of the game's isometric map viewport:

| Field | Type | Description |
|-------|------|-------------|
| `viewport_clip_rect` | `MapViewportRect` | Fixed screen rectangle in which the map is drawn |
| `map_projection_rect` | `MapViewportRect` | Projected map rectangle translated while the camera scrolls |
| `camera_boundary_tiles` | `[MapTileReference; 8]` | Tile coordinates and row-major indices used to constrain camera movement |
| `cells` | `Vec<MapViewportCell>` | Cached screen-to-map lookup cells (500 entries, 20 bytes each) |
| `scroll_direction` | `i32` | Smooth-scroll direction: `-1`=idle, `0`=up, `1`=up-right, `2`=right, `3`=down-right, `4`=down, `5`=down-left, `6`=left, `7`=up-left |
| `smooth_scroll_offset_x` | `u32` | Accumulated horizontal sub-tile scroll offset |
| `smooth_scroll_offset_y` | `u32` | Accumulated vertical sub-tile scroll offset |
| `scroll_animation_frame` | `u32` | Current smooth-scroll animation frame |
| `scroll_animation_frame_count` | `u32` | Total frames in the active smooth-scroll animation |

Each `MapViewportCell` contains:
- `screen_x` (`u32`): Screen X coordinate
- `screen_y` (`u32`): Screen Y coordinate
- `map_x` (`u32`): Map tile X coordinate
- `map_y` (`u32`): Map tile Y coordinate
- `map_tile_index` (`u32`): Computed as `map_y * map_width + map_x`

### Sprite Paths

Four fixed-size (60-byte) null-terminated WINDOWS-1250 strings representing character sprite paths.

### Character Data (`character`)

112 bytes containing character position, stats, and runtime state:

| Field | Type | Description |
|-------|------|-------------|
| `script_event_active` | `u32` | Whether a script/event is running |
| `armor_display_mode` | `u32` | Armor rendering mode by gender |
| `character_position_x` | `i16` | Character X position |
| `character_position_y` | `i16` | Character Y position |
| `is_moving` | `u8` | Whether the character is moving |
| `tile_value_under_player` | `u8` | Ground tile ID under the player |
| `busy_event_flag` | `u8` | Non-combat interaction flag |
| `target_tile_value` | `u8` | Target tile value during movement |
| `sub_action_state` | `u8` | Movement sub-state (0=idle, 1=start, 2=walking, 3=action) |
| `selected_spell_id` | `u8` | Currently selected spell |
| `attack_anim_frame` | `i16` | Attack animation frame |
| `attack_action_state` | `u8` | Attack/cast state |
| `action_mode` | `u8` | Transient action mode selector |
| `anim_frame_delay_counter` | `u8` | Animation frame delay counter |
| `anim_frame_delay_threshold` | `u8` | Animation frame delay threshold |
| `movement_state` | `u8` | Movement mode (0=idle, 1=running, 2=walking, 3=special) |
| `animation_frame` | `u8` | Current animation frame index |
| `level_up_pending` | `u8` | Level-up pending flag |
| `elapsed_frame_counter` | `i16` | Held action frame counter |
| `hit_buildup_counter` | `i16` | Rapid hit counter |
| `selected_action_index` | `i16` | Selected action index |
| `clickable_item_count` | `u8` | Count of clickable map entities |
| `active_status_effect` | `u8` | Active status effect |
| `pending_status_effect` | `u8` | Pending status effect |
| `poison_tick_interval` | `i16` | Poison damage tick interval |
| `poison_tick_counter` | `i16` | Poison damage tick counter |
| `strength` | `u16` | Strength stat |
| `agility` | `u16` | Agility stat |
| `wisdom` | `u16` | Wisdom stat |
| `constitution` | `u16` | Constitution stat |
| `morale` | `u16` | Morale stat |
| `hp_current` | `u16` | Current HP |
| `hp_max` | `u16` | Maximum HP |
| `mp_current` | `u16` | Current MP |
| `mp_max` | `u16` | Maximum MP |
| `xp` | `u32` | Experience points |
| `level` | `u8` | Character level |
| `unspent_stat_points` | `u8` | Unspent stat points |
| `gold` | `u32` | Gold amount |
| `offense` | `u16` | Offense stat |
| `defense` | `u16` | Defense stat |
| `dodge` | `u8` | Dodge rate |
| `hit` | `u8` | Hit rate |
| `magic_power` | `u16` | Magic power |
| `attack_mod` | `u8` | Attack modifier |
| `thievery` | `u8` | Pickpocketing skill |
| `lockpick` | `u8` | Lockpicking skill |
| `haggle` | `u8` | Haggling skill |
| `perception` | `u8` | Perception skill |
| `traps` | `u8` | Traps skill |
| `sword_lv` | `u8` | Swords skill level |
| `sword_kills` | `u16` | Swords kills |
| `axe_lv` | `u8` | Axes skill level |
| `axe_kills` | `u16` | Axes kills |
| `archery_lv` | `u8` | Archery skill level |
| `archery_kills` | `u16` | Archery kills |
| `polearm_lv` | `u8` | Polearm skill level |
| `polearm_kills` | `u16` | Polearm kills |
| `magic_lv` | `u8` | Magic skill level |
| `magic_kills` | `u16` | Magic kills |
| `holy_lv` | `u8` | Holy magic skill level |
| `holy_kills` | `u16` | Holy magic kills |
| `dark_lv` | `u8` | Dark magic skill level |
| `dark_kills` | `u16` | Dark magic kills |
| `cached_tile_value` | `u16` | Cached pathfinder tile value |
| `combat_action_state` | `u8` | Combat action state |
| `reserved_05c` | `u16` | Reserved |
| `reserved_05d` | `u16` | Reserved |
| `ui_hover_active` | `u8` | UI hover active flag |
| `status_effect_stack` | `u8` | Status effect stack count |

### Character Identity (`character_identity`)

35 bytes containing character name, class, and spell-bar state:

| Field | Type | Description |
|-------|------|-------------|
| `player_name` | String (11 bytes) | Player name (WINDOWS-1250, null-terminated) |
| `player_class_id` | `u16` | Player class ID |
| `player_class_name` | String (20 bytes) | Player class name (WINDOWS-1250, null-terminated) |
| `selected_spell_ui_index` | `u16` | UI index of selected spell |

**Note**: The full identity block in the save file is larger than 35 bytes. The `CharacterIdentity::read_from` method skips over many runtime fields (inventory serial counters, waypoint data, movement state, pathfinding scratch, etc.) and only retains the trailing 35 bytes.

### Inventory Data (`inventory`)

Raw inventory data for 5 item categories, each prefixed with a `u16` count:

| Category | Record Size | Description |
|----------|-------------|-------------|
| Event items | 244 bytes | Event-type items |
| Misc items | 264 bytes | Misc-type items |
| Edit items | 272 bytes | Edit-type items |
| Weapon items | 292 bytes | Weapon-type items |
| Heal items | 256 bytes | Heal-type items |

Each item record contains a copy of its database definition followed by save-runtime fields.
The runtime category is zero-based: `0`=weapon, `1`=heal, `2`=edit,
`3`=misc, and `4`=event. This numbering differs from the one-based item type
used by some map and object files.

| Category | Runtime fields after/copied into the definition |
|----------|--------------------------------------------------|
| Event | Category, alignment byte, category-local record index |
| Misc | Category, category-local record index, global inventory-instance ID |
| Edit | Category, alignment byte, category-local record index |
| Weapon | Category, global inventory-instance ID |
| Heal | Reserved definition byte, category, category-local record index, two runtime scratch bytes |

The global inventory-instance ID connects a weapon to equipped slots and connects
weapon or miscellaneous items to inventory placement cells. Heal runtime scratch
bytes are normally zero, but saved games show that they are not initialized consistently.

### Inventory Slots (`inventory_slots`)

3,780 bytes (108 + 96 + 3,576) containing:

| Section | Size | Description |
|---------|------|-------------|
| Equipped equipment | 108 bytes (12 × 9) | 12 equipment slots |
| Belt potions | 96 bytes (6 × 16) | 6 belt item cells |
| Inventory placement | 3,576 bytes (189 × 20) | 3 pages × 7 columns × 9 cells |

### Learned Spells (`learned_spells`)

41 bytes, one byte per spell, indicating whether each spell has been learned.

### Party Members (`party_members`)

Variable-length records, each consisting of:
- A 21-byte name (WINDOWS-1250, null-terminated)
- A 300-byte `PartyMemberBinaryRecord` containing stats, position, AI state, and combat data
- An optional 52-byte combat snapshot (48 bytes + 4-byte terminator), present when the companion has an active combat object

The 300-byte payload is a serialized stream of overlapping four-byte runtime
snapshots. Adjacent snapshots frequently begin one or two bytes apart. As a
result, the stream repeats parts of health, mana, class, level, spell IDs,
animation flags, coordinates, and other fields. The parser exposes the primary
values and preserves each repeated snapshot tail or overlap under an explicit
`*_snapshot_tail` or `*_snapshot_overlap` name.

### Events (`events`)

2,251 fixed-size records (284 bytes each):

| Field | Type | Description |
|-------|------|-------------|
| `event_id` | `u32` | Event identifier and fixed-table index |
| `required_event_id` | `u32` | Event whose triggered state controls conditional event types |
| `event_type` | `u32` | Dispatch rule: `0`=once unconditionally, `1`=limited unconditionally, `2`=always unconditionally, `3`=once while required event is untriggered, `4`=limited while required event is untriggered, `5`=always while required event is untriggered, `6`=once after required event triggers, `7`=limited after required event triggers, `8`=always after required event triggers |
| `script_filename` | String (260 bytes) | Script filename (WINDOWS-1250, null-terminated) |
| `execution_limit` | `u32` | Trigger limit used by event types `1`, `4`, and `7` |
| `execution_count` | `u32` | Number of times dispatch has started |
| `has_triggered` | `u32` | Triggered state: `0`=not triggered, `1`=triggered |

### Post-Events Data (`post_events`)

Character movement-event log between the events and journal sections:

| Field | Type | Description |
|-------|------|-------------|
| `shake_active` | `u32` | Screen-shake active flag (offset `+0x244`; zero in shipped saves) |
| `shake_frames_remaining` | `u32` | Screen-shake frames remaining (offset `+0x248`; zero in shipped saves) |
| `walk_milestones` | `Vec<WalkMilestoneRecord>` | Walk milestone events (count × 24 bytes; container `+0x208`) |
| `walk_completions` | `Vec<WalkCompletionRecord>` | Walk completion events (count × 24 bytes; container `+0x21c`) |
| `recruitable_companion_world_presence` | `[u32; 8]` | World-presence state for each recruitable companion (`0`=removed from map, `1`=available in world) |
| `dismissed_companion_progression` | `[DismissedCompanionProgression; 8]` | Retained progression for companions that previously left the party |

Each dismissed-companion progression record contains three bytes:

| Field | Type | Description |
|-------|------|-------------|
| `is_saved` | `u8` | Whether retained progression exists (`0`=none, `1`=present) |
| `companion_level` | `u8` | Companion level when the progression was retained |
| `player_level` | `u8` | Player level when the progression was retained |

Milestone records are written at walk animation milestones (walk progress
reaching `path_length - 2`, or the walk frame counter reaching 4). Completion
records are written when a walk cycle finishes.

#### Walk Milestone Record (24 bytes)

| Field | Type | Description |
|-------|------|-------------|
| `id` | `u32` | Event id: `400` in the 1.45 binary; `10`/`100`/`200`/`300` while the walk-freshness counter is active; ascending global counter in shipped saves |
| `direction` | `u32` | Walk direction (animation-step direction, 0-7) |
| `state` | `u32` | Character state byte (`1` while walking) |
| `walk_type` | `u32` | Walk-type flag: `0` in the 1.45 binary (which duplicates `direction` here); `1` in shipped saves |
| `x` | `u32` | X coordinate |
| `y` | `u32` | Y coordinate |

#### Walk Completion Record (24 bytes)

| Field | Type | Description |
|-------|------|-------------|
| `id` | `u32` | Event id: `2000` in the 1.45 binary; ascending global counter in shipped saves |
| `direction` | `u32` | Normalized walk direction (0-3; walk directions 4-7 map to 0-3) |
| `diagonal` | `u32` | Diagonal flag (`1` for diagonal walk directions) |
| `character_index` | `u32` | Character index (`0` for party members in the 1.45 binary; `0`-`2` in shipped saves) |
| `x` | `u32` | X coordinate |
| `y` | `u32` | Y coordinate |

### Journal Data (`journal`)

| Field | Type | Description |
|-------|------|-------------|
| `header` | `JournalHeader` | 42-byte journal UI state |
| `main` | `Vec<JournalEntry>` | Main quest entries (100 × 37 bytes) |
| `side` | `Vec<JournalEntry>` | Side quest entries (100 × 37 bytes) |
| `trade` | `Vec<JournalEntry>` | Trading offer entries (100 × 37 bytes) |

#### Journal Header (42 bytes)

| Field | Type | Description |
|-------|------|-------------|
| `is_world_map_open` | `u8` | Combined-interface view: `0`=journal, `1`=world map |
| `selected_map_layer` | `u8` | Selected world-map layer (`0`-`2`) |
| `map_marker_discovery` | `WorldMapMarkerDiscovery` | Persistent marker discovery state for all three world-map layers |
| `active_section` | `u8` | Active journal section: `0`=main, `1`=side, `2`=trade |
| `section_first_visible_entries` | `[u8; 3]` | First visible entry index per section |
| `section_selected_entries` | `[u8; 3]` | Selected entry index per section |
| `section_entry_counts` | `[u8; 3]` | Active entry count per section |

`WorldMapMarkerDiscovery` divides the 30-byte marker storage as follows:

| Field | Type | Description |
|-------|------|-------------|
| `layer_0` | `[u8; 10]` | Layer 0 marker flags: `0`=hidden, `1`=discovered |
| `layer_1` | `[u8; 10]` | Layer 1 marker flags: `0`=hidden, `1`=discovered |
| `layer_2` | `[u8; 7]` | Layer 2 marker flags: `0`=hidden, `1`=discovered |
| `unused_layer_2_slots` | `[u8; 3]` | Unused tail of layer 2's ten-slot storage |

#### Journal Entry (37 bytes)

| Field | Type | Description |
|-------|------|-------------|
| `entry_index` | `u8` | Slot within this journal section |
| `quest_title` | String (32 bytes) | Quest title (WINDOWS-1250, null-terminated) |
| `quest_id` | `u8` | Quest ID from Quest.scr |
| `follow_up_quest_id_1` | `u8` | First linked follow-up quest ID, or `0` if absent |
| `follow_up_quest_id_2` | `u8` | Second linked follow-up quest ID, or `0` if absent |
| `is_completed` | `u8` | Completion flag: `0`=active, `1`=completed |

## Binary Record Macro

Several structs in this module use the `#[derive(BinaryRecord)]` macro from `dispel-macros`, which generates `parse` and `write` methods for binary serialization. The `#[binary_record(...)]` attribute controls field encoding, including:

- `string(encoding = "WINDOWS-1250", size = N)`: Fixed-size null-terminated string
- `size = N`: Fixed-size byte array
- `inventory_item(wire_type = "i32")`: Custom enum encoding

## Validation

The writer performs preflight validation before serializing:

1. **Count limits**: All collection counts must fit in their wire types (`u16` or `u32`)
2. **Fixed collection lengths**: Sprite paths (4), learned spells (41), events (2,251), journal entries (100 per section), map viewport cells (500)
3. **Cross-field consistency**: Visited-map count must match map section count and map ID count; party member count must match the stored count
4. **Trailer size validation**: Map extra-object trailer size must match the computed expected size

## Extractor

The save file format is parsed by the `SaveFile` struct in `src/references/save_file/mod.rs`, which implements the `Extractor` trait.

### How to Run

```bash
# Extract save file to JSON
cargo run -- extract -i "fixtures/Dispel/Save/save_file.sav"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```

## Purpose

This file stores the complete player game state, including:
1. Character progression (stats, level, XP, gold)
2. Inventory and equipment
3. Party member recruitment and state
4. Quest journal progress
5. Map exploration state and visited locations
6. Event script flags
7. Isometric viewport state

## Legal Notice

⚠️ **DISCLAIMER**: This documentation describes technical file format specifications only. It does not distribute any copyrighted game content, save data, or proprietary assets. All references to save file structures are for **educational and research purposes** to document file organization and data structures.

**DISPEL®** is a registered trademark. This documentation is **not affiliated with, endorsed by, or sponsored by** the trademark owner.

### Legal Compliance

This documentation:
- Describes **file format specifications only**
- Does **not** distribute any save data or game content
- Focuses on **technical organization**, not creative content
- Uses **generic descriptions** of file purposes
- Maintains **nominal fair use** for trademark references

## Notes

- All integers are little-endian
- Text fields use WINDOWS-1250 encoding
- The `game_tmp_blob_size` field in the header acts as a jump address; the reader seeks to this offset after parsing map sections
- The `CharacterIdentity` struct only retains the trailing 35 bytes of the full identity block, skipping many runtime fields
- Party member records have an optional combat snapshot that is conditionally present based on a marker value
- Reserved runtime and header fields are preserved verbatim for round-trip fidelity.
- The writer validates all fixed-size constraints before serialization to prevent corrupt output
