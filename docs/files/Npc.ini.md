# Npc.ini documentation

## File information

### Overview

The tool reads `Npc.ini` as a CSV file with comments. Each row holds a visual type ID, a sprite filename and a description.

### File structure

- **Location**: `Npc.ini`
- **Encoding**: EUC-KR (Korean character encoding)
- **Format**: CSV (Comma-Separated Values) with comments

### Format specification

```ini
; Comment line
id,sprite_filename,description
1,guard.spr,City Guard
2,merchant.spr,Shopkeeper
...
```

### Field definitions

| Field           | Type   | Description                                      |
|-----------------|--------|--------------------------------------------------|
| id              | i32    | Unique NPC visual type identifier                |
| sprite_filename | string | SPR filename or "null" for no sprite             |
| description     | string | NPC role/appearance description (EUC-KR encoded) |

### Special values

- `null`: The tool reads this string when no sprite filename is present.
- `;`: Lines starting with a semicolon are comments.
- Empty lines: The parser ignores them.

### Example entries

```ini
; Party members
1,Party1.spr,Party Member 1
2,Party2.spr,Party Member 2

; Guards
9,guard1.spr,City Guard
10,guard2.spr,Town Guard

; Kings
16,King1.spr,Royal Guard
17,King2.spr,Royal Guard
```

### Technical details

**Encoding**: EUC-KR (Extended Unix Code Korea)

- The encoding covers Korean characters in descriptions.
- The reader and writer must use this encoding.

**File processing:**

- The parser ignores comment lines that start with `;`.
- It skips empty lines.
- It splits rows on commas.
- It reads `null` as a missing sprite filename.

### Usage

1. The tool reads NPC definitions from `Npc.ini`.
2. It links visual type IDs to sprite files.
3. It stores each description with its ID.
4. Other file formats reference these IDs.

### Notes

- Some entries have a `null` sprite filename.
- Descriptions are in Korean and use EUC-KR encoding.
- The file uses Windows-style line endings (`\r\n`).
- IDs may have gaps in the sequence.

## Extractor

The `NpcIni` struct in `src/references/npc_ini.rs` parses the file. It implements the `Extractor` trait.

### How to run

```bash
# Extract Npc.ini to JSON
cargo run -- extract -i "Npc.ini"

# Import to SQLite database
cargo run -- database import "path/to/Dispel/" "database.sqlite"
```
