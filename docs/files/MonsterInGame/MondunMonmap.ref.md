# Mondun/Monmap Files - Monster Placement References

## Legal Notice

⚠️ **DISCLAIMER**: This documentation describes technical file format specifications only. It does not distribute any copyrighted game content, monster data, or proprietary assets. All references to monster types and placements are for **educational and research purposes** to document file organization and data structures.

**DISPEL®** is a registered trademark. This documentation is **not affiliated with, endorsed by, or sponsored by** the trademark owner.

## Overview

Binary files that define monster placements, coordinates, and loot configurations for game maps.

## File Structure

**Location**: `MonsterInGame/` directory
**Encoding**: Binary (Little-Endian)
**Record Size**: 56 bytes per monster entry

## File Types

| File Pattern | Map Type | Description |
|--------------|----------|-------------|
| `mondun*.ref` | Dungeon | Monster placements for dungeon maps (e.g., mondun01.ref, mondun02.ref) |
| `Monmap*.ref` | Overworld | Monster placements for regular/overworld maps (e.g., Monmap1.ref, Monmap2.ref) |

## Binary Format

```
[Header: 4 bytes]
- record_count: i32 (number of monster entries)

[Records: 56 bytes each]
- file_id: i32 (file identifier)
- mon_id: i32 (monster type ID, links to Monster.db)
- pos_x: i32 (tile X coordinate)
- pos_y: i32 (tile Y coordinate)
- padding1-5: i32 × 5 (unused)
- loot1_item_id: u8 (first loot item ID)
- loot1_item_type: u8 (first loot item type)
- padding6-7: u8 × 2 (unused)
- loot2_item_id: u8 (second loot item ID)
- loot2_item_type: u8 (second loot item type)
- padding8-9: u8 × 2 (unused)
- loot3_item_id: u8 (third loot item ID)
- loot3_item_type: u8 (third loot item type)
- padding10-11: u8 × 2 (unused)
- padding12-13: i32 × 2 (unused)
```

## Example Files

**Dungeon Maps (mondun*.ref):**
- mondun01.ref - Goblin Dungeon monsters
- mondun02.ref - Bandit Dungeon monsters
- mondun03.ref - Tomb of the Last Pope (level 1)
- mondun04.ref - Tomb of the Last Pope (level 2)
- ... up to mondun25.ref

**Overworld Maps (monmap*.ref):**
- Monmap1.ref - Aesh region monsters
- Monmap2.ref - Shereg region monsters
- Monmap3.ref - Yam region monsters

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

**loot*_item_id**: Item IDs that the monster can drop when defeated.

**loot*_item_type**: Item type (weapon, armor, potion, etc.) from the ItemTypeId enum.

## Usage in Game

1. Game loads map from `Map.ini`
2. References the associated monster file (mondun*.ref or monmap*.ref)
3. Spawns monsters at specified coordinates
4. Configures loot drops based on item IDs and types
5. Links monster behavior to `mon_id` definitions

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
- `Weapon`: Weapons and combat items
- `Armor`: Protective gear
- `Healing`: Health restoration items
- `Misc`: Various utility items
- `Quest`: Quest-related objects

### Loot Probability
- Each slot has independent drop chance
- Multiple items can drop from one monster
- Loot quality scales with monster difficulty

## File Purpose
Defines monster placements on specific maps with exact coordinates and loot drop configurations. Used for:
- Populating dungeons with enemies
- Setting up ambush points
- Creating balanced combat encounters
- Distributing loot rewards
- World building and difficulty scaling

## Monster Placement Files
- **Dungeons**: `Mondun01.ref` through `Mondun25.ref`
- **Maps**: `Monmap1.ref`, `Monmap2.ref`, `Monmap3.ref`
- **Special**: Various map-specific monster files

## Related Files
- `Monster.db` - Monster definitions and statistics
- `Monster.ini` - Monster initialization data
- `*.map` files - Map geometry and tiles
- `AllMap.ini` - Map metadata and associations

## Implementation
- **Rust Module**: `src/references/monster_ref.rs`
- **Extractor**: `MonsterRef` struct implementing `Extractor` trait
- **Data Structure**: `MonsterRef` with position and loot data
- **Database**: Saved to SQLite via `save_monster_refs` function

## Example Usage

### Extract monster placements:
```bash
cargo run -- ref monster-ref "Dispel/MonsterInGame/Mondun01.ref"
```

### Import to database:
```bash
cargo run -- database import "Dispel/"
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
- Padding fields are unused and typically contain 0x00 or 0xFF values
- Files are processed by `MonsterRef` struct in the codebase
- **No copyrighted game content** is reproduced or distributed

## Extractor

An extractor is available in `src/references/monster_ref.rs` to parse this file format.

### How to Run

```bash
# Extract Mondun01.ref to JSON
cargo run -- ref monster-ref "fixtures/Dispel/MonsterInGame/Mondun01.ref"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
