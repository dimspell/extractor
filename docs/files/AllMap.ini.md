# AllMap.ini Documentation

> **DISPEL®** is a registered trademark. This project is not affiliated with, endorsed by, or sponsored by the trademark
> owner.

## File Information

- **Location**: `Dispel/AllMap.ini` (relative to game installation directory)
- **Encoding**: WINDOWS-1250
- **Format**: CSV (Comma-Separated Values)
- **Comment lines**: Lines starting with `;` are ignored

`AllMap.ini` is a master map list file used by the game engine to index all available maps and their associated
resources.

## Structure

Each line represents one map with the following fields:

```
id,map_file,name,pgp,dlg,lit
```

### Field Definitions

| Field      | Type        | Description                                                   |
|------------|-------------|---------------------------------------------------------------|
| `id`       | Integer     | Unique map identifier                                         |
| `map_file` | String      | Filename of the .map file without an extension (e.g., "cat1") |
| `name`     | String      | Display name shown in-game                                    |
| `pgp`      | String/Null | Conversation script filename or "null" if absent              |
| `dlg`      | String/Null | Dialog text filename or "null" if absent                      |
| `lit`      | Integer     | Lighting indicator: `0` = dark/dungeon, `1` = lit/outdoor     |

### Example Entries

```
1,cat1,Forest,Pgpcat1.pgp,Dlgcat1.dlg,1
2,cat2,Dungeon,Pgpcat2.pgp,Dlgcat2.dlg,0
3,cat3,Village,null,null,1
```

### Special Values

- **"null"**: Used for `pgp` and `dlg` fields when the file is absent
- **Lighting**: `0` indicates dark/dungeon maps, `1` indicates lit/outdoor maps

## Purpose

This file serves as the master index for all game maps, linking map IDs to their respective filenames and metadata. The
game engine uses this file to:

1. Load the correct map files
2. Associate party (PGP) and dialog (DLG) files with maps
3. Determine lighting conditions for rendering

## Parser

The file is parsed by the `Map` struct in `src/references/all_map_ini.rs`, which implements the `Extractor` and
`Localizable` traits.

### How to Run

```bash
# Extract AllMap.ini to JSON
cargo run -- extract -i "AllMap.ini"

# Import to SQLite database
cargo run -- database import "path/to/Dispel/" "database.sqlite"
```
