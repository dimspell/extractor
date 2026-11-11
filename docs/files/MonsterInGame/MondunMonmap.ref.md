# Mondun/Monmap Files - Monster Placement References

> DISPEL® is a registered trademark. This project is not affiliated with,
> endorsed by, or sponsored by the trademark owner.

## File Information

- **Location**: `MonsterInGame/` directory
- **Encoding**: Binary (Little-Endian)
- **Record Size**: 56 bytes per monster entry

Binary files that define monster placements, coordinates, event triggers, and loot configurations for game maps.

## File Types

| File Pattern  | Map Type  | Description                                                                    |
|---------------|-----------|--------------------------------------------------------------------------------|
| `Mondun*.ref` | Dungeon   | Monster placements for dungeon maps (e.g., Mondun01.ref, Mondun02.ref)         |
| `Monmap*.ref` | Overworld | Monster placements for regular/overworld maps (e.g., Monmap1.ref, Monmap2.ref) |

## Binary Format

```
[Header: 4 bytes]
- record_count: i32 (number of monster entries)

[Records: 56 bytes each: 14 little-endian i32 values]

| Offset | Field | Meaning |
|---:|---|---|
| 0 | placement_id | Map-local monster-placement identifier. |
| 4 | monster_db_id | One-based monster ID, used by `Monster.db` and `Monster.ini`. |
| 8 | map_x | Spawn tile X coordinate. |
| 12 | map_y | Spawn tile Y coordinate. |
| 16 | initial_patrol_countdown | Initial patrol/scan countdown. |
| 20 | skip_ai_action | When set, skips an AI action branch. |
| 24 | initial_active_flag | Initial active/awake runtime flag. Original map files observed so far use zero. |
| 28 | ai_type_override | `-1` uses the `Monster.db` AI type; 0 or 1 overrides it. |
| 32 | event_id_on_kill | Event ID triggered after this monster dies. |
| 36 | loot_item_1 | First packed `InventoryItem` loot slot. |
| 40 | loot_item_2 | Second packed `InventoryItem` loot slot. |
| 44 | loot_item_3 | Third packed `InventoryItem` loot slot. |
| 48 | drop_all_loot | `1` drops all populated loot slots; other observed values select a slot. |
| 52 | force_ai_update | `1` runs the AI update path even when the normal active flag is clear. |
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

**monster_db_id**: Links to monster definitions in `Monster.db`. Determines monster type, stats, and appearance.

**map_x, map_y**: Tile coordinates where the monster spawns on the map.

**event_id_on_kill**: Links to the event triggered when the monster dies.

**loot_item_1..3**: Packed 32-bit `InventoryItem` values. The low bytes contain the item ID and type; the upper bits are
preserved when writing.

## Loot System

### Loot Slots

- 3 loot slots per monster
- Each slot has item ID and type
- Items dropped when monster is defeated

### Item Types

- `Weapon` (1): Weapons and combat items
- `Healing` (2): Health restoration items
- `Edit` (3): Modifiable equipment
- `Event` (4): Quest-related objects
- `Misc` (5): Various utility items
- `Other` (255): Undefined/catch-all

## Related Files

- `Monster.db` - Monster definitions and statistics
- `Monster.ini` - Monster visual/sprite data
- `Event.ini` - Event definitions referenced by `event_id_on_kill`
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
cargo run -- extract -i "fixtures/Dispel/MonsterInGame/Mondun01.ref"
```

### Import to database:

```bash
cargo run -- database import "fixtures/Dispel/"
```

## Coordinate System

- Tile-based coordinates
- Y-axis increases downward

## Technical Notes

- Both file types use identical binary format
- Distinction is organizational (dungeon vs overworld)
- Padding fields have constrained value ranges (see Binary Format above)
- Files are processed by `MonsterRef` struct in the codebase

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
