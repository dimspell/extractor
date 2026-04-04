# Mondun/Monmap Files - Monster Placement References

## Legal Notice

⚠️ **DISCLAIMER**: This documentation describes technical file format specifications only. It does not distribute any copyrighted game content, monster data, or proprietary assets. All references to monster types and placements are for **educational and research purposes** to document file organization and data structures.

**DISPEL®** is a registered trademark. This documentation is **not affiliated with, endorsed by, or sponsored by** the trademark owner.

## Overview

Binary files that define monster placements, coordinates, event triggers, and loot configurations for game maps.

## File Structure

**Location**: `MonsterInGame/` directory
**Encoding**: Binary (Little-Endian)
**Record Size**: 56 bytes per monster entry

## File Types

| File Pattern | Map Type | Description |
|--------------|----------|-------------|
| `Mondun*.ref` | Dungeon | Monster placements for dungeon maps (e.g., Mondun01.ref, Mondun02.ref) |
| `Monmap*.ref` | Overworld | Monster placements for regular/overworld maps (e.g., Monmap1.ref, Monmap2.ref) |

## Binary Format

```
[Header: 4 bytes]
- record_count: i32 (number of monster entries)

[Records: 56 bytes each]
- file_id: i32 (file identifier / record number)
- mon_id: i32 (monster type ID, links to Monster.db)
- pos_x: i32 (tile X coordinate)
- pos_y: i32 (tile Y coordinate)
- padding1: i32 (flag, values: 0 or 1)
- padding2: i32 (flag, values: 0 or 1)
- padding3: i32 (flag, values: always 0)
- padding4: i32 (flag, values: -1, 0, or 1)
- event_id: i32 (event trigger ID, links to Event.ini)
- loot1_item_id: u8 (first loot item ID)
- loot1_item_type: u8 (first loot item type)
- padding6: u8 (values: 0 or 255)
- padding7: u8 (values: 0 or 255)
- loot2_item_id: u8 (second loot item ID)
- loot2_item_type: u8 (second loot item type)
- padding8: u8 (values: 0 or 255)
- padding9: u8 (values: 0 or 255)
- loot3_item_id: u8 (third loot item ID)
- loot3_item_type: u8 (third loot item type)
- padding10: u8 (values: 0 or 255)
- padding11: u8 (values: 0 or 255)
- padding12: i32 (flag, values: -1, 0, or 1)
- padding13: i32 (flag, values: 0 or 1)
```

## Example Files

**Dungeon Maps (Mondun*.ref):**
- Mondun01.ref through Mondun25.ref

**Overworld Maps (Monmap*.ref):**
- Monmap1.ref, Monmap2.ref, Monmap3.ref

## Map.ini Integration

These files are referenced in `Ref/Map.ini` to associate monster placements with specific maps:

```
; Map ID, X, Y, Width, Height, MonsterFile, NPCFile, ExitFile, MapType
1,150,181,424,0,monmap1.ref,npcmap2.ref,Extmap1.ref,3
2,149,136,413,1,monmap2.ref,npcmap2.ref,Extmap2.ref,5
14,0,145,25,8,mondun01.ref,null,Extdun01.ref,8
16,0,148,25,9,mondun02.ref,null,Extdun02.ref,9
```

## Field Details

**mon_id**: Links to monster definitions in `Monster.db`. Determines monster type, stats, and appearance.

**pos_x, pos_y**: Tile coordinates where the monster spawns on the map.

**event_id**: Links to event definitions in `Event.ini`. Triggers events when the monster is interacted with or defeated.

**padding1–4, padding12–13**: Unknown flag fields with constrained value ranges. Likely control monster behavior (patrol, chase, spawn conditions).

**padding6–11**: Byte-sized padding fields, typically 0 or 255 (0xFF). May be alignment or unused flags.

**loot*_item_id**: Item IDs that the monster can drop when defeated.

**loot*_item_type**: Item type from the ItemTypeId enum (Weapon=1, Armor, Healing=2, Misc=4, Edit=3, Event=5, Other=255).

## Usage in Game

1. Game loads map from `Map.ini`
2. References the associated monster file (Mondun*.ref or Monmap*.ref)
3. Spawns monsters at specified coordinates
4. Configures loot drops based on item IDs and types
5. Links monster behavior to `mon_id` definitions
6. Triggers events via `event_id` when conditions are met

## Monster Type IDs
- Links to `Monster.db` entries
- Identifies specific monster types
- Determines monster appearance and stats

## Loot System

### Loot Slots
- 3 loot slots per monster
- Each slot has item ID and type
- Items dropped when monster is defeated

### Item Types
- `Weapon` (1): Weapons and combat items
- `Armor` (1): Protective gear (shares type ID with Weapon)
- `Healing` (2): Health restoration items
- `Misc` (4): Various utility items
- `Edit` (3): Modifiable equipment
- `Event` (5): Quest-related objects
- `Other` (255): Undefined/catch-all

## File Purpose
Defines monster placements on specific maps with exact coordinates, event triggers, and loot drop configurations. Used for:
- Populating dungeons with enemies
- Setting up ambush points
- Creating balanced combat encounters
- Distributing loot rewards
- World building and difficulty scaling

## Related Files
- `Monster.db` - Monster definitions and statistics
- `Monster.ini` - Monster visual/sprite data
- `Event.ini` - Event definitions referenced by `event_id`
- `*.map` files - Map geometry and tiles
- `AllMap.ini` - Map metadata and associations

## Implementation
- **Rust Module**: `src/references/monster_ref.rs`
- **Editor**: `src/references/monster_ref_editor.rs` (EditableRecord impl)
- **Extractor**: `MonsterRef` struct implementing `Extractor` trait
- **Data Structure**: `MonsterRef` with position, event, and loot data
- **Database**: Saved to SQLite via `save_monster_refs` function

## Example Usage

### Extract monster placements (new CLI):
```bash
cargo run -- extract -i fixtures/Dispel/MonsterInGame/Mondun01.ref
```

### Extract monster placements (legacy):
```bash
cargo run -- ref monster-ref "fixtures/Dispel/MonsterInGame/Mondun01.ref"
```

### Import to database:
```bash
cargo run -- database import "fixtures/Dispel/"
```

## Coordinate System
- Isometric tile-based coordinates
- Each tile is 32×32 pixels
- Origin typically at top-left of map
- Y-axis increases downward

## Legal Compliance

This documentation:
- Describes **file format specifications only**
- Does **not** distribute any monster data or game content
- Focuses on **binary structure and organization**, not creative content
- Uses **generic descriptions** of file purposes
- Maintains **nominal fair use** for trademark references

## Technical Notes

- Both file types use identical binary format
- Distinction is organizational (dungeon vs overworld)
- Padding fields have constrained value ranges (see Binary Format above)
- Files are processed by `MonsterRef` struct in the codebase
- **No copyrighted game content** is reproduced or distributed

## Extractor

An extractor and GUI editor are available for this file format.

### CLI Commands

```bash
# Extract to JSON (auto-detects type by filename)
cargo run -- extract -i fixtures/Dispel/MonsterInGame/Mondun01.ref

# Extract with type override
cargo run -- extract -i unknown_file.ref --type monster_ref

# Validate extracted JSON
cargo run -- validate -i monsteref.json --type monster_ref
```

### GUI Editor

The MonsterRef editor provides a 3-panel interface:
1. **File list** — discovered Mondun*/Monmap*.ref files
2. **Record list** — monster placements in the selected file
3. **Record editor** — editable fields with monster name dropdown (loaded from Monster.ini)

All saves create a timestamped `.bak` backup automatically.
