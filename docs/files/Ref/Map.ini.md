# Map.ini Documentation

## Legal Notice

⚠️ **DISCLAIMER**: This documentation describes technical file format specifications only. It does not distribute any copyrighted game content, map data, or proprietary assets. All references to map configurations are for **educational and research purposes** to document file organization and data structures.

**DISPEL®** is a registered trademark. This documentation is **not affiliated with, endorsed by, or sponsored by** the trademark owner.

## Overview
`Map.ini` is a map initialization data file used by the game engine to configure individual map properties, starting positions, and associated resources.

## File Format
- **Encoding**: EUC-KR
- **Format**: CSV (Comma-Separated Values)
- **Comment lines**: Lines starting with `;` are ignored
- **Location**: `Dispel/Ref/Map.ini` (relative to game installation directory)

## Structure
Each line represents one map initialization record with the following fields:

```
id,camera_event,start_x,start_y,map_id,monsters,npcs,extras,cd_track
```

### Field Definitions
| Field | Type | Description |
|-------|------|-------------|
| `id` | Integer | Unique map identifier |
| `camera_event` | Integer | Event ID triggered when camera moves |
| `start_x` | Integer | Initial player X coordinate (isometric tiles) |
| `start_y` | Integer | Initial player Y coordinate (isometric tiles) |
| `map_id` | Integer | Target map ID for linking/transition |
| `monsters` | String/Null | Monster placement REF filename or "null" |
| `npcs` | String/Null | NPC placement REF filename or "null" |
| `extras` | String/Null | Extra interactive objects REF filename or "null" |
| `cd_track` | Integer | Background music CD track number |

## Example Entries

```
; Default values
0,0,0,0,0,null,null,null,0

; First start
1,150,181,424,0,monmap1.ref,npcmap1.ref,Extmap1.ref,3

; Moving between sections
2,149,136,413,1,monmap2.ref,npcmap2.ref,Extmap2.ref,5
```

## Special Values
- **"null"**: Used for `monsters`, `npcs`, and `extras` fields when the file is absent
- **Comments**: Lines starting with `;` are ignored
- **Coordinates**: Use isometric tile coordinate system
- **Event IDs**: Special event numbers that trigger when camera moves

## Purpose
This file defines initialization parameters for each map including:
1. Starting player positions (X,Y coordinates)
2. Linked resource files (monsters, NPCs, extra objects)
3. Background music track selection
4. Map transition and linking information
5. Camera movement event triggers

The game engine uses this data during map loading to:
- Position the player correctly
- Load appropriate monster/NPC placements
- Set up interactive objects
- Play the correct background music
- Handle map transitions and camera events

## Legal Compliance

This documentation:
- Describes **file format specifications only**
- Does **not** distribute any map data or game content
- Focuses on **technical organization**, not creative content
- Uses **generic descriptions** of file purposes
- Maintains **nominal fair use** for trademark references

## Notes
- The file is parsed by the `MapIni` struct in `src/references/map_ini.rs`
- Map initialization records are stored in a database using the `save_map_inis` function
- The file format is strictly enforced with EUC-KR encoding
- Coordinate system uses isometric tiles, not pixel coordinates
- **No copyrighted game content** is reproduced or distributed

## Related Files Location
- **Monster placement files** (`mon*.ref`): `Dispel/MonsterInGame/` directory
- **NPC placement files** (`npc*.ref`): `Dispel/NpcInGame/` directory
- **Extra objects files** (`Ext*.ref`): `Dispel/ExtraInGame/` directory

## Extractor

An extractor is available in `src/references/map_ini.rs` to parse this file format.

### How to Run

```bash
# Extract Map.ini to JSON
cargo run -- extract -i "fixtures/Dispel/Ref/Map.ini"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
