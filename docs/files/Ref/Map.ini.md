# Map.ini Documentation

## File Information

- **Location**: `Dispel/Ref/Map.ini` (relative to game installation directory)
- **Encoding**: EUC-KR
- **Format**: CSV (Comma-Separated Values)
- **Comment lines**: The parser ignores lines starting with `;`.

The tool reads `Map.ini` as a map initialization list. Each row holds a map ID, coordinates, an event ID and resource file names.

## Structure

Each line represents one map initialization record with the following fields:

```
id,camera_event,start_x,start_y,map_id,monsters,npcs,extras,cd_track
```

### Field Definitions

| Field          | Type        | Description                                      |
|----------------|-------------|--------------------------------------------------|
| `id`           | Integer     | Unique map identifier                            |
| `camera_event` | Integer     | Event ID triggered when camera moves             |
| `start_x`      | Integer     | Initial player X coordinate (isometric tiles)    |
| `start_y`      | Integer     | Initial player Y coordinate (isometric tiles)    |
| `map_id`       | Integer     | Target map ID for linking/transition             |
| `monsters`     | String/Null | Monster placement REF filename or "null"         |
| `npcs`         | String/Null | NPC placement REF filename or "null"             |
| `extras`       | String/Null | Extra interactive objects REF filename or "null" |
| `cd_track`     | Integer     | Background music CD track number                 |

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
- **Comments**: The parser ignores lines starting with `;`.
- **Coordinates**: Use isometric tile coordinate system
- **Event IDs**: Special event numbers that trigger when camera moves

## Purpose

This file defines initialization parameters for each map including:

1. Start positions (X,Y coordinates)
2. Linked resource files (monsters, NPCs, extra objects)
3. Background music track selection
4. Map transition and linking information
5. Camera movement event triggers

The tool stores these values:

- Start coordinates
- Monster and NPC placement file names
- Extra object file names
- Music track number
- Camera event and target map IDs

## Notes

- The `MapIni` struct in `src/references/map_ini.rs` parses the file.
- The `save_map_inis` function stores the records in a database.
- The file format uses EUC-KR encoding
- Coordinate system uses isometric tiles, not pixel coordinates

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
