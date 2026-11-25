# AllMap.ini documentation

## File information

- **Location**: `Dispel/AllMap.ini` (relative to game installation directory)
- **Encoding**: WINDOWS-1250
- **Format**: CSV (Comma-Separated Values)
- **Comment lines**: The parser ignores lines starting with `;`

The tool reads `AllMap.ini` as a master map list. Each row links a map ID to a map file name, two dialogue file names and a lighting value.

## Structure

Each non-comment line holds six comma-separated fields:

```
id,map_file,name,pgp,dlg,lit
```

### Field definitions

| Field      | Type        | Description                                                   |
|------------|-------------|---------------------------------------------------------------|
| `id`       | Integer     | Unique map identifier                                         |
| `map_file` | String      | Filename of the .map file without an extension (e.g., "cat1") |
| `name`     | String      | Display name shown in-game                                    |
| `pgp`      | String/Null | Conversation script filename or "null" if absent              |
| `dlg`      | String/Null | Dialog text filename or "null" if absent                      |
| `lit`      | Integer     | Lighting indicator: `0` = dark/dungeon, `1` = lit/outdoor     |

### Example entries

```
1,cat1,Forest,Pgpcat1.pgp,Dlgcat1.dlg,1
2,cat2,Dungeon,Pgpcat2.pgp,Dlgcat2.dlg,0
3,cat3,Village,null,null,1
```

### Special values

- **"null"**: The tool reads this string when a `pgp` or `dlg` value is absent.
- **Light values**: `0` marks dark or dungeon maps. `1` marks lit or outdoor maps.

## Tool use

The tool links each map ID to a map file name. It also stores the dialogue file names and the lighting value.

## Parser

The `Map` struct in `src/references/all_map_ini.rs` parses the file. It implements the `Extractor` and `Localizable` traits.

### How to run

```bash
# Extract AllMap.ini to JSON
cargo run -- extract -i "AllMap.ini"

# Import to SQLite database
cargo run -- database import "path/to/Dispel/" "database.sqlite"
```
