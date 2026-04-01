# AllMap.ini Documentation

## Overview
`AllMap.ini` is a master map list file used by the game engine to index all available maps and their associated resources.

## File Format
- **Encoding**: WINDOWS-1250
- **Format**: CSV (Comma-Separated Values)
- **Comment lines**: Lines starting with `;` are ignored
- **Location**: `Dispel/AllMap.ini` (relative to game installation directory)

## Structure
Each line represents one map with the following fields:

```
id,map_file,name,pgp,dlg,lit
```

### Field Definitions
| Field | Type | Description |
|-------|------|-------------|
| `id` | Integer | Unique map identifier |
| `map_file` | String | Filename of the .map file without an extension (e.g., "map1") |
| `name` | String | Display name shown in-game |
| `pgp` | String/Null | Conversation script filename or "null" if absent |
| `dlg` | String/Null | Dialog text filename or "null" if absent |
| `lit` | Integer | Lighting indicator: `0` = dark/dungeon, `1` = lit/outdoor |

## Example Entries

```
0,map1,Aesh,Pgpmap1.pgp,Dlgmap1.dlg,0
1,map2,Shereg,Pgpmap2.pgp,Dlgmap2.dlg,0
2,map3,Yam,Pgpmap3.pgp,Dlgmap3.dlg,0
```

## Special Values
- **"null"**: Used for `pgp` and `dlg` fields when the file is absent
- **Lighting**: `0` indicates dark/dungeon maps, `1` indicates lit/outdoor maps

## Purpose
This file serves as the master index for all game maps, linking map IDs to their respective filenames and metadata. The game engine uses this file to:
1. Load the correct map files
2. Associate party (PGP) and dialog (DLG) files with maps
3. Determine lighting conditions for rendering

## Notes
- The file is parsed by the `Map` struct in `src/references/all_map_ini.rs`
- Maps are stored in a database using the `save_maps` function
- The file format is strictly enforced with WINDOWS-1250 encoding
